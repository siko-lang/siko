use crate::siko::{
    hir::{
        Data::{Enum as HirEnum, Struct as HirStruct},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    hir_lowering::{FunctionBuilder::Builder, NameManager::NameManager},
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::builtins::{
        getArrayTypeName, getBoolTypeName, getI32TypeName, getI8TypeName, getIntTypeName, getU16TypeName,
        getU32TypeName, getU64TypeName, getU8TypeName,
    },
};

pub struct Lowering {
    pub program: HirProgram,
    pub nameManager: NameManager,
}

impl Lowering {
    pub fn new(program: HirProgram) -> Lowering {
        Lowering {
            program,
            nameManager: NameManager::new(),
        }
    }

    pub fn lowerType(&self, ty: &HirType) -> MirType {
        match ty {
            HirType::Named(name, _) => {
                if self.program.structs.get(name).is_some() {
                    if *name == getIntTypeName() {
                        MirType::Int64
                    } else if *name == getU8TypeName() {
                        MirType::UInt8
                    } else if *name == getI8TypeName() {
                        MirType::Int8
                    } else if *name == getI32TypeName() {
                        MirType::Int32
                    } else if *name == getU64TypeName() {
                        MirType::UInt64
                    } else if *name == getU16TypeName() {
                        MirType::UInt16
                    } else if *name == getU32TypeName() {
                        MirType::UInt32
                    } else {
                        MirType::Struct(self.nameManager.processName(name))
                    }
                } else {
                    if *name == getBoolTypeName() {
                        MirType::Int64
                    } else {
                        MirType::Union(self.nameManager.processName(name))
                    }
                }
            }
            HirType::Tuple(_) => unreachable!("Tuple in MIR"),
            HirType::Function(_, _) => unreachable!("Function type in MIR"),
            HirType::Var(_) => unreachable!("Type variable in MIR"),
            HirType::Reference(ty) => MirType::Ptr(Box::new(self.lowerType(ty))),
            HirType::Ptr(ty) => MirType::Ptr(Box::new(self.lowerType(ty))),
            HirType::SelfType => todo!(),
            HirType::Never(_) => MirType::Void,
            HirType::NumericConstant(_) => unreachable!("NumericConstant ty lowering in MIR"),
            HirType::Void => MirType::Void,
            HirType::VoidPtr => MirType::VoidPtr,
            HirType::Coroutine(_, _) => {
                unreachable!("Coroutine type in MIR")
            }
        }
    }

    pub fn lowerStruct(&self, s: &HirStruct) -> Struct {
        let (base, ctx) = s.name.getUnmonomorphized();
        let fields = if base == getArrayTypeName() {
            let ctx = ctx.expect("Array type without context");
            let itemTy = ctx.args[0].clone();
            let itemTy = self.lowerType(&itemTy);
            let len = match &ctx.args[1] {
                HirType::NumericConstant(v) => v,
                _ => panic!("Array length is not a numeric constant"),
            };
            let len = (&len).parse().expect("Array length is not a valid number");
            let mirField = MirField {
                name: format!("field0"),
                ty: MirType::Array(Box::new(itemTy), len),
            };
            vec![mirField]
        } else {
            let mut fields = Vec::new();
            for f in &s.fields {
                let mirField = MirField {
                    name: f.name.clone(),
                    ty: self.lowerType(&f.ty),
                };
                fields.push(mirField);
            }
            fields
        };
        //println!("Lowering structDef {}", s.name);
        Struct {
            name: self.nameManager.processName(&s.name),
            originalName: if s.originalName.is_empty() {
                format!("{}", s.name)
            } else {
                s.originalName.clone()
            },
            fields: fields,
            size: 0,
            alignment: 0,
        }
    }

    pub fn lowerEnum(&self, e: &HirEnum) -> Union {
        let mut variants = Vec::new();

        for v in &e.variants {
            assert_eq!(v.items.len(), 1);
            let mirVariant = MirVariant {
                name: self.nameManager.processName(&v.name),
                originalName: v.name.toString(),
                ty: self.lowerType(&v.items[0]),
            };
            variants.push(mirVariant);
        }
        Union {
            name: self.nameManager.processName(&e.name),
            originalName: e.name.to_string(),
            variants: variants,
            size: 0,
            alignment: 0,
            payloadSize: 0,
        }
    }

    pub fn lowerProgram(&self) -> MirProgram {
        let mut mirProgram = MirProgram::new();

        //println!("Lowering structs");

        for (n, s) in &self.program.structs {
            if *n == getIntTypeName() {
                continue;
            }
            if *n == getU8TypeName() {
                continue;
            }
            let c = self.lowerStruct(s);
            mirProgram.structs.insert(self.nameManager.processName(n), c);
        }

        //println!("Lowering enums");

        for (n, e) in &self.program.enums {
            if *n == getBoolTypeName() {
                continue;
            }
            let u = self.lowerEnum(e);
            mirProgram.unions.insert(self.nameManager.processName(n), u);
        }

        //println!("Lowering functions");

        for (_, function) in &self.program.functions {
            let mut builder = Builder::new(&self, function);
            if let Some(f) = builder.lowerFunction() {
                mirProgram.functions.insert(f.name.clone(), f);
            }
        }

        mirProgram
    }
}
