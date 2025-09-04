use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use crate::siko::{
    backend::borrowcheck::DataGroups::ExtendedType,
    hir::{
        Apply::Apply,
        Type::{normalizeTypesWithSub, Type},
    },
    qualifiedname::QualifiedName,
};

#[derive(Clone)]
pub struct Link {
    pub from: Type,
    pub to: Type,
}

impl Link {
    pub fn new(from: Type, to: Type) -> Self {
        Link { from, to }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

impl Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct FunctionProfile {
    pub name: QualifiedName,
    pub args: Vec<ExtendedType>,
    pub result: ExtendedType,
    pub links: Vec<Link>,
}

impl FunctionProfile {
    pub fn collectVars(&self) -> Vec<Type> {
        let mut allVars = Vec::new();
        for arg in &self.args {
            for v in &arg.vars {
                if !allVars.contains(v) {
                    allVars.push(v.clone());
                }
            }
        }
        for v in &self.result.vars {
            if !allVars.contains(v) {
                allVars.push(v.clone());
            }
        }
        for link in &self.links {
            if !allVars.contains(&link.from) {
                allVars.push(link.from.clone());
            }
            if !allVars.contains(&link.to) {
                allVars.push(link.to.clone());
            }
        }
        allVars
    }

    fn argVars(&self) -> Vec<Type> {
        let mut argVars = Vec::new();
        for arg in &self.args {
            for v in &arg.vars {
                if !argVars.contains(v) {
                    argVars.push(v.clone());
                }
            }
        }
        argVars
    }

    fn resultVars(&self) -> Vec<Type> {
        let mut resultVars = Vec::new();
        for v in &self.result.vars {
            if !resultVars.contains(v) {
                resultVars.push(v.clone());
            }
        }
        resultVars
    }

    pub fn processLinks(&mut self) {
        let mut allDeps = BTreeMap::new();
        let argVars = self.argVars();
        for argVar in &argVars {
            allDeps.entry(argVar.clone()).or_insert_with(Vec::new);
        }
        let resultVars = self.resultVars();
        for resultVar in &resultVars {
            allDeps.entry(resultVar.clone()).or_insert_with(Vec::new);
        }
        for link in &mut self.links {
            allDeps.entry(link.to.clone()).or_insert_with(Vec::new);
            allDeps
                .entry(link.from.clone())
                .or_insert_with(Vec::new)
                .push(link.to.clone());
        }
        //println!("ArgVars = {:?}", argVars);
        //println!("ResultVars = {:?}", resultVars);
        let mut finalLinks = Vec::new();
        for argVar in argVars {
            //println!("Processing argVar: {}", argVar);
            let mut current = vec![argVar.clone()];
            loop {
                let mut added = false;
                let copy = current.clone();
                for c in &copy {
                    let deps = allDeps.get(&c).expect("argVar must be in allDeps");
                    for dep in deps {
                        let mut found = false;
                        for rVar in &resultVars {
                            if dep == rVar {
                                found = true;
                                //println!("Link created: {} -> {}", argVar, rVar);
                                finalLinks.push(Link::new(argVar.clone(), rVar.clone()));
                            }
                        }
                        if !found && !current.contains(dep) {
                            added = true;
                            //println!("Dependency added: {} -> {}", argVar, dep);
                            current.push(dep.clone());
                        }
                    }
                }
                if !added {
                    break;
                }
            }
        }
        self.links = finalLinks;
    }

    pub fn normalize(&mut self) {
        let allVars = self.collectVars();
        let (_, sub) = normalizeTypesWithSub(&allVars);
        *self = self.clone().apply(&sub);
    }
}

impl Display for FunctionProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}({}) -> {} {:?}", self.name, args, self.result, self.links)
    }
}
