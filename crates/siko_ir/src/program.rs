use crate::class::Class;
use crate::class::ClassId;
use crate::class::ClassMember;
use crate::class::ClassMemberId;
use crate::class::Instance;
use crate::class::InstanceId;
use crate::data::Adt;
use crate::data::Record;
use crate::data::TypeDef;
use crate::data::TypeDefId;
use crate::data_type_info::AdtTypeInfo;
use crate::data_type_info::RecordTypeInfo;
use crate::expr::Expr;
use crate::expr::ExprId;
use crate::expr::FunctionArgumentRef;
use crate::function::Function;
use crate::function::FunctionId;
use crate::function::FunctionInfo;
use crate::function_dep_processor::FunctionDependencyProcessor;
use crate::instance_resolver::InstanceResolver;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::type_signature::TypeSignature;
use crate::type_signature::TypeSignatureId;
use crate::type_var_generator::TypeVarGenerator;
use crate::types::Type;
use crate::unifier::Unifier;
use siko_constants::BOOL_MODULE_NAME;
use siko_constants::BOOL_TYPE_NAME;
use siko_constants::CHAR_MODULE_NAME;
use siko_constants::CHAR_TYPE_NAME;
use siko_constants::EQ_CLASS_NAME;
use siko_constants::FLOAT_MODULE_NAME;
use siko_constants::FLOAT_TYPE_NAME;
use siko_constants::INT_MODULE_NAME;
use siko_constants::INT_TYPE_NAME;
use siko_constants::LIST_MODULE_NAME;
use siko_constants::LIST_TYPE_NAME;
use siko_constants::MAIN_FUNCTION;
use siko_constants::MAIN_MODULE_NAME;
use siko_constants::OPTION_MODULE_NAME;
use siko_constants::OPTION_TYPE_NAME;
use siko_constants::ORDERING_MODULE_NAME;
use siko_constants::ORDERING_TYPE_NAME;
use siko_constants::ORD_CLASS_NAME;
use siko_constants::ORD_OP_NAME;
use siko_constants::PARTIALEQ_CLASS_NAME;
use siko_constants::PARTIALEQ_OP_NAME;
use siko_constants::PARTIALORD_CLASS_NAME;
use siko_constants::SHOW_CLASS_NAME;
use siko_constants::STRING_MODULE_NAME;
use siko_constants::STRING_TYPE_NAME;
use siko_location_info::item::ItemInfo;
use siko_util::dependency_processor::DependencyGroup;
use siko_util::ItemContainer;
use std::collections::BTreeMap;

pub struct Program {
    pub type_signatures: ItemContainer<TypeSignatureId, ItemInfo<TypeSignature>>,
    pub exprs: ItemContainer<ExprId, ItemInfo<Expr>>,
    pub functions: ItemContainer<FunctionId, Function>,
    pub typedefs: ItemContainer<TypeDefId, TypeDef>,
    pub patterns: ItemContainer<PatternId, ItemInfo<Pattern>>,
    pub classes: ItemContainer<ClassId, Class>,
    pub class_members: ItemContainer<ClassMemberId, ClassMember>,
    pub instances: ItemContainer<InstanceId, Instance>,
    pub expr_types: BTreeMap<ExprId, Type>,
    pub pattern_types: BTreeMap<PatternId, Type>,
    pub function_types: BTreeMap<FunctionId, Type>,
    pub class_names: BTreeMap<String, ClassId>,
    pub class_member_types: BTreeMap<ClassMemberId, (Type, Type)>,
    pub named_types: BTreeMap<String, BTreeMap<String, TypeDefId>>,
    pub type_var_generator: TypeVarGenerator,
    pub function_dependency_groups: Vec<DependencyGroup<FunctionId>>,
    pub adt_type_info_map: BTreeMap<TypeDefId, AdtTypeInfo>,
    pub record_type_info_map: BTreeMap<TypeDefId, RecordTypeInfo>,
    pub instance_resolver: InstanceResolver,
}

impl Program {
    pub fn new(type_var_generator: TypeVarGenerator) -> Program {
        Program {
            type_signatures: ItemContainer::new(),
            exprs: ItemContainer::new(),
            functions: ItemContainer::new(),
            typedefs: ItemContainer::new(),
            patterns: ItemContainer::new(),
            classes: ItemContainer::new(),
            class_members: ItemContainer::new(),
            instances: ItemContainer::new(),
            expr_types: BTreeMap::new(),
            pattern_types: BTreeMap::new(),
            function_types: BTreeMap::new(),
            class_names: BTreeMap::new(),
            class_member_types: BTreeMap::new(),
            named_types: BTreeMap::new(),
            type_var_generator: type_var_generator.clone(),
            function_dependency_groups: Vec::new(),
            adt_type_info_map: BTreeMap::new(),
            record_type_info_map: BTreeMap::new(),
            instance_resolver: InstanceResolver::new(type_var_generator),
        }
    }

    pub fn get_list_type_id(&self) -> TypeDefId {
        let id = self.get_named_type(LIST_MODULE_NAME, LIST_TYPE_NAME);
        id
    }

    pub fn get_list_type(&self, ty: Type) -> Type {
        let id = self.get_list_type_id();
        Type::Named(LIST_TYPE_NAME.to_string(), id, vec![ty])
    }

    pub fn get_string_type(&self) -> Type {
        let id = self.get_named_type(STRING_MODULE_NAME, STRING_TYPE_NAME);
        Type::Named(STRING_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_bool_type(&self) -> Type {
        let id = self.get_named_type(BOOL_MODULE_NAME, BOOL_TYPE_NAME);
        Type::Named(BOOL_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_float_type(&self) -> Type {
        let id = self.get_named_type(FLOAT_MODULE_NAME, FLOAT_TYPE_NAME);
        Type::Named(FLOAT_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_char_type(&self) -> Type {
        let id = self.get_named_type(CHAR_MODULE_NAME, CHAR_TYPE_NAME);
        Type::Named(CHAR_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_int_type(&self) -> Type {
        let id = self.get_named_type(INT_MODULE_NAME, INT_TYPE_NAME);
        Type::Named(INT_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_ordering_type(&self) -> Type {
        let id = self.get_named_type(ORDERING_MODULE_NAME, ORDERING_TYPE_NAME);
        Type::Named(ORDERING_TYPE_NAME.to_string(), id, Vec::new())
    }

    pub fn get_option_type(&self, ty: Type) -> Type {
        let id = self.get_named_type(OPTION_MODULE_NAME, OPTION_TYPE_NAME);
        Type::Named(OPTION_TYPE_NAME.to_string(), id, vec![ty])
    }

    pub fn get_json_object_item_type(&self) -> Type {
        let id = self.get_named_type("Json", "JsonObjectItem");
        Type::Named("JsonObjectItem".to_string(), id, vec![])
    }

    pub fn get_json_type(&self) -> Type {
        let id = self.get_named_type("Json", "Json");
        Type::Named("Json".to_string(), id, vec![])
    }

    pub fn get_show_type(&self) -> Type {
        let class_id = self.get_show_class_id();
        let mut var = self.type_var_generator.clone();
        let index = var.get_new_index();
        Type::Var(index, vec![class_id])
    }

    pub fn get_show_class_id(&self) -> ClassId {
        let class_id = self
            .class_names
            .get(SHOW_CLASS_NAME)
            .expect("Show not found")
            .clone();
        class_id
    }

    pub fn get_partialeq_class_id(&self) -> ClassId {
        let class_id = self
            .class_names
            .get(PARTIALEQ_CLASS_NAME)
            .expect("PartialEq not found")
            .clone();
        class_id
    }

    pub fn get_partialord_class_id(&self) -> ClassId {
        let class_id = self
            .class_names
            .get(PARTIALORD_CLASS_NAME)
            .expect("PartialOrd not found")
            .clone();
        class_id
    }

    pub fn get_eq_class_id(&self) -> ClassId {
        let class_id = self
            .class_names
            .get(EQ_CLASS_NAME)
            .expect("Eq not found")
            .clone();
        class_id
    }

    pub fn get_ord_class_id(&self) -> ClassId {
        let class_id = self
            .class_names
            .get(ORD_CLASS_NAME)
            .expect("Ord not found")
            .clone();
        class_id
    }

    pub fn get_partialeq_op_id(&self) -> ClassMemberId {
        let class_id = self
            .class_names
            .get(PARTIALEQ_CLASS_NAME)
            .expect("PartialEq not found");
        let class = self.classes.get(class_id);
        class
            .members
            .get(PARTIALEQ_OP_NAME)
            .expect("PartialEq op not found")
            .clone()
    }

    pub fn get_ord_op_id(&self) -> ClassMemberId {
        let class_id = self.class_names.get(ORD_CLASS_NAME).expect("Ord not found");
        let class = self.classes.get(class_id);
        class
            .members
            .get(ORD_OP_NAME)
            .expect("Ord op not found")
            .clone()
    }

    pub fn get_adt_by_name(&self, module: &str, name: &str) -> &Adt {
        let id = self
            .named_types
            .get(module)
            .expect("Module not found")
            .get(name)
            .expect("Typedef not found");
        if let TypeDef::Adt(adt) = self.typedefs.get(id) {
            adt
        } else {
            panic!("{}/{} is not an adt", module, name)
        }
    }

    pub fn get_record_by_name(&self, module: &str, name: &str) -> &Record {
        let id = self
            .named_types
            .get(module)
            .expect("Module not found")
            .get(name)
            .expect("Typedef not found");
        if let TypeDef::Record(record) = self.typedefs.get(id) {
            record
        } else {
            panic!("{}/{} is not a record", module, name)
        }
    }

    pub fn get_constructor_by_name(&self, module: &str, name: &str, variant: &str) -> FunctionId {
        let adt = self.get_adt_by_name(module, name);
        let index = adt.get_variant_index(variant);
        adt.variants[index].constructor
    }

    pub fn get_named_type(&self, module: &str, name: &str) -> TypeDefId {
        self.named_types
            .get(module)
            .expect("Module not found")
            .get(name)
            .expect("Typedef not found")
            .clone()
    }

    pub fn get_module_and_name(&self, typedef_id: TypeDefId) -> (String, String) {
        let typedef = self.typedefs.get(&typedef_id);
        let (module, name) = match typedef {
            TypeDef::Adt(adt) => (adt.module.clone(), adt.name.clone()),
            TypeDef::Record(record) => (record.module.clone(), record.name.clone()),
        };
        (module, name)
    }

    pub fn get_unifier(&self) -> Unifier {
        Unifier::new(self.type_var_generator.clone())
    }

    pub fn get_main(&self) -> Option<FunctionId> {
        for (id, function) in &self.functions.items {
            match &function.info {
                FunctionInfo::NamedFunction(info) => {
                    if info.module == MAIN_MODULE_NAME && info.name == MAIN_FUNCTION {
                        return Some(*id);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn calculate_function_dependencies(&mut self) {
        let function_dep_processor = FunctionDependencyProcessor::new(self);

        self.function_dependency_groups = function_dep_processor.process_functions();
    }

    pub fn disambiguate_expr(&mut self, expr_id: ExprId, selected_index: usize) {
        let expr_info = self.exprs.get_mut(&expr_id);
        match expr_info.item.clone() {
            Expr::FieldAccess(infos, receiver) => {
                expr_info.item = Expr::FieldAccess(vec![infos[selected_index].clone()], receiver);
            }
            Expr::RecordUpdate(receiver, updates) => {
                expr_info.item =
                    Expr::RecordUpdate(receiver, vec![updates[selected_index].clone()]);
            }
            _ => {}
        }
    }

    pub fn get_expr_type(&self, expr_id: &ExprId) -> &Type {
        self.expr_types.get(expr_id).expect("Expr type not found")
    }

    pub fn get_pattern_type(&self, pattern_id: &PatternId) -> &Type {
        self.pattern_types
            .get(pattern_id)
            .expect("Pattern type not found")
    }

    pub fn get_function_type(&self, function_id: &FunctionId) -> &Type {
        self.function_types
            .get(function_id)
            .expect("Function type not found")
    }

    pub fn update_arg_ref(&mut self, expr_id: &ExprId, arg_ref: FunctionArgumentRef) {
        let expr = &mut self.exprs.get_mut(&expr_id).item;
        *expr = Expr::ArgRef(arg_ref);
    }

    pub fn get_show_member_id(&self) -> ClassMemberId {
        let class_id = self.get_show_class_id();
        let class = self.classes.get(&class_id);
        let show_id = class.members.get("show").expect("show not found").clone();
        show_id
    }

    pub fn get_opeq_member_id(&self) -> ClassMemberId {
        let class_id = self.get_partialeq_class_id();
        let class = self.classes.get(&class_id);
        let cmp_id = class.members.get("opEq").expect("opEq not found").clone();
        cmp_id
    }

    pub fn get_partialcmp_member_id(&self) -> ClassMemberId {
        let class_id = self.get_partialord_class_id();
        let class = self.classes.get(&class_id);
        let cmp_id = class
            .members
            .get("partialCmp")
            .expect("partialCmp not found")
            .clone();
        cmp_id
    }

    pub fn get_cmp_member_id(&self) -> ClassMemberId {
        let class_id = self.get_ord_class_id();
        let class = self.classes.get(&class_id);
        let cmp_id = class.members.get("cmp").expect("cmp not found").clone();
        cmp_id
    }
}
