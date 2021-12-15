use std::collections::BTreeMap;

use crate::actor::Actor;
use crate::actor::ActorId;
use crate::actor::Protocol;
use crate::actor::ProtocolId;
use crate::class::Class;
use crate::class::ClassId;
use crate::class::Constraint;
use crate::class::Instance;
use crate::class::InstanceId;
use crate::data::Adt;
use crate::data::AdtId;
use crate::data::Record;
use crate::data::RecordField;
use crate::data::RecordFieldId;
use crate::data::RecordId;
use crate::data::Variant;
use crate::data::VariantId;
use crate::expr::Case;
use crate::expr::Expr;
use crate::expr::ExprId;
use crate::function::Function;
use crate::function::FunctionBody;
use crate::function::FunctionId;
use crate::function::FunctionType;
use crate::function::FunctionTypeId;
use crate::import::Import;
use crate::import::ImportId;
use crate::module::Module;
use crate::module::ModuleId;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::types::TypeSignature;
use crate::types::TypeSignatureId;
use siko_constants::FROMJSON_CLASS_NAME;
use siko_constants::TOJSON_CLASS_NAME;
use siko_location_info::item::ItemInfo;
use siko_location_info::location_id::LocationId;
use siko_util::ItemContainer;

#[derive(Debug)]
pub struct Program {
    pub modules: ItemContainer<ModuleId, Module>,
    pub functions: ItemContainer<FunctionId, Function>,
    pub function_types: ItemContainer<FunctionTypeId, FunctionType>,
    pub records: ItemContainer<RecordId, Record>,
    pub adts: ItemContainer<AdtId, Adt>,
    pub variants: ItemContainer<VariantId, Variant>,
    pub classes: ItemContainer<ClassId, Class>,
    pub instances: ItemContainer<InstanceId, Instance>,
    pub exprs: ItemContainer<ExprId, ItemInfo<Expr>>,
    pub type_signatures: ItemContainer<TypeSignatureId, ItemInfo<TypeSignature>>,
    pub patterns: ItemContainer<PatternId, ItemInfo<Pattern>>,
    pub imports: ItemContainer<ImportId, Import>,
    pub record_fields: ItemContainer<RecordFieldId, RecordField>,
    pub protocols: ItemContainer<ProtocolId, Protocol>,
    pub actors: ItemContainer<ActorId, Actor>,
}

struct RecordInstanceInfo {
    index: usize,
    location_id: LocationId,
    record_id: RecordId,
}

struct AdtInstanceInfo {
    index: usize,
    location_id: LocationId,
    adt_id: AdtId,
}

enum InstanceInfo {
    Record(RecordInstanceInfo),
    Adt(AdtInstanceInfo),
}

impl Program {
    pub fn new() -> Program {
        Program {
            modules: ItemContainer::new(),
            functions: ItemContainer::new(),
            function_types: ItemContainer::new(),
            records: ItemContainer::new(),
            adts: ItemContainer::new(),
            variants: ItemContainer::new(),
            classes: ItemContainer::new(),
            instances: ItemContainer::new(),
            exprs: ItemContainer::new(),
            type_signatures: ItemContainer::new(),
            patterns: ItemContainer::new(),
            imports: ItemContainer::new(),
            record_fields: ItemContainer::new(),
            protocols: ItemContainer::new(),
            actors: ItemContainer::new(),
        }
    }

    fn create_expr(&mut self, expr: Expr, location_id: LocationId) -> ExprId {
        let id = self.exprs.get_id();
        self.exprs.add_item(id, ItemInfo::new(expr, location_id));
        id
    }

    fn create_adt_tojson_instance(&mut self, info: AdtInstanceInfo) {
        let id = self.instances.get_id();
        let adt = self.adts.get(&info.adt_id).clone();
        let mut constraints = Vec::new();
        let mut type_args = Vec::new();
        for arg in &adt.type_args {
            let c = Constraint {
                class_name: TOJSON_CLASS_NAME.to_string(),
                arg: arg.0.clone(),
                location_id: info.location_id,
            };
            let type_arg = TypeSignature::TypeArg(arg.0.clone());
            let type_arg_id = self.type_signatures.get_id();
            self.type_signatures
                .add_item(type_arg_id, ItemInfo::new(type_arg, info.location_id));
            type_args.push(type_arg_id);
            constraints.push(c);
        }
        let type_signature_id = self.type_signatures.get_id();
        let type_signature = TypeSignature::Named(adt.name.clone(), type_args);
        self.type_signatures.add_item(
            type_signature_id,
            ItemInfo::new(type_signature, info.location_id),
        );
        let mut member_functions = BTreeMap::new();
        let mut cases = Vec::new();
        for variant_id in &adt.variants {
            let json_object_item_name = "Json.JsonObjectItem".to_string();
            let json_object_item_path_id =
                self.create_expr(Expr::Path(json_object_item_name.clone()), info.location_id);
            let variant = self.variants.get(variant_id).clone();
            let mut object_items = Vec::new();
            let mut bindings = Vec::new();
            let variant_type_signature =
                self.type_signatures.get(&variant.type_signature_id).clone();
            let item_count = if let TypeSignature::Variant(_, items) = variant_type_signature.item {
                items.len()
            } else {
                panic!("variant type signature is not a variant!")
            };
            for item_index in 0..item_count {
                let tojson_name = "Json.Serialize.toJson".to_string();
                let tojson_name_path_id =
                    self.create_expr(Expr::Path(tojson_name.clone()), info.location_id);
                let item_name = format!("i{}", item_index);
                let item_name_path_id =
                    self.create_expr(Expr::Path(item_name.clone()), info.location_id);
                let tojson_fn_call_id = self.create_expr(
                    Expr::FunctionCall(tojson_name_path_id, vec![item_name_path_id]),
                    info.location_id,
                );
                let bind = Pattern::Binding(item_name);
                let bind_id = self.patterns.get_id();
                self.patterns
                    .add_item(bind_id, ItemInfo::new(bind, info.location_id));
                bindings.push(bind_id);
                object_items.push(tojson_fn_call_id);
            }
            let list_ctor_id = self.create_expr(Expr::List(object_items), info.location_id);
            let jsonlist_name = "Json.JsonList".to_string();
            let jsonlist_name_path_id =
                self.create_expr(Expr::Path(jsonlist_name.clone()), info.location_id);
            let jsonlist_id = self.create_expr(
                Expr::FunctionCall(jsonlist_name_path_id, vec![list_ctor_id]),
                info.location_id,
            );
            let variant_name_string_literal_id =
                self.create_expr(Expr::StringLiteral(variant.name.clone()), info.location_id);
            let json_object_item_fn_id = self.create_expr(
                Expr::FunctionCall(
                    json_object_item_path_id,
                    vec![variant_name_string_literal_id, jsonlist_id],
                ),
                info.location_id,
            );
            let json_object_path_id =
                self.create_expr(Expr::Path("Json.JsonObject".to_string()), info.location_id);
            let object_item_list_id =
                self.create_expr(Expr::List(vec![json_object_item_fn_id]), info.location_id);
            let json_object_fn_call_id = self.create_expr(
                Expr::FunctionCall(json_object_path_id, vec![object_item_list_id]),
                info.location_id,
            );
            let pattern_id = self.patterns.get_id();

            let ctor_pattern = Pattern::Constructor(variant.name.clone(), bindings);
            self.patterns
                .add_item(pattern_id, ItemInfo::new(ctor_pattern, info.location_id));
            let case = Case {
                pattern_id: pattern_id,
                body: json_object_fn_call_id,
            };
            cases.push(case);
        }

        let to_json_function_id = self.functions.get_id();
        member_functions.insert("toJson".to_string(), vec![to_json_function_id]);
        let arg0_name = "arg0".to_string();
        let arg0_name_path_id = self.create_expr(Expr::Path(arg0_name.clone()), info.location_id);
        let case_expr_id =
            self.create_expr(Expr::CaseOf(arg0_name_path_id, cases), info.location_id);
        let to_json_name = "toJson".to_string();
        let to_json_function = Function {
            id: to_json_function_id,
            name: to_json_name.clone(),
            args: vec![("arg0".to_string(), info.location_id)],
            body: FunctionBody::Expr(case_expr_id),
            location_id: info.location_id,
        };
        self.functions
            .add_item(to_json_function_id, to_json_function);
        {
            let module = self.modules.get_mut(&adt.module_id);
            module.instances.push(id);
        }
        let member_function_types = BTreeMap::new();
        let instance = Instance {
            id: id,
            name: None,
            class_name: TOJSON_CLASS_NAME.to_string(),
            type_signature_id: type_signature_id,
            constraints: constraints,
            member_functions: member_functions,
            member_function_types: member_function_types,
            location_id: info.location_id,
        };
        self.instances.add_item(instance.id, instance);
    }

    fn create_adt_fromjson_instance(&mut self, info: AdtInstanceInfo) {
        let id = self.instances.get_id();
        let adt = self.adts.get(&info.adt_id).clone();
        let mut constraints = Vec::new();
        let mut type_args = Vec::new();
        for arg in &adt.type_args {
            let c = Constraint {
                class_name: FROMJSON_CLASS_NAME.to_string(),
                arg: arg.0.clone(),
                location_id: info.location_id,
            };
            let type_arg = TypeSignature::TypeArg(arg.0.clone());
            let type_arg_id = self.type_signatures.get_id();
            self.type_signatures
                .add_item(type_arg_id, ItemInfo::new(type_arg, info.location_id));
            type_args.push(type_arg_id);
            constraints.push(c);
        }
        let type_signature_id = self.type_signatures.get_id();
        let type_signature = TypeSignature::Named(adt.name.clone(), type_args);
        self.type_signatures.add_item(
            type_signature_id,
            ItemInfo::new(type_signature, info.location_id),
        );
        let mut member_functions = BTreeMap::new();
        let mut cases = Vec::new();
        for variant_id in &adt.variants {
            let variant = self.variants.get(variant_id).clone();
            let mut object_items = Vec::new();
            let variant_type_signature =
                self.type_signatures.get(&variant.type_signature_id).clone();
            let item_count = if let TypeSignature::Variant(_, items) = variant_type_signature.item {
                items.len()
            } else {
                panic!("variant type signature is not a variant!")
            };
            for item_index in 0..item_count {
                let index_literal =
                    self.create_expr(Expr::IntegerLiteral(item_index as i64), info.location_id);
                let fromjson_name = "Json.Serialize.fromJson".to_string();
                let fromjson_name_path_id =
                    self.create_expr(Expr::Path(fromjson_name.clone()), info.location_id);
                let items_name_path_id =
                    self.create_expr(Expr::Path("items".to_string()), info.location_id);
                let atindex_path_id =
                    self.create_expr(Expr::Path("atIndex".to_string()), info.location_id);
                let atindex_call_id = self.create_expr(
                    Expr::FunctionCall(atindex_path_id, vec![items_name_path_id, index_literal]),
                    info.location_id,
                );
                let fromjson_fn_call_id = self.create_expr(
                    Expr::FunctionCall(fromjson_name_path_id, vec![atindex_call_id]),
                    info.location_id,
                );
                object_items.push(fromjson_fn_call_id);
            }
            let ctor_path_id = self.create_expr(Expr::Path(variant.name.clone()), info.location_id);
            let ctor_fn_call_id = self.create_expr(
                Expr::FunctionCall(ctor_path_id, object_items),
                info.location_id,
            );
            let pattern_id = self.patterns.get_id();
            let string_pattern = Pattern::StringLiteral(variant.name.clone());
            self.patterns
                .add_item(pattern_id, ItemInfo::new(string_pattern, info.location_id));
            let case = Case {
                pattern_id: pattern_id,
                body: ctor_fn_call_id,
            };
            cases.push(case);
        }

        let wildcard_pat = self.patterns.get_id();
        self.patterns.add_item(
            wildcard_pat,
            ItemInfo::new(Pattern::Wildcard, info.location_id),
        );
        let panic_path = self.create_expr(Expr::Path("panic".to_string()), info.location_id);
        let unexpected_string = self.create_expr(
            Expr::StringLiteral("unexpected variant in json".to_string()),
            info.location_id,
        );
        let panic_call = self.create_expr(
            Expr::FunctionCall(panic_path, vec![unexpected_string]),
            info.location_id,
        );
        let wildcard_case = Case {
            pattern_id: wildcard_pat,
            body: panic_call,
        };
        cases.push(wildcard_case);

        let from_json_function_id = self.functions.get_id();
        member_functions.insert("fromJson".to_string(), vec![from_json_function_id]);
        let arg0_name = "arg0".to_string();
        let arg0_name_path_id = self.create_expr(Expr::Path(arg0_name.clone()), info.location_id);
        let name_name = "name".to_string();
        let name_name_path_id = self.create_expr(Expr::Path(name_name.clone()), info.location_id);
        let case_expr_id =
            self.create_expr(Expr::CaseOf(name_name_path_id, cases), info.location_id);
        let name_pat = Pattern::Binding("name".to_string());
        let name_pat_id = self.patterns.get_id();
        self.patterns
            .add_item(name_pat_id, ItemInfo::new(name_pat, info.location_id));
        let items_pat = Pattern::Binding("items".to_string());
        let items_pat_id = self.patterns.get_id();
        self.patterns
            .add_item(items_pat_id, ItemInfo::new(items_pat, info.location_id));
        let bind_pattern = Pattern::Tuple(vec![name_pat_id, items_pat_id]);
        let bind_pattern_id = self.patterns.get_id();
        self.patterns.add_item(
            bind_pattern_id,
            ItemInfo::new(bind_pattern, info.location_id),
        );
        let unpack_path =
            self.create_expr(Expr::Path("unpackVariant".to_string()), info.location_id);
        let unpack_expr_id = self.create_expr(
            Expr::FunctionCall(unpack_path, vec![arg0_name_path_id]),
            info.location_id,
        );
        let bind_expr_id = self.create_expr(
            Expr::Bind(bind_pattern_id, unpack_expr_id),
            info.location_id,
        );
        let do_expr_id =
            self.create_expr(Expr::Do(vec![bind_expr_id, case_expr_id]), info.location_id);
        let from_json_name = "fromJson".to_string();
        let from_json_function = Function {
            id: from_json_function_id,
            name: from_json_name.clone(),
            args: vec![("arg0".to_string(), info.location_id)],
            body: FunctionBody::Expr(do_expr_id),
            location_id: info.location_id,
        };
        self.functions
            .add_item(from_json_function_id, from_json_function);
        {
            let module = self.modules.get_mut(&adt.module_id);
            module.instances.push(id);
        }
        let member_function_types = BTreeMap::new();
        let instance = Instance {
            id: id,
            name: None,
            class_name: FROMJSON_CLASS_NAME.to_string(),
            type_signature_id: type_signature_id,
            constraints: constraints,
            member_functions: member_functions,
            member_function_types: member_function_types,
            location_id: info.location_id,
        };
        self.instances.add_item(instance.id, instance);
    }

    fn create_record_fromjson_instance(&mut self, _info: RecordInstanceInfo) {}

    fn create_record_tojson_instance(&mut self, info: RecordInstanceInfo) {
        let id = self.instances.get_id();
        let record = self.records.get(&info.record_id).clone();
        let mut constraints = Vec::new();
        let mut type_args = Vec::new();
        for arg in &record.type_args {
            let c = Constraint {
                class_name: TOJSON_CLASS_NAME.to_string(),
                arg: arg.0.clone(),
                location_id: info.location_id,
            };
            let type_arg = TypeSignature::TypeArg(arg.0.clone());
            let type_arg_id = self.type_signatures.get_id();
            self.type_signatures
                .add_item(type_arg_id, ItemInfo::new(type_arg, info.location_id));
            type_args.push(type_arg_id);
            constraints.push(c);
        }
        let type_signature_id = self.type_signatures.get_id();
        let type_signature = TypeSignature::Named(record.name.clone(), type_args);
        self.type_signatures.add_item(
            type_signature_id,
            ItemInfo::new(type_signature, info.location_id),
        );
        let mut member_functions = BTreeMap::new();
        let mut object_items = Vec::new();
        for field in &record.fields {
            let field = self.record_fields.get(field).clone();
            let json_object_item_name = "Json.JsonObjectItem".to_string();
            let json_object_item_path_id =
                self.create_expr(Expr::Path(json_object_item_name.clone()), info.location_id);
            let tojson_name = "Json.Serialize.toJson".to_string();
            let tojson_name_path_id =
                self.create_expr(Expr::Path(tojson_name.clone()), info.location_id);
            let arg0_name = "arg0".to_string();
            let arg0_name_path_id =
                self.create_expr(Expr::Path(arg0_name.clone()), info.location_id);
            let field_name_string_literal_id =
                self.create_expr(Expr::StringLiteral(field.name.clone()), info.location_id);
            let field_access_id = self.create_expr(
                Expr::FieldAccess(field.name.clone(), arg0_name_path_id),
                info.location_id,
            );
            let tojson_fn_call_id = self.create_expr(
                Expr::FunctionCall(tojson_name_path_id, vec![field_access_id]),
                info.location_id,
            );
            let json_object_item_fn_id = self.create_expr(
                Expr::FunctionCall(
                    json_object_item_path_id,
                    vec![field_name_string_literal_id, tojson_fn_call_id],
                ),
                info.location_id,
            );
            object_items.push(json_object_item_fn_id);
        }
        let to_json_function_id = self.functions.get_id();
        member_functions.insert("toJson".to_string(), vec![to_json_function_id]);
        let json_object_path_id =
            self.create_expr(Expr::Path("Json.JsonObject".to_string()), info.location_id);
        let object_item_list_id = self.create_expr(Expr::List(object_items), info.location_id);
        let json_object_fn_call_id = self.create_expr(
            Expr::FunctionCall(json_object_path_id, vec![object_item_list_id]),
            info.location_id,
        );
        // Json.JsonObject [JsonObjectItem "alma" (toJson r.alma), JsonObjectItem "korte" (toJson r.korte)]
        let to_json_name = "toJson".to_string();
        let to_json_function = Function {
            id: to_json_function_id,
            name: to_json_name.clone(),
            args: vec![("arg0".to_string(), info.location_id)],
            body: FunctionBody::Expr(json_object_fn_call_id),
            location_id: info.location_id,
        };
        self.functions
            .add_item(to_json_function_id, to_json_function);
        {
            let module = self.modules.get_mut(&record.module_id);
            module.instances.push(id);
        }
        let member_function_types = BTreeMap::new();
        let instance = Instance {
            id: id,
            name: None,
            class_name: TOJSON_CLASS_NAME.to_string(),
            type_signature_id: type_signature_id,
            constraints: constraints,
            member_functions: member_functions,
            member_function_types: member_function_types,
            location_id: info.location_id,
        };
        self.instances.add_item(instance.id, instance);
    }

    pub fn create_tojson_instances(&mut self) {
        let mut tojson_instances = Vec::new();
        let mut fromjson_instances = Vec::new();
        for (id, record) in &mut self.records.items {
            let mut instances = Vec::new();
            for (index, class) in record.derived_classes.iter().enumerate() {
                //println!("{} derives {}", record.name, class.name);
                let mut args = Vec::new();
                for arg in &record.type_args {
                    args.push(arg.0.clone());
                }
                if class.name == TOJSON_CLASS_NAME {
                    let info = RecordInstanceInfo {
                        index: index,
                        location_id: class.location_id,
                        record_id: *id,
                    };
                    instances.push(InstanceInfo::Record(info));
                }
            }
            for info in instances {
                match info {
                    InstanceInfo::Record(ref r) => {
                        record.derived_classes.remove(r.index);
                        tojson_instances.push(info);
                    }
                    InstanceInfo::Adt(_) => {
                        panic!("No")
                    }
                }
            }
            let mut instances = Vec::new();
            for (index, class) in record.derived_classes.iter().enumerate() {
                //println!("{} derives {}", record.name, class.name);
                let mut args = Vec::new();
                for arg in &record.type_args {
                    args.push(arg.0.clone());
                }
                if class.name == FROMJSON_CLASS_NAME {
                    let info = RecordInstanceInfo {
                        index: index,
                        location_id: class.location_id,
                        record_id: *id,
                    };
                    instances.push(InstanceInfo::Record(info));
                }
            }
            for info in instances {
                match info {
                    InstanceInfo::Record(_) => {
                        //record.derived_classes.remove(r.index);
                        //fromjson_instances.push(info);
                    }
                    InstanceInfo::Adt(_) => {
                        panic!("No")
                    }
                }
            }
        }
        for (id, adt) in &mut self.adts.items {
            let mut instances = Vec::new();
            for (index, class) in adt.derived_classes.iter().enumerate() {
                //println!("{} derives {}", record.name, class.name);
                let mut args = Vec::new();
                for arg in &adt.type_args {
                    args.push(arg.0.clone());
                }
                if class.name == TOJSON_CLASS_NAME {
                    let info = AdtInstanceInfo {
                        index: index,
                        location_id: class.location_id,
                        adt_id: *id,
                    };
                    instances.push(InstanceInfo::Adt(info));
                }
            }
            for info in instances {
                match info {
                    InstanceInfo::Record(_) => {
                        panic!("No2")
                    }
                    InstanceInfo::Adt(ref a) => {
                        adt.derived_classes.remove(a.index);
                        tojson_instances.push(info);
                    }
                }
            }
            let mut instances = Vec::new();
            for (index, class) in adt.derived_classes.iter().enumerate() {
                //println!("{} derives {}", record.name, class.name);
                let mut args = Vec::new();
                for arg in &adt.type_args {
                    args.push(arg.0.clone());
                }
                if class.name == FROMJSON_CLASS_NAME {
                    let info = AdtInstanceInfo {
                        index: index,
                        location_id: class.location_id,
                        adt_id: *id,
                    };
                    instances.push(InstanceInfo::Adt(info));
                }
            }
            for info in instances {
                match info {
                    InstanceInfo::Record(_) => {
                        panic!("No2")
                    }
                    InstanceInfo::Adt(ref a) => {
                        adt.derived_classes.remove(a.index);
                        fromjson_instances.push(info);
                    }
                }
            }
        }
        for info in tojson_instances {
            match info {
                InstanceInfo::Record(r) => {
                    self.create_record_tojson_instance(r);
                }
                InstanceInfo::Adt(a) => {
                    self.create_adt_tojson_instance(a);
                }
            }
        }
        for info in fromjson_instances {
            match info {
                InstanceInfo::Record(r) => {
                    self.create_record_fromjson_instance(r);
                }
                InstanceInfo::Adt(a) => {
                    self.create_adt_fromjson_instance(a);
                }
            }
        }
    }
}
