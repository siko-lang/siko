use crate::siko::hir::Data::Enum;
use crate::siko::location::Location::Location;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::repeat;

use super::Resolver::Resolver;

pub struct MatchCompiler<'a> {
    bodyLocation: Location,
    branches: Vec<Pattern>,
    resolver: Resolver<'a>,
}

impl<'a> MatchCompiler<'a> {
    pub fn new(
        bodyLocation: Location,
        branches: Vec<Pattern>,
        moduleResolver: &'a ModuleResolver,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> MatchCompiler<'a> {
        MatchCompiler {
            bodyLocation: bodyLocation,
            branches: branches,
            resolver: Resolver::new(moduleResolver, variants, enums),
        }
    }

    fn generateAlternatives(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: pattern.location.clone(),
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
                            location: pattern.location.clone(),
                        };

                        let args = repeat(wildcardPattern.clone()).take(variant.items.len()).collect();
                        let pat = Pattern {
                            pattern: SimplePattern::Named(id, args),
                            location: pattern.location.clone(),
                        };
                        result.push(pat);
                    }
                    for (index, arg) in args.iter().enumerate() {
                        let alternatives = self.generateAlternatives(arg);
                        for alternative in alternatives {
                            let mut altArgs = Vec::new();
                            altArgs.extend(args.iter().cloned().take(index));
                            altArgs.push(alternative);
                            altArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier {
                                name: name.toString(),
                                location: pattern.location.clone(),
                            };
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, altArgs),
                                location: pattern.location.clone(),
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
                    let alternatives = self.generateAlternatives(arg);
                    for alternative in alternatives {
                        let mut altArgs = Vec::new();
                        altArgs.extend(args.iter().cloned().take(index));
                        altArgs.push(alternative);
                        altArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(altArgs),
                            location: pattern.location.clone(),
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

    pub fn isNextUseful(&self, prev: &Pattern, next: &Pattern) -> bool {
        match (&prev.pattern, &next.pattern) {
            (SimplePattern::Named(id1, args1), SimplePattern::Named(id2, args2)) => {
                if id1 == id2 {
                    if args1.len() != args2.len() {
                        return false;
                    }
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        if self.isNextUseful(arg1, arg2) {
                            return true;
                        }
                    }
                    false
                } else {
                    true
                }
            }
            (SimplePattern::Named(_, _), SimplePattern::Wildcard) => !self.generateAlternatives(prev).is_empty(),
            (SimplePattern::Bind(_, _), SimplePattern::Bind(_, _)) => false,
            (SimplePattern::Bind(_, _), SimplePattern::Wildcard) => true,
            (SimplePattern::Tuple(args1), SimplePattern::Tuple(args2)) => {
                if args1.len() != args2.len() {
                    return false;
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if self.isNextUseful(arg1, arg2) {
                        return true;
                    }
                }
                false
            }
            (SimplePattern::Tuple(_), SimplePattern::Wildcard) => !self.generateAlternatives(prev).is_empty(),
            (SimplePattern::StringLiteral(val1), SimplePattern::StringLiteral(val2)) => val1 != val2,
            (SimplePattern::StringLiteral(_), SimplePattern::Wildcard) => true,
            (SimplePattern::IntegerLiteral(val1), SimplePattern::IntegerLiteral(val2)) => val1 != val2,
            (SimplePattern::IntegerLiteral(_), SimplePattern::Wildcard) => true,
            _ => false,
        }
    }

    fn isMissingPattern(&self, alt: &Pattern) -> bool {
        for branch in &self.branches {
            if !self.isNextUseful(branch, &alt) {
                return false;
            }
        }
        true
    }

    pub fn check(&mut self) {
        // let mut allDecisions = Vec::new();
        let mut allAlternatives = BTreeSet::new();
        for (index, branch) in self.branches.iter().enumerate() {
            println!("Pattern {}", branch);
            let alternatives = self.generateAlternatives(&branch);
            for (prevIndex, prev) in self.branches.iter().enumerate() {
                if prevIndex >= index {
                    continue;
                }
                if !self.isNextUseful(prev, branch) {
                    ResolverError::RedundantPattern(branch.location.clone()).report();
                }
            }
            for alt in alternatives {
                println!("   Alt: {}", alt);
                allAlternatives.insert(alt);
            }
        }
        for alt in allAlternatives {
            if self.isMissingPattern(&alt) {
                ResolverError::MissingPattern(alt.to_string(), self.bodyLocation.clone()).report();
            }
        }
    }
}
