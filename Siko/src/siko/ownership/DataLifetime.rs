use std::collections::BTreeMap;

use crate::siko::{
    ir::{Lifetime::LifetimeInfo, Program::Program, Type::Type},
    qualifiedname::QualifiedName,
};

use super::DataGroups::createDataGroups;

pub struct DataLifeTimeInference {
    program: Program,
}

impl DataLifeTimeInference {
    pub fn new(program: Program) -> DataLifeTimeInference {
        DataLifeTimeInference { program: program }
    }

    pub fn process(mut self) -> Program {
        let mut lifetimeInfoHandler = LifetimeInfoHandler::new();
        let data_groups = createDataGroups(&self.program.classes, &self.program.enums);
        for group in data_groups {
            let mut handler = GroupLifetimeHandler::new(group.items.clone(), &lifetimeInfoHandler);
            //println!("Processing group {:?}", group.items);
            for item in &group.items {
                if let Some(c) = self.program.classes.get_mut(&item) {
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
                if let Some(c) = self.program.classes.get_mut(&item) {
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

            let lifetimes = handler.lifetimes;
            for item in &group.items {
                lifetimeInfoHandler.add(item.clone(), lifetimes.clone());
            }
            for item in &group.items {
                if let Some(c) = self.program.classes.get_mut(&item) {
                    c.lifetime_info = Some(lifetimes.clone());
                }
                if let Some(e) = self.program.enums.get_mut(&item) {
                    e.lifetime_info = Some(lifetimes.clone());
                }
            }
        }
        self.program
    }
}

struct LifetimeInfoHandler {
    args: BTreeMap<QualifiedName, LifetimeInfo>,
}

impl LifetimeInfoHandler {
    fn new() -> LifetimeInfoHandler {
        LifetimeInfoHandler {
            args: BTreeMap::new(),
        }
    }

    fn add(&mut self, qn: QualifiedName, info: LifetimeInfo) {
        self.args.insert(qn, info);
    }
}

struct GroupLifetimeHandler<'a> {
    lifetimes: LifetimeInfo,
    items: Vec<QualifiedName>,
    lifetimeInfoHandler: &'a LifetimeInfoHandler,
}

impl<'a> GroupLifetimeHandler<'a> {
    fn new(
        items: Vec<QualifiedName>,
        lifetimeInfoHandler: &'a LifetimeInfoHandler,
    ) -> GroupLifetimeHandler<'a> {
        GroupLifetimeHandler {
            lifetimes: LifetimeInfo::new(),
            items: items,
            lifetimeInfoHandler: lifetimeInfoHandler,
        }
    }

    pub fn processType(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Named(n, _, None) => {
                //println!("Looking for base {}", n);
                if !self.items.contains(&n) {
                    let mut lifetimes = LifetimeInfo::new();
                    let args: &LifetimeInfo = self
                        .lifetimeInfoHandler
                        .args
                        .get(&n)
                        .expect("dep not found in lifetime_args");
                    for _ in &args.args {
                        lifetimes.add(self.lifetimes.allocate());
                    }
                    Type::Named(n.clone(), Vec::new(), Some(lifetimes))
                } else {
                    ty.clone()
                }
            }
            Type::Reference(inner, None) => {
                let l = self.lifetimes.allocate();
                let inner = self.processType(&inner);
                Type::Reference(Box::new(inner), Some(l))
            }
            _ => ty.clone(),
        }
    }

    pub fn updateGroup(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Named(n, _, None) => {
                //println!("Looking for base {}", n);
                if self.items.contains(&n) {
                    Type::Named(n.clone(), Vec::new(), Some(self.lifetimes.clone()))
                } else {
                    ty.clone()
                }
            }
            Type::Reference(inner, l) => {
                let inner = self.updateGroup(inner);
                Type::Reference(Box::new(inner), l.clone())
            }
            _ => ty.clone(),
        }
    }
}
