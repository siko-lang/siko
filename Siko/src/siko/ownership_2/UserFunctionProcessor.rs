use core::panic;
use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Data::{Enum, Field, Struct, Variant},
        Function::Function,
        Instruction::InstructionKind,
        OwnershipVar::{OwnershipVar, OwnershipVarInfo},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::QualifiedName,
};

struct Mapper {
    mapping: BTreeMap<OwnershipVar, OwnershipVar>,
}

impl Mapper {
    pub fn new() -> Self {
        Mapper {
            mapping: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, from: OwnershipVar, to: OwnershipVar) {
        self.mapping.insert(from, to);
    }

    pub fn map(&self, var: OwnershipVar) -> OwnershipVar {
        if let Some(mapped_var) = self.mapping.get(&var) {
            *mapped_var
        } else {
            panic!("OwnershipVar {} not found in mapping", var);
        }
    }
}

trait Mappable {
    fn map(&self, mapper: &Mapper) -> Self;
}

impl Mappable for OwnershipVar {
    fn map(&self, mapper: &Mapper) -> Self {
        mapper.map(*self)
    }
}

impl Mappable for OwnershipVarInfo {
    fn map(&self, mapper: &Mapper) -> Self {
        let mut new_info = OwnershipVarInfo::new();
        for var in &self.args {
            new_info.add(var.map(mapper));
        }
        new_info
    }
}

impl Mappable for Type {
    fn map(&self, mapper: &Mapper) -> Self {
        match self {
            Type::OwnershipVar(var, ty, info) => {
                let mapped_var = var.map(mapper);
                let mapped_info = info.map(mapper);
                Type::OwnershipVar(mapped_var, ty.clone(), mapped_info)
            }
            t => panic!("Type mapping not implemented for this type: {}", t),
        }
    }
}

impl Mappable for Field {
    fn map(&self, mapper: &Mapper) -> Self {
        Field {
            name: self.name.clone(),
            ty: self.ty.map(mapper),
        }
    }
}

impl Mappable for Struct {
    fn map(&self, mapper: &Mapper) -> Self {
        let mut new_fields = Vec::new();
        for field in &self.fields {
            new_fields.push(field.map(mapper));
        }
        let new_ownership_info = self.ownership_info.as_ref().map(|info| info.map(mapper));
        Struct {
            name: self.name.clone(),
            ty: self.ty.clone(),
            fields: new_fields,
            methods: self.methods.clone(),
            ownership_info: new_ownership_info,
        }
    }
}

impl Mappable for Variant {
    fn map(&self, mapper: &Mapper) -> Self {
        Variant {
            name: self.name.clone(),
            items: self.items.iter().map(|item| item.map(mapper)).collect(),
        }
    }
}

impl Mappable for Enum {
    fn map(&self, mapper: &Mapper) -> Self {
        let new_variants = self.variants.iter().map(|v| v.map(mapper)).collect();
        let new_methods = self.methods.clone();
        let new_ownership_info = self.ownership_info.as_ref().map(|info| info.map(mapper));
        Enum {
            name: self.name.clone(),
            ty: self.ty.clone(),
            variants: new_variants,
            methods: new_methods,
            ownership_info: new_ownership_info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalVariable {
    pub fnName: QualifiedName,
    pub var: Variable,
}

pub struct UserFunctionProcessor<'a> {
    program: &'a Program,
    variables: BTreeMap<GlobalVariable, OwnershipVarInfo>,
    allocator: OwnershipVarInfo,
}
impl<'a> UserFunctionProcessor<'a> {
    pub fn new(program: &'a Program) -> Self {
        UserFunctionProcessor {
            program,
            variables: BTreeMap::new(),
            allocator: OwnershipVarInfo::new(),
        }
    }

    pub fn instantiateStruct(&mut self, s: &Struct) {
        let mut mapping = Mapper::new();
        let info = s.ownership_info.as_ref().unwrap();
        info.args.iter().for_each(|var| {
            let ownership_var = self.allocator.allocate();
            mapping.add(*var, ownership_var);
        });
        let s = s.map(&mapping);
    }

    pub fn processType(&mut self, t: &Type) {
        if let Type::Named(name, _) = t {
            if let Some(s) = self.program.structs.get(name) {}
        }
    }

    pub fn initialize(&mut self, f: &'a Function) {
        if let Some(body) = &f.body {
            let blockIds = body.getAllBlockIds();
            for id in blockIds {
                let block = body.getBlockById(id);
                for i in &block.instructions {
                    let vars = i.kind.collectVariables();
                    for var in vars {
                        let t = var.getType();
                        let global_var = GlobalVariable {
                            fnName: f.name.clone(),
                            var,
                        };
                        self.variables.insert(global_var, OwnershipVarInfo::new());
                    }
                }
            }
        }
    }

    pub fn processUserDefinedFunctions(&mut self, f: &'a Function) {}
}
