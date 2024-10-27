use crate::siko::hir::Data::Enum;
use crate::siko::location::Location::Location;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::repeat;

use super::Resolver::Resolver;

pub struct MatchCompiler<'a> {
    ctx: &'a ReportContext,
    bodyLocation: Location,
    branches: Vec<Pattern>,
    resolver: Resolver<'a>,
    errors: Vec<ResolverError>,
}

impl<'a> MatchCompiler<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        bodyLocation: Location,
        branches: Vec<Pattern>,
        moduleResolver: &'a ModuleResolver,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> MatchCompiler<'a> {
        MatchCompiler {
            ctx: ctx,
            bodyLocation: bodyLocation,
            branches: branches,
            resolver: Resolver::new(moduleResolver, variants, enums),
            errors: Vec::new(),
        }
    }

    fn resolve(&self, pattern: &Pattern) -> Pattern {
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                let id = Identifier {
                    name: name.toString(),
                    location: Location::empty(),
                };
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Named(id, args),
                    location: Location::empty(),
                }
            }
            SimplePattern::Bind(_, _) => pattern.clone(),
            SimplePattern::Tuple(args) => {
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Tuple(args),
                    location: Location::empty(),
                }
            }
            SimplePattern::StringLiteral(_) => pattern.clone(),
            SimplePattern::IntegerLiteral(_) => pattern.clone(),
            SimplePattern::Wildcard => pattern.clone(),
        }
    }

    fn generateChoices(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: Location::empty(),
        };
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                let mut result = Vec::new();
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let e = self.resolver.enums.get(enumName).expect("enum not found");
                    for variant in &e.variants {
                        if variant.name == name {
                            continue;
                        }
                        let id = Identifier {
                            name: variant.name.toString(),
                            location: Location::empty(),
                        };

                        let args = repeat(wildcardPattern.clone()).take(variant.items.len()).collect();
                        let pat = Pattern {
                            pattern: SimplePattern::Named(id, args),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                    for (index, arg) in args.iter().enumerate() {
                        let choices = self.generateChoices(arg);
                        for choice in choices {
                            let mut choiceArgs = Vec::new();
                            choiceArgs.extend(args.iter().cloned().take(index));
                            choiceArgs.push(choice);
                            choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier {
                                name: name.toString(),
                                location: Location::empty(),
                            };
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, choiceArgs),
                                location: Location::empty(),
                            };
                            result.push(pat);
                        }
                    }
                }
                result
            }
            SimplePattern::Bind(_, _) => Vec::new(),
            SimplePattern::Tuple(args) => {
                let mut result = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    let choices = self.generateChoices(arg);
                    for choice in choices {
                        let mut choiceArgs = Vec::new();
                        choiceArgs.extend(args.iter().cloned().take(index));
                        choiceArgs.push(choice);
                        choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(choiceArgs),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                }
                result
            }
            SimplePattern::StringLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::IntegerLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::Wildcard => Vec::new(),
        }
    }

    pub fn isMatch(&self, prev: &Pattern, next: &Pattern) -> bool {
        match (&prev.pattern, &next.pattern) {
            (SimplePattern::Named(id1, args1), SimplePattern::Named(id2, args2)) => {
                if id1 == id2 {
                    if args1.len() != args2.len() {
                        return false;
                    }
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        if !self.isMatch(arg1, arg2) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            (SimplePattern::Named(_, _), SimplePattern::Wildcard) => true,
            (SimplePattern::Bind(_, _), SimplePattern::Bind(_, _)) => false,
            (SimplePattern::Bind(_, _), SimplePattern::Wildcard) => false,
            (SimplePattern::Tuple(args1), SimplePattern::Tuple(args2)) => {
                if args1.len() != args2.len() {
                    return false;
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if !self.isMatch(arg1, arg2) {
                        return false;
                    }
                }
                true
            }
            (SimplePattern::Tuple(_), SimplePattern::Wildcard) => true,
            (SimplePattern::StringLiteral(val1), SimplePattern::StringLiteral(val2)) => val1 == val2,
            (SimplePattern::StringLiteral(_), SimplePattern::Wildcard) => true,
            (SimplePattern::IntegerLiteral(val1), SimplePattern::IntegerLiteral(val2)) => val1 == val2,
            (SimplePattern::IntegerLiteral(_), SimplePattern::Wildcard) => true,
            _ => false,
        }
    }

    fn isMissingPattern(&self, alt: &Pattern) -> bool {
        for branch in &self.branches {
            if self.isMatch(alt, &branch) {
                return false;
            }
        }
        true
    }

    fn merge(&self, pat1: &Pattern, pat2: &Pattern) -> Option<Pattern> {
        match (&pat1.pattern, &pat2.pattern) {
            (SimplePattern::Named(id1, args1), SimplePattern::Named(id2, args2)) => {
                if id1 == id2 {
                    let mut args = Vec::new();
                    if args1.len() != args2.len() {
                        return None;
                    }
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        if let Some(arg) = self.merge(arg1, arg2) {
                            args.push(arg)
                        } else {
                            return None;
                        }
                    }
                    Some(Pattern {
                        pattern: SimplePattern::Named(id1.clone(), args),
                        location: pat1.location.clone(),
                    })
                } else {
                    None
                }
            }
            (SimplePattern::Tuple(args1), SimplePattern::Tuple(args2)) => {
                let mut args = Vec::new();
                if args1.len() != args2.len() {
                    return None;
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if let Some(arg) = self.merge(arg1, arg2) {
                        args.push(arg)
                    } else {
                        return None;
                    }
                }
                Some(Pattern {
                    pattern: SimplePattern::Tuple(args),
                    location: pat1.location.clone(),
                })
            }
            (SimplePattern::Wildcard, _) => Some(pat2.clone()),
            (_, SimplePattern::Wildcard) => Some(pat1.clone()),
            _ => None,
        }
    }

    pub fn check(&mut self) {
        //println!("=======================");
        // let mut allDecisions = Vec::new();
        let mut allChoices = BTreeSet::new();
        for branch in &self.branches {
            let branch = self.resolve(branch);
            allChoices.insert(branch.clone());
            //println!("Pattern {}", branch);
            let choices = self.generateChoices(&branch);
            for choice in choices {
                //println!("   Alt: {}", choice);
                allChoices.insert(choice);
            }
        }
        // for choice in &allChoices {
        //     println!("   Alt: {}", choice);
        // }
        let mut allMerged = BTreeSet::new();
        for (index1, choice1) in allChoices.iter().enumerate() {
            let mut merged = false;
            for (index2, choice2) in allChoices.iter().enumerate() {
                if index1 == index2 {
                    continue;
                }
                if let Some(mergedPat) = self.merge(choice1, choice2) {
                    allMerged.insert(mergedPat);
                    merged = true;
                }
            }
            if !merged {
                allMerged.insert(choice1.clone());
            }
        }
        // for choice in &allMerged {
        //     println!("   merged: {}", choice);
        // }
        for branch in self.branches.iter() {
            let resolvedBranch = self.resolve(branch);
            let mut reduced = BTreeSet::new();
            for m in &allMerged {
                if !self.isMatch(&m, &resolvedBranch) {
                    reduced.insert(m.clone());
                }
            }
            if reduced.len() == allMerged.len() {
                self.errors.push(ResolverError::RedundantPattern(branch.location.clone()));
            }
            allMerged = reduced;
        }
        for m in allMerged {
            self.errors.push(ResolverError::MissingPattern(m.to_string(), self.bodyLocation.clone()));
        }

        for err in &self.errors {
            err.reportOnly(self.ctx);
        }
        if !self.errors.is_empty() {
            std::process::exit(1);
        }
    }
}
