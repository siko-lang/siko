use std::{collections::BTreeMap, ops::Deref};

use crate::siko::{
    hir::{OwnershipVar::OwnershipVarInfo, Program::Program, Type::Type},
    qualifiedname::QualifiedName,
};

use super::DataGroups::createDataGroups;

pub struct DataOwnershipVarInference {
    program: Program,
}

impl DataOwnershipVarInference {
    pub fn new(program: Program) -> DataOwnershipVarInference {
        DataOwnershipVarInference { program: program }
    }

    pub fn process(mut self) -> Program {
        let mut ownershipVarInfoHandler = OwnershipVarInfoHandler::new();
        let data_groups = createDataGroups(&self.program.structs, &self.program.enums);
        for group in data_groups {
            let mut handler = GroupOwnershipVarInfoHandler::new(group.items.clone(), &ownershipVarInfoHandler);
            // println!("Processing group {:?}", group.items);
            for item in &group.items {
                if let Some(c) = self.program.structs.get_mut(&item) {
                    for f in &mut c.fields {
                        f.ty = handler.processType(&f.ty);
                    }
                }
                if let Some(e) = self.program.enums.get_mut(&item) {
                    for v in &mut e.variants {
                        for i in v.items.iter_mut() {
                            *i = handler.processType(i);
                        }
                    }
                }
            }

            for item in &group.items {
                if let Some(c) = self.program.structs.get_mut(&item) {
                    c.ty = handler.updateGroup(&c.ty);
                    for f in &mut c.fields {
                        f.ty = handler.updateGroup(&f.ty);
                    }
                }
                if let Some(e) = self.program.enums.get_mut(&item) {
                    e.ty = handler.updateGroup(&e.ty);
                    for v in &mut e.variants {
                        for i in v.items.iter_mut() {
                            *i = handler.updateGroup(i);
                        }
                    }
                }
            }

            let ownership_info = handler.ownership_info;
            for item in &group.items {
                ownershipVarInfoHandler.add(item.clone(), ownership_info.clone());
            }
            for item in &group.items {
                if let Some(c) = self.program.structs.get_mut(&item) {
                    c.ownership_info = Some(ownership_info.clone());
                }
                if let Some(e) = self.program.enums.get_mut(&item) {
                    e.ownership_info = Some(ownership_info.clone());
                }
            }
        }

        //ownershipVarInfoHandler.dump();
        //println!("program after data ownership var inference:\n{}", self.program);
        self.program
    }
}

struct OwnershipVarInfoHandler {
    args: BTreeMap<QualifiedName, OwnershipVarInfo>,
}

impl OwnershipVarInfoHandler {
    fn new() -> OwnershipVarInfoHandler {
        OwnershipVarInfoHandler { args: BTreeMap::new() }
    }

    fn add(&mut self, qn: QualifiedName, info: OwnershipVarInfo) {
        self.args.insert(qn, info);
    }

    fn dump(&self) {
        for (qn, info) in &self.args {
            println!("{}: {:?}", qn, info);
        }
    }
}

struct GroupOwnershipVarInfoHandler<'a> {
    ownership_info: OwnershipVarInfo,
    items: Vec<QualifiedName>,
    ownershipVarInfoHandler: &'a OwnershipVarInfoHandler,
}

impl<'a> GroupOwnershipVarInfoHandler<'a> {
    fn new(
        items: Vec<QualifiedName>,
        lifetimeInfoHandler: &'a OwnershipVarInfoHandler,
    ) -> GroupOwnershipVarInfoHandler<'a> {
        GroupOwnershipVarInfoHandler {
            ownership_info: OwnershipVarInfo::new(),
            items: items,
            ownershipVarInfoHandler: lifetimeInfoHandler,
        }
    }

    pub fn processType(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Named(n, _) => {
                //println!("Looking for base {}", n);
                if !self.items.contains(&n) {
                    let mut ownershipVarInfo = OwnershipVarInfo::new();
                    let args: &OwnershipVarInfo = self
                        .ownershipVarInfoHandler
                        .args
                        .get(&n)
                        .expect("dep not found in ownershipVarInfoHandler");
                    let selfVar = self.ownership_info.allocate();
                    for _ in &args.args {
                        ownershipVarInfo.add(self.ownership_info.allocate());
                    }
                    //println!("Adding ownership var info for {}: {:?}", n, ownershipVarInfo);
                    Type::OwnershipVar(selfVar, Box::new(ty.clone()), ownershipVarInfo)
                } else {
                    let selfVar = self.ownership_info.allocate();
                    Type::OwnershipVar(selfVar, Box::new(ty.clone()), OwnershipVarInfo::new())
                }
            }
            _ => ty.clone(),
        }
    }

    pub fn updateGroup(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Named(n, _) => {
                let mut info = self.ownership_info.clone();
                //println!("Looking for base {}", n);
                if self.items.contains(&n) {
                    Type::OwnershipVar(info.allocate(), Box::new(ty.clone()), self.ownership_info.clone())
                } else {
                    ty.clone()
                }
            }
            Type::OwnershipVar(v, ity, _) => {
                if let Type::Named(n, _) = ity.deref() {
                    if self.items.contains(n) {
                        Type::OwnershipVar(v.clone(), ity.clone(), self.ownership_info.clone())
                    } else {
                        ty.clone()
                    }
                } else {
                    panic!("Expected OwnershipVar to have a named type, found {}", ty);
                }
            }
            _ => ty.clone(),
        }
    }
}
