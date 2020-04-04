use crate::error::TypecheckError;
use siko_ir::data::TypeDefId;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::walker::Visitor;
use siko_location_info::location_id::LocationId;
use siko_util::Counter;
use std::collections::BTreeMap;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug)]
enum ChoiceKind {
    Binding,
    Variant(TypeDefId, usize),
    String(String),
    Int(i64),
    Char(char),
}

#[derive(Clone)]
struct Choice {
    first_location: Option<LocationId>,
    sub_choices: BTreeMap<ChoiceKind, usize>,
}

impl Choice {
    fn new() -> Choice {
        Choice {
            first_location: None,
            sub_choices: BTreeMap::new(),
        }
    }
}

struct CaseInfo {
    choices: BTreeMap<usize, Choice>,
    next_id: Counter,
}

impl CaseInfo {
    fn new() -> CaseInfo {
        CaseInfo {
            choices: BTreeMap::new(),
            next_id: Counter::new(),
        }
    }

    fn add_first(&mut self) -> usize {
        let id = self.next_id.next();
        let choice = Choice::new();
        self.choices.insert(id, choice);
        id
    }

    fn get_choice(&self, id: usize) -> &Choice {
        self.choices.get(&id).expect("Choice not found")
    }

    fn check_choices(&self, first: usize, program: &Program, errors: &mut Vec<TypecheckError>) {
        let choice = self.get_choice(first);
        let mut bind_found = false;
        let mut variants = Vec::new();
        let mut constant_pattern_found = false;
        for (kind, _) in &choice.sub_choices {
            match kind {
                ChoiceKind::Binding => {
                    bind_found = true;
                }
                ChoiceKind::Variant(id, index) => {
                    variants.push((id, index));
                }
                ChoiceKind::String(_) => {
                    constant_pattern_found = true;
                }
                ChoiceKind::Int(_) => {
                    constant_pattern_found = true;
                }
                ChoiceKind::Char(_) => {
                    constant_pattern_found = true;
                }
            }
        }
        if !bind_found {
            if !variants.is_empty() {
                let typedef_id = variants[0].0;
                let adt = program.typedefs.get(&typedef_id).get_adt();
                if adt.variants.len() != variants.len() {
                    let err = TypecheckError::NonExhaustivePattern(
                        choice.first_location.expect("First location not found"),
                    );
                    errors.push(err);
                }
            }
            if constant_pattern_found {
                let err = TypecheckError::NonExhaustivePattern(
                    choice.first_location.expect("First location not found"),
                );
                errors.push(err);
            }

            for (_, choice) in &choice.sub_choices {
                if errors.is_empty() {
                    self.check_choices(*choice, program, errors);
                }
            }
        }
    }

    fn add_choice(
        &mut self,
        parent: usize,
        kind: ChoiceKind,
        choice_added: &mut bool,
        location_id: LocationId,
    ) -> usize {
        let parent = self
            .choices
            .get_mut(&parent)
            .expect("Parent choice not found");
        let mut newly_added = false;
        let new_id = self.next_id.next();
        if parent.first_location.is_none() {
            parent.first_location = Some(location_id);
        }
        if let Some(bind_id) = parent.sub_choices.get(&ChoiceKind::Binding) {
            return *bind_id;
        }
        let id = *parent.sub_choices.entry(kind).or_insert_with(|| {
            newly_added = true;
            new_id
        });
        if newly_added {
            let choice = Choice::new();
            self.choices.insert(id, choice);
            *choice_added = true;
        }
        id
    }

    fn check_pattern(
        &mut self,
        pattern_id: &PatternId,
        program: &Program,
        parent: usize,
        choice_added: &mut bool,
    ) -> usize {
        let item_info = program.patterns.get(pattern_id);
        match &item_info.item {
            Pattern::Binding(_) => {
                return self.add_choice(
                    parent,
                    ChoiceKind::Binding,
                    choice_added,
                    item_info.location_id,
                );
            }
            Pattern::Wildcard => {
                return self.add_choice(
                    parent,
                    ChoiceKind::Binding,
                    choice_added,
                    item_info.location_id,
                );
            }
            Pattern::Variant(id, index, items) => {
                let mut id = self.add_choice(
                    parent,
                    ChoiceKind::Variant(*id, *index),
                    choice_added,
                    item_info.location_id,
                );
                for item in items {
                    id = self.check_pattern(item, program, id, choice_added);
                }
                id
            }
            Pattern::StringLiteral(s) => {
                return self.add_choice(
                    parent,
                    ChoiceKind::String(s.clone()),
                    choice_added,
                    item_info.location_id,
                );
            }
            Pattern::IntegerLiteral(i) => {
                return self.add_choice(
                    parent,
                    ChoiceKind::Int(i.clone()),
                    choice_added,
                    item_info.location_id,
                );
            }
            Pattern::CharLiteral(i) => {
                return self.add_choice(
                    parent,
                    ChoiceKind::Char(i.clone()),
                    choice_added,
                    item_info.location_id,
                );
            }
            Pattern::Tuple(items) => {
                let mut id = parent;
                for item in items {
                    id = self.check_pattern(item, program, id, choice_added);
                }
                id
            }
            Pattern::Record(_, items) => {
                let mut id = parent;
                for item in items {
                    id = self.check_pattern(item, program, id, choice_added);
                }
                id
            }
            Pattern::Typed(item, _) => {
                return self.check_pattern(item, program, parent, choice_added);
            }
            Pattern::CharRange(..) => {
                // FIXME: add checks for char ranges
                return 0;
            }
            Pattern::Guarded(..) => {
                // FIXME:
                // We cannot check at compile time whether the guard will return True or not,
                // so guarded patterns are ignored for now, we assume that they fail
                // This can be improved by checking the guard expression
                return 0;
            }
        }
    }
}

pub struct PatternChecker<'a> {
    program: &'a Program,
    errors: &'a mut Vec<TypecheckError>,
}

impl<'a> PatternChecker<'a> {
    pub fn new(program: &'a Program, errors: &'a mut Vec<TypecheckError>) -> PatternChecker<'a> {
        PatternChecker {
            program: program,
            errors: errors,
        }
    }
}

impl<'a> Visitor for PatternChecker<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, _: ExprId, expr: &Expr) {
        if let Expr::CaseOf(_, cases, _) = expr {
            let mut case_info = CaseInfo::new();
            let first = case_info.add_first();
            for case in cases {
                let mut choice_added = false;

                let id = case_info.check_pattern(
                    &case.pattern_id,
                    &self.program,
                    first,
                    &mut choice_added,
                );
                if id == 0 {
                    // guarded pattern, ignore it
                } else {
                    let location = self.program.patterns.get(&case.pattern_id).location_id;
                    if !choice_added {
                        //let err = TypecheckError::UnreachablePattern(location);
                        //self.errors.push(err);
                    }
                }
            }
            case_info.check_choices(first, &self.program, self.errors);
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}
