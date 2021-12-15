use crate::data::TypeDef;
use crate::data::TypeDefId;
use crate::expr::Expr;
use crate::expr::ExprId;
use crate::function::Function;
use crate::function::FunctionId;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::types::Closure;
use crate::types::PartialFunctionCall;
use crate::types::PartialFunctionCallId;
use crate::types::Type;
use siko_location_info::item::ItemInfo;
use siko_location_info::location_id::LocationId;
use siko_util::ItemContainer;
use std::collections::BTreeMap;

pub struct Program {
    pub exprs: ItemContainer<ExprId, ItemInfo<Expr>>,
    pub expr_types: BTreeMap<ExprId, Type>,
    pub patterns: ItemContainer<PatternId, ItemInfo<Pattern>>,
    pub pattern_types: BTreeMap<PatternId, Type>,
    pub functions: ItemContainer<FunctionId, Function>,
    pub typedefs: ItemContainer<TypeDefId, TypeDef>,
    pub closures: BTreeMap<Type, Closure>,
    pub partial_function_calls: ItemContainer<PartialFunctionCallId, PartialFunctionCall>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            exprs: ItemContainer::new(),
            expr_types: BTreeMap::new(),
            patterns: ItemContainer::new(),
            pattern_types: BTreeMap::new(),
            functions: ItemContainer::new(),
            typedefs: ItemContainer::new(),
            closures: BTreeMap::new(),
            partial_function_calls: ItemContainer::new(),
        }
    }

    pub fn add_expr(&mut self, expr: Expr, location_id: LocationId, ty: Type) -> ExprId {
        let expr_info = ItemInfo {
            item: expr,
            location_id: location_id,
        };
        let expr_id = self.exprs.get_id();
        self.exprs.add_item(expr_id, expr_info);
        self.expr_types.insert(expr_id, ty);
        expr_id
    }

    pub fn add_pattern(
        &mut self,
        pattern: Pattern,
        location_id: LocationId,
        ty: Type,
    ) -> PatternId {
        let pattern_info = ItemInfo {
            item: pattern,
            location_id: location_id,
        };
        let pattern_id = self.patterns.get_id();
        self.patterns.add_item(pattern_id, pattern_info);
        self.pattern_types.insert(pattern_id, ty);
        pattern_id
    }

    pub fn get_expr_type(&self, expr_id: &ExprId) -> &Type {
        self.expr_types.get(expr_id).expect("Expr type not found")
    }

    pub fn get_pattern_type(&self, pattern_id: &PatternId) -> &Type {
        self.pattern_types
            .get(pattern_id)
            .expect("Pattern type not found")
    }

    pub fn add_closure_type(&mut self, ty: &Type) -> Type {
        if let Type::Function(from, to) = ty {
            let from = if from.is_function() {
                self.add_closure_type(from)
            } else {
                *from.clone()
            };
            let to = if to.is_function() {
                self.add_closure_type(to)
            } else {
                *to.clone()
            };
            let ty = Type::Function(Box::new(from.clone()), Box::new(to.clone()));
            let index = self.closures.len();
            self.closures.entry(ty.clone()).or_insert_with(|| Closure {
                name: format!("Closure{}", index),
                ty: ty.clone(),
                from_ty: from,
                to_ty: to,
            });
            Type::Closure(Box::new(ty.clone()))
        } else {
            panic!("{} is not a closure", ty.to_string(self))
        }
    }

    pub fn get_closure_type(&self, ty: &Type) -> &Closure {
        match ty {
            Type::Function(..) => match self.closures.get(ty) {
                Some(c) => c,
                None => panic!("Closure type {} not found", ty.to_string(self)),
            },
            Type::Closure(ty) => self.get_closure_type(ty),
            _ => panic!("Closure type {} not found", ty.to_string(self)),
        }
    }

    pub fn update_expr(&mut self, expr_id: ExprId, expr: Expr) {
        self.exprs.get_mut(&expr_id).item = expr;
    }
}
