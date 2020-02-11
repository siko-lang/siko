use crate::char;
use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::float;
use crate::int;
use crate::iterator;
use crate::list;
use crate::map;
use crate::std_ops;
use crate::std_util;
use crate::std_util_basic;
use crate::string;
use crate::util::get_opt_ordering_value;
use crate::util::get_ordering_value;
use crate::value::BuiltinCallable;
use crate::value::Callable;
use crate::value::CallableKind;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::OPTION_MODULE_NAME;
use siko_constants::OPTION_TYPE_NAME;
use siko_constants::ORDERING_MODULE_NAME;
use siko_constants::ORDERING_TYPE_NAME;
use siko_ir::class::ClassMember;
use siko_ir::class::ClassMemberId;
use siko_ir::data::Adt;
use siko_ir::data::TypeDefId;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::function::FunctionId;
use siko_ir::function::FunctionInfo;
use siko_ir::function::NamedFunctionKind;
use siko_ir::instance_resolver::ResolutionResult;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use siko_location_info::error_context::ErrorContext;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::thread_local;

thread_local! {
    static INTERPRETER_CONTEXT: RefCell<Option<Interpreter>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct VariantCache {
    pub variants: BTreeMap<String, usize>,
}

impl VariantCache {
    pub fn new(adt: &Adt) -> VariantCache {
        let mut variants = BTreeMap::new();
        for (index, variant) in adt.variants.iter().enumerate() {
            variants.insert(variant.name.clone(), index);
        }
        VariantCache { variants: variants }
    }

    pub fn get_index(&self, name: &str) -> usize {
        self.variants.get(name).expect("Variant not found").clone()
    }
}

#[derive(Clone)]
pub struct TypeDefIdCache {
    pub option_id: TypeDefId,
    pub ordering_id: TypeDefId,
    pub option_variants: VariantCache,
    pub ordering_variants: VariantCache,
}

pub struct Interpreter {
    program: Program,
    error_context: ErrorContext,
    typedefid_cache: Option<TypeDefIdCache>,
    extern_functions: BTreeMap<(String, String), Box<dyn ExternFunction>>,
}

impl Interpreter {
    fn new(program: Program, error_context: ErrorContext) -> Interpreter {
        Interpreter {
            program: program,
            error_context: error_context,
            typedefid_cache: None,
            extern_functions: BTreeMap::new(),
        }
    }

    fn call(&self, callable_value: Value, args: Vec<Value>, expr_id: Option<ExprId>) -> Value {
        match callable_value.core {
            ValueCore::Callable(mut callable) => {
                let mut callable_func_ty = callable_value.ty;
                callable.values.extend(args);
                loop {
                    let needed_arg_count = match &callable.kind {
                        CallableKind::Builtin(builtin) => match builtin {
                            BuiltinCallable::Show => 1,
                            BuiltinCallable::PartialEq => 2,
                            BuiltinCallable::PartialOrd => 2,
                            BuiltinCallable::Ord => 2,
                        },
                        CallableKind::FunctionId(function_id) => {
                            let func = self.program.functions.get(function_id);
                            func.arg_count
                        }
                    };
                    if needed_arg_count > callable.values.len() {
                        callable_func_ty = callable_func_ty.get_result_type(callable.values.len());
                        return Value::new(ValueCore::Callable(callable), callable_func_ty);
                    } else {
                        let rest = callable.values.split_off(needed_arg_count);
                        let mut call_args = Vec::new();
                        std::mem::swap(&mut call_args, &mut callable.values);
                        let arg_count = call_args.len();
                        let mut environment = Environment::new(callable.kind, call_args);
                        callable_func_ty = callable_func_ty.get_result_type(arg_count);
                        let result = match &callable.kind {
                            CallableKind::Builtin(builtin) => self.execute_builtin(
                                builtin,
                                &mut environment,
                                expr_id,
                                &callable.unifier,
                                callable_func_ty.clone(),
                            ),
                            CallableKind::FunctionId(id) => self.execute(
                                *id,
                                &mut environment,
                                expr_id,
                                &callable.unifier,
                                callable_func_ty.clone(),
                            ),
                        };
                        if !rest.is_empty() {
                            if let ValueCore::Callable(new_callable) = result.core {
                                callable = new_callable;
                                callable_func_ty = result.ty;
                                callable.values.extend(rest);
                            } else {
                                unreachable!()
                            }
                        } else {
                            return result;
                        }
                    }
                }
            }
            _ => panic!("Cannot call {:?}", callable_value),
        }
    }

    fn match_pattern(
        &self,
        pattern_id: &PatternId,
        value: &Value,
        environment: &mut Environment,
        unifier: &Unifier,
    ) -> bool {
        let pattern = &self.program.patterns.get(pattern_id).item;
        match pattern {
            Pattern::Binding(_) => {
                environment.add(*pattern_id, value.clone());
                return true;
            }
            Pattern::Tuple(ids) => match &value.core {
                ValueCore::Tuple(vs) => {
                    for (index, id) in ids.iter().enumerate() {
                        let v = &vs[index];
                        if !self.match_pattern(id, v, environment, unifier) {
                            return false;
                        }
                    }
                    return true;
                }
                _ => {
                    return false;
                }
            },
            Pattern::Record(p_type_id, p_ids) => match &value.core {
                ValueCore::Record(type_id, vs) => {
                    if type_id == p_type_id {
                        for (index, p_id) in p_ids.iter().enumerate() {
                            let v = &vs[index];
                            if !self.match_pattern(p_id, v, environment, unifier) {
                                return false;
                            }
                        }
                        return true;
                    }
                    return false;
                }
                _ => {
                    return false;
                }
            },
            Pattern::Variant(p_type_id, p_index, p_ids) => match &value.core {
                ValueCore::Variant(type_id, index, vs) => {
                    if type_id == p_type_id && index == p_index {
                        for (index, p_id) in p_ids.iter().enumerate() {
                            let v = &vs[index];
                            if !self.match_pattern(p_id, v, environment, unifier) {
                                return false;
                            }
                        }
                        return true;
                    }
                    return false;
                }
                _ => {
                    return false;
                }
            },
            Pattern::Guarded(id, guard_expr_id) => {
                if self.match_pattern(id, value, environment, unifier) {
                    let guard_value = self.eval_expr(*guard_expr_id, environment, unifier);
                    return guard_value.core.as_bool();
                } else {
                    return false;
                }
            }
            Pattern::Typed(id, _) => self.match_pattern(id, value, environment, unifier),
            Pattern::Wildcard => {
                return true;
            }
            Pattern::IntegerLiteral(p_v) => {
                let r = match &value.core {
                    ValueCore::Int(v) => p_v == v,
                    _ => false,
                };
                return r;
            }
            Pattern::CharLiteral(p_v) => {
                let r = match &value.core {
                    ValueCore::Char(v) => p_v == v,
                    _ => false,
                };
                return r;
            }
            Pattern::CharRange(start, end) => {
                let r = match &value.core {
                    ValueCore::Char(v) => {
                        let range = std::ops::Range { start, end };
                        range.contains(&v)
                    }
                    _ => false,
                };
                return r;
            }
            Pattern::StringLiteral(p_v) => {
                let r = match &value.core {
                    ValueCore::String(v) => p_v == v,
                    _ => false,
                };
                return r;
            }
        }
    }

    pub fn call_show(arg: Value) -> String {
        let string_ty = Interpreter::get_string_type();
        let v = Interpreter::call_specific_class_member(vec![arg], "Show", "show", string_ty);
        v.core.as_string()
    }

    pub fn get_string_type() -> Type {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let string_ty = i.program.get_string_type();
            string_ty
        })
    }

    pub fn get_bool_type() -> Type {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let bool_ty = i.program.get_bool_type();
            bool_ty
        })
    }

    pub fn get_char_type() -> Type {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let bool_ty = i.program.get_char_type();
            bool_ty
        })
    }

    pub fn get_bool_value(v: bool) -> Value {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let bool_ty = i.program.get_bool_type();
            if let Type::Named(_, id, _) = &bool_ty {
                Value::new(
                    ValueCore::Variant(*id, if v == true { 0 } else { 1 }, vec![]),
                    bool_ty,
                )
            } else {
                unreachable!()
            }
        })
    }

    pub fn get_optional_ordering_type() -> Type {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let option_ordering_ty = i.program.get_option_type(i.program.get_ordering_type());
            option_ordering_ty
        })
    }

    pub fn get_ordering_type() -> Type {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            i.program.get_ordering_type()
        })
    }

    pub fn call_specific_class_member(
        args: Vec<Value>,
        class_name: &str,
        member_name: &str,
        expr_ty: Type,
    ) -> Value {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let class_id = i
                .program
                .class_names
                .get(class_name)
                .expect("Show not found");
            let class = i.program.classes.get(class_id);
            let class_member_id = class.members.get(member_name).expect("show not found");
            let v = i.call_class_member(class_member_id, args, None, expr_ty);
            v
        })
    }

    pub fn call_func(callable: Value, args: Vec<Value>, expr_id: Option<ExprId>) -> Value {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            return i.call(callable, args, expr_id);
        })
    }

    pub fn call_abort(current_expr: ExprId) {
        INTERPRETER_CONTEXT.with(|i| {
            let b = i.borrow();
            let i = b.as_ref().expect("Interpreter not set");
            let location_id = i.program.exprs.get(&current_expr).location_id;
            i.error_context
                .report_error(format!("Assertion failed"), location_id);
            panic!("Abort not implemented");
        })
    }

    pub fn call_op_eq(arg1: Value, arg2: Value) -> Value {
        let bool_ty = Interpreter::get_bool_type();
        Interpreter::call_specific_class_member(vec![arg1, arg2], "PartialEq", "opEq", bool_ty)
    }

    pub fn call_op_partial_cmp(arg1: Value, arg2: Value) -> Value {
        let option_ordering_ty = Interpreter::get_optional_ordering_type();
        Interpreter::call_specific_class_member(
            vec![arg1, arg2],
            "PartialOrd",
            "partialCmp",
            option_ordering_ty,
        )
    }

    pub fn call_op_partial_eq(arg1: Value, arg2: Value) -> Value {
        Interpreter::call_specific_class_member(
            vec![arg1, arg2],
            "PartialEq",
            "opEq",
            Interpreter::get_bool_type(),
        )
    }

    pub fn call_op_cmp(arg1: Value, arg2: Value) -> Value {
        let ordering_ty = Interpreter::get_ordering_type();
        Interpreter::call_specific_class_member(vec![arg1, arg2], "Ord", "cmp", ordering_ty)
    }

    fn get_call_unifier(
        &self,
        arg_values: &Vec<Value>,
        func_ty: &Type,
        expected_result_ty: &Type,
    ) -> Unifier {
        /*
        println!("Func type {}", func_ty);
        let mut arg_types = Vec::new();
        func_ty.get_args(&mut arg_types);
        for (index, arg) in arg_values.iter().enumerate() {
            println!("{}.arg {}", index, arg.ty);
        }
        for (index, arg) in arg_types.iter().enumerate() {
            println!("{}.argty {}", index, arg);
        }*/
        let mut call_unifier = self.program.get_unifier();
        let mut func_ty = func_ty.clone();
        for arg in arg_values {
            let mut arg_types = Vec::new();
            func_ty.get_args(&mut arg_types);
            let r = call_unifier.unify(&arg.ty, &arg_types[0]);
            assert!(r.is_ok());
            func_ty.apply(&call_unifier);
            func_ty = func_ty.get_result_type(1);
        }
        //println!("{} {}", expected_result_ty, func_ty);
        let r = call_unifier.unify(&func_ty, expected_result_ty);
        assert!(r.is_ok());
        call_unifier
    }

    fn check_member(
        class_member: &ClassMember,
        name: &str,
        builtin: BuiltinCallable,
    ) -> Option<CallableKind> {
        if class_member.name == name {
            Some(CallableKind::Builtin(builtin))
        } else {
            None
        }
    }

    fn call_class_member(
        &self,
        class_member_id: &ClassMemberId,
        arg_values: Vec<Value>,
        expr_id: Option<ExprId>,
        expr_ty: Type,
    ) -> Value {
        for arg in &arg_values {
            assert!(arg.ty.is_concrete_type());
        }
        let member = self.program.class_members.get(class_member_id);
        let (class_member_type, class_arg_ty) = self
            .program
            .class_member_types
            .get(class_member_id)
            .expect("untyped class member");
        let call_unifier = self.get_call_unifier(
            &arg_values,
            &class_member_type.remove_fixed_types(),
            &expr_ty,
        );
        let function_type = call_unifier.apply(&class_member_type);
        let class_arg = call_unifier.apply(&class_arg_ty.remove_fixed_types());
        let class = self.program.classes.get(&member.class_id);
        assert!(class_arg.is_concrete_type());
        match self
            .program
            .instance_resolver
            .get(member.class_id, class_arg)
        {
            ResolutionResult::AutoDerived => {
                let kind = match (class.module.as_ref(), class.name.as_ref()) {
                    ("Std.Ops", "Show") => Some(CallableKind::Builtin(BuiltinCallable::Show)),
                    ("Std.Ops", "PartialEq") => {
                        Interpreter::check_member(member, "opEq", BuiltinCallable::PartialEq)
                    }
                    ("Std.Ops", "PartialOrd") => {
                        Interpreter::check_member(member, "partialCmp", BuiltinCallable::PartialOrd)
                    }
                    ("Std.Ops", "Ord") => {
                        Interpreter::check_member(member, "cmp", BuiltinCallable::Ord)
                    }
                    _ => panic!(
                        "Auto derive of {}/{} is not implemented",
                        class.module, class.name
                    ),
                };
                if let Some(kind) = kind {
                    let callable = Value::new(
                        ValueCore::Callable(Callable {
                            kind: kind,
                            values: vec![],
                            unifier: call_unifier,
                        }),
                        function_type,
                    );
                    return self.call(callable, arg_values, expr_id);
                } else {
                    let member_function_id = member
                        .default_implementation
                        .expect("Default implementation not found");
                    let callable = Value::new(
                        ValueCore::Callable(Callable {
                            kind: CallableKind::FunctionId(member_function_id),
                            values: vec![],
                            unifier: call_unifier,
                        }),
                        function_type,
                    );
                    return self.call(callable, arg_values, expr_id);
                }
            }
            ResolutionResult::UserDefined(instance_id) => {
                let instance = self.program.instances.get(&instance_id);
                let member_function_id =
                    if let Some(instance_member) = instance.members.get(&member.name) {
                        instance_member.function_id
                    } else {
                        member
                            .default_implementation
                            .expect("Default implementation not found")
                    };
                let callable = Value::new(
                    ValueCore::Callable(Callable {
                        kind: CallableKind::FunctionId(member_function_id),
                        values: vec![],
                        unifier: call_unifier,
                    }),
                    function_type,
                );
                return self.call(callable, arg_values, expr_id);
            }
        }
    }

    fn eval_expr(
        &self,
        expr_id: ExprId,
        environment: &mut Environment,
        unifier: &Unifier,
    ) -> Value {
        let expr = &self.program.exprs.get(&expr_id).item;
        //println!("Eval {} {}", expr_id, expr);
        let expr_ty = self.program.get_expr_type(&expr_id).clone();
        let expr_ty = unifier.apply(&expr_ty);
        match expr {
            Expr::IntegerLiteral(v) => Value::new(ValueCore::Int(*v), expr_ty),
            Expr::StringLiteral(v) => Value::new(ValueCore::String(v.clone()), expr_ty),
            Expr::FloatLiteral(v) => Value::new(ValueCore::Float(*v), expr_ty),
            Expr::CharLiteral(v) => Value::new(ValueCore::Char(*v), expr_ty),
            Expr::ArgRef(arg_ref) => {
                return environment.get_arg(arg_ref);
            }
            Expr::StaticFunctionCall(function_id, args) => {
                let func_ty = self
                    .program
                    .get_function_type(function_id)
                    .remove_fixed_types();
                let arg_values: Vec<_> = args
                    .iter()
                    .map(|arg| self.eval_expr(*arg, environment, unifier))
                    .collect();
                for arg in &arg_values {
                    assert!(arg.ty.is_concrete_type());
                    //println!("ARG {}", arg.ty.get_resolved_type_string(&self.program));
                }
                /*
                let f = self.program.functions.get(function_id);
                println!("Calling {}", f.info);
                */
                let call_unifier = self.get_call_unifier(&arg_values, &func_ty, &expr_ty);
                let function_type = call_unifier.apply(&func_ty);
                /*
                println!(
                    "Function type {}",
                    function_type.get_resolved_type_string(&self.program)
                );
                */
                let callable = Value::new(
                    ValueCore::Callable(Callable {
                        kind: CallableKind::FunctionId(*function_id),
                        values: vec![],
                        unifier: call_unifier,
                    }),
                    function_type,
                );
                return self.call(callable, arg_values, Some(expr_id));
            }
            Expr::DynamicFunctionCall(function_expr_id, args) => {
                let function_expr_id = self.eval_expr(*function_expr_id, environment, unifier);
                let arg_values: Vec<_> = args
                    .iter()
                    .map(|arg| self.eval_expr(*arg, environment, unifier))
                    .collect();
                return self.call(function_expr_id, arg_values, Some(expr_id));
            }
            Expr::Do(exprs) => {
                let mut environment = Environment::block_child(environment);
                let mut result = Value::new(ValueCore::Tuple(vec![]), expr_ty);
                assert!(!exprs.is_empty());
                for expr in exprs {
                    result = self.eval_expr(*expr, &mut environment, unifier);
                }
                return result;
            }
            Expr::Bind(pattern_id, expr_id) => {
                let value = self.eval_expr(*expr_id, environment, unifier);
                let r = self.match_pattern(pattern_id, &value, environment, unifier);
                assert!(r);
                return Value::new(ValueCore::Tuple(vec![]), expr_ty);
            }
            Expr::ExprValue(_, pattern_id) => {
                return environment.get_value(pattern_id);
            }
            Expr::If(cond, true_branch, false_branch) => {
                let cond_value = self.eval_expr(*cond, environment, unifier);
                if cond_value.core.as_bool() {
                    return self.eval_expr(*true_branch, environment, unifier);
                } else {
                    return self.eval_expr(*false_branch, environment, unifier);
                }
            }
            Expr::Tuple(exprs) => {
                let values: Vec<_> = exprs
                    .iter()
                    .map(|e| self.eval_expr(*e, environment, unifier))
                    .collect();
                return Value::new(ValueCore::Tuple(values), expr_ty);
            }
            Expr::List(exprs) => {
                let values: Vec<_> = exprs
                    .iter()
                    .map(|e| self.eval_expr(*e, environment, unifier))
                    .collect();
                return Value::new(ValueCore::List(values), expr_ty);
            }
            Expr::TupleFieldAccess(index, tuple) => {
                let tuple_value = self.eval_expr(*tuple, environment, unifier);
                if let ValueCore::Tuple(t) = &tuple_value.core {
                    return t[*index].clone();
                } else {
                    unreachable!()
                }
            }
            Expr::Formatter(fmt, args) => {
                let subs: Vec<_> = fmt.split("{}").collect();
                let values: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(*e, environment, unifier))
                    .collect();
                let mut result = String::new();
                for (index, sub) in subs.iter().enumerate() {
                    result += sub;
                    if values.len() > index {
                        let value_as_string = Interpreter::call_show(values[index].clone());
                        result += &value_as_string;
                    }
                }
                return Value::new(ValueCore::String(result), expr_ty);
            }
            Expr::FieldAccess(infos, record_expr) => {
                let record = self.eval_expr(*record_expr, environment, unifier);
                let (id, values) = if let ValueCore::Record(id, values) = &record.core {
                    (id, values)
                } else {
                    unreachable!()
                };
                for info in infos {
                    if info.record_id != *id {
                        continue;
                    }
                    return values[info.index].clone();
                }
                unreachable!()
            }
            Expr::CaseOf(body, cases, _) => {
                let case_value = self.eval_expr(*body, environment, unifier);
                for case in cases {
                    let mut case_env = Environment::block_child(environment);
                    if self.match_pattern(&case.pattern_id, &case_value, &mut case_env, unifier) {
                        let val = self.eval_expr(case.body, &mut case_env, unifier);
                        return val;
                    }
                }
                unreachable!()
            }
            Expr::RecordInitialization(type_id, items) => {
                let mut values: Vec<_> = Vec::with_capacity(items.len());
                for _ in 0..items.len() {
                    values.push(Value::new(ValueCore::Tuple(vec![]), expr_ty.clone()));
                    // dummy value
                }
                for item in items {
                    let value = self.eval_expr(item.expr_id, environment, unifier);
                    values[item.index] = value;
                }
                return Value::new(ValueCore::Record(*type_id, values), expr_ty);
            }
            Expr::RecordUpdate(record_expr_id, updates) => {
                let value = self.eval_expr(*record_expr_id, environment, unifier);
                if let ValueCore::Record(id, mut values) = value.core {
                    for update in updates {
                        if id == update.record_id {
                            for item in &update.items {
                                let value = self.eval_expr(item.expr_id, environment, unifier);
                                values[item.index] = value;
                            }
                            return Value::new(ValueCore::Record(id, values), expr_ty);
                        }
                    }
                }
                unreachable!()
            }
            Expr::ClassFunctionCall(class_member_id, args) => {
                let arg_values: Vec<_> = args
                    .iter()
                    .map(|e| self.eval_expr(*e, environment, unifier))
                    .collect();
                return self.call_class_member(class_member_id, arg_values, Some(expr_id), expr_ty);
            }
        }
    }

    fn call_extern(
        &self,
        module: &str,
        name: &str,
        environment: &mut Environment,
        current_expr: Option<ExprId>,
        kind: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        if let Some(f) = self
            .extern_functions
            .get(&(module.to_string(), name.to_string()))
        {
            return f.call(environment, current_expr, kind, ty);
        } else {
            panic!("Unimplemented extern function {} {}", module, name);
        }
    }

    fn execute_builtin(
        &self,
        builtin: &BuiltinCallable,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &Unifier,
        _: Type,
    ) -> Value {
        match builtin {
            BuiltinCallable::Show => {
                let v = environment.get_arg_by_index(0);
                return Value::new(
                    ValueCore::String(v.core.show(&self.program)),
                    self.program.get_string_type(),
                );
            }
            BuiltinCallable::PartialEq => {
                let lhs = environment.get_arg_by_index(0);
                let rhs = environment.get_arg_by_index(1);
                if let ValueCore::Variant(id1, index1, items1) = &lhs.core {
                    if let ValueCore::Variant(id2, index2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        if index1 != index2 {
                            return Interpreter::get_bool_value(false);
                        } else {
                            for (item1, item2) in items1.iter().zip(items2.iter()) {
                                let value =
                                    Interpreter::call_op_partial_eq(item1.clone(), item2.clone());
                                let v = value.core.as_bool();
                                if !v {
                                    return Interpreter::get_bool_value(false);
                                }
                            }
                            return Interpreter::get_bool_value(true);
                        }
                    }
                }
                if let ValueCore::Record(id1, items1) = &lhs.core {
                    if let ValueCore::Record(id2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        for (item1, item2) in items1.iter().zip(items2.iter()) {
                            let value =
                                Interpreter::call_op_partial_eq(item1.clone(), item2.clone());
                            let v = value.core.as_bool();
                            if !v {
                                return Interpreter::get_bool_value(false);
                            }
                        }
                        return Interpreter::get_bool_value(true);
                    }
                }
                unimplemented!()
            }
            BuiltinCallable::PartialOrd => {
                let lhs = environment.get_arg_by_index(0);
                let rhs = environment.get_arg_by_index(1);
                if let ValueCore::Variant(id1, index1, items1) = &lhs.core {
                    if let ValueCore::Variant(id2, index2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        if index1 < index2 {
                            return get_opt_ordering_value(Some(Ordering::Less));
                        } else if index1 == index2 {
                            for (item1, item2) in items1.iter().zip(items2.iter()) {
                                let value =
                                    Interpreter::call_op_partial_cmp(item1.clone(), item2.clone());
                                let some_index = self
                                    .program
                                    .get_adt_by_name(OPTION_MODULE_NAME, OPTION_TYPE_NAME)
                                    .get_variant_index("Some");
                                let equal_index = self
                                    .program
                                    .get_adt_by_name(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME)
                                    .get_variant_index("Equal");
                                if let ValueCore::Variant(_, index, items) = &value.core {
                                    if *index == some_index {
                                        let ordering_value = &items[0];
                                        if let ValueCore::Variant(_, index, _) =
                                            &ordering_value.core
                                        {
                                            if *index == equal_index {
                                                continue;
                                            }
                                        }
                                    }
                                }
                                return value;
                            }
                            return get_opt_ordering_value(Some(Ordering::Equal));
                        } else {
                            return get_opt_ordering_value(Some(Ordering::Greater));
                        }
                    }
                }
                if let ValueCore::Record(id1, items1) = &lhs.core {
                    if let ValueCore::Record(id2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        for (item1, item2) in items1.iter().zip(items2.iter()) {
                            let value =
                                Interpreter::call_op_partial_cmp(item1.clone(), item2.clone());
                            let some_index = self
                                .program
                                .get_adt_by_name(OPTION_MODULE_NAME, OPTION_TYPE_NAME)
                                .get_variant_index("Some");
                            let equal_index = self
                                .program
                                .get_adt_by_name(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME)
                                .get_variant_index("Equal");
                            if let ValueCore::Variant(_, index, items) = &value.core {
                                if *index == some_index {
                                    let ordering_value = &items[0];
                                    if let ValueCore::Variant(_, index, _) = &ordering_value.core {
                                        if *index == equal_index {
                                            continue;
                                        }
                                    }
                                }
                            }
                            return value;
                        }
                        return get_opt_ordering_value(Some(Ordering::Equal));
                    }
                }
                unimplemented!()
            }
            BuiltinCallable::Ord => {
                let lhs = environment.get_arg_by_index(0);
                let rhs = environment.get_arg_by_index(1);
                if let ValueCore::Variant(id1, index1, items1) = &lhs.core {
                    if let ValueCore::Variant(id2, index2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        if index1 < index2 {
                            return get_ordering_value(Ordering::Less);
                        } else if index1 == index2 {
                            for (item1, item2) in items1.iter().zip(items2.iter()) {
                                let value = Interpreter::call_op_cmp(item1.clone(), item2.clone());
                                let equal_index = self
                                    .program
                                    .get_adt_by_name(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME)
                                    .get_variant_index("Equal");
                                if let ValueCore::Variant(_, index, _) = &value.core {
                                    if *index == equal_index {
                                        continue;
                                    }
                                }
                                return value;
                            }
                            return get_ordering_value(Ordering::Equal);
                        } else {
                            return get_ordering_value(Ordering::Greater);
                        }
                    }
                }
                if let ValueCore::Record(id1, items1) = &lhs.core {
                    if let ValueCore::Record(id2, items2) = &rhs.core {
                        assert_eq!(id1, id2);
                        for (item1, item2) in items1.iter().zip(items2.iter()) {
                            let value = Interpreter::call_op_cmp(item1.clone(), item2.clone());
                            let equal_index = self
                                .program
                                .get_adt_by_name(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME)
                                .get_variant_index("Equal");
                            if let ValueCore::Variant(_, index, _) = &value.core {
                                if *index == equal_index {
                                    continue;
                                }
                            }
                            return value;
                        }
                        return get_ordering_value(Ordering::Equal);
                    }
                }
                unimplemented!()
            }
        }
    }

    fn execute(
        &self,
        id: FunctionId,
        environment: &mut Environment,
        current_expr: Option<ExprId>,
        unifier: &Unifier,
        expr_ty: Type,
    ) -> Value {
        assert!(expr_ty.is_concrete_type());
        let function = self.program.functions.get(&id);
        match &function.info {
            FunctionInfo::NamedFunction(info) => match info.body {
                Some(body) => {
                    return self.eval_expr(body, environment, unifier);
                }
                None => {
                    return self.call_extern(
                        &info.module,
                        &info.name,
                        environment,
                        current_expr,
                        &info.kind,
                        expr_ty,
                    );
                }
            },
            FunctionInfo::Lambda(info) => {
                return self.eval_expr(info.body, environment, unifier);
            }
            FunctionInfo::VariantConstructor(info) => {
                let adt = self.program.typedefs.get(&info.type_id).get_adt();
                let variant = &adt.variants[info.index];
                let mut values = Vec::new();
                for index in 0..variant.items.len() {
                    let v = environment.get_arg_by_index(index);
                    values.push(v);
                }
                return Value::new(
                    ValueCore::Variant(info.type_id, info.index, values),
                    expr_ty,
                );
            }
            FunctionInfo::RecordConstructor(info) => {
                let record = self.program.typedefs.get(&info.type_id).get_record();
                let mut values = Vec::new();
                for index in 0..record.fields.len() {
                    let v = environment.get_arg_by_index(index);
                    values.push(v);
                }
                return Value::new(ValueCore::Record(info.type_id, values), expr_ty);
            }
        }
    }

    fn build_typedefid_cache(&mut self) {
        let option = self
            .program
            .get_adt_by_name(OPTION_MODULE_NAME, OPTION_TYPE_NAME);
        let ordering = self
            .program
            .get_adt_by_name(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME);
        let cache = TypeDefIdCache {
            option_id: option.id,
            ordering_id: ordering.id,
            option_variants: VariantCache::new(option),
            ordering_variants: VariantCache::new(ordering),
        };
        self.typedefid_cache = Some(cache);
    }

    pub fn get_typedef_id_cache() -> TypeDefIdCache {
        INTERPRETER_CONTEXT.with(|i| {
            let i = i.borrow();
            i.as_ref()
                .expect("Interpreter not set")
                .typedefid_cache
                .clone()
                .expect("TypedefId cache not set")
        })
    }

    fn execute_main(interpreter: &Interpreter) -> Value {
        let main_id = interpreter.program.get_main().expect("Main does not exist");
        let mut environment = Environment::new(CallableKind::FunctionId(main_id), vec![]);
        let unifier = interpreter.program.get_unifier();
        return interpreter.execute(
            main_id,
            &mut environment,
            None,
            &unifier,
            Type::Tuple(vec![]),
        );
    }

    pub fn add_extern_function(
        &mut self,
        module: &str,
        name: &str,
        extern_function: Box<dyn ExternFunction>,
    ) {
        self.extern_functions
            .insert((module.to_string(), name.to_string()), extern_function);
    }

    pub fn run(program: Program, error_context: ErrorContext) -> Value {
        let mut interpreter = Interpreter::new(program, error_context);
        int::register_extern_functions(&mut interpreter);
        char::register_extern_functions(&mut interpreter);
        float::register_extern_functions(&mut interpreter);
        string::register_extern_functions(&mut interpreter);
        map::register_extern_functions(&mut interpreter);
        list::register_extern_functions(&mut interpreter);
        std_util_basic::register_extern_functions(&mut interpreter);
        std_util::register_extern_functions(&mut interpreter);
        std_ops::register_extern_functions(&mut interpreter);
        iterator::register_extern_functions(&mut interpreter);
        interpreter.build_typedefid_cache();
        INTERPRETER_CONTEXT.with(|c| {
            let mut p = c.borrow_mut();
            *p = Some(interpreter);
        });
        INTERPRETER_CONTEXT.with(|c| {
            let p = c.borrow();
            let i = p.as_ref().expect("Interpreter not set");
            Interpreter::execute_main(i)
        })
    }
}
