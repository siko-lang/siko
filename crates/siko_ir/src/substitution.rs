use crate::class::ClassId;
use crate::types::Type;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Constraint {
    pub class_id: ClassId,
    pub ty: Type,
}

#[derive(Debug)]
pub enum Error {
    Fail,
    RecursiveType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Substitution {
    var_map: BTreeMap<usize, Type>,
    constraints: BTreeMap<ClassId, Vec<Type>>,
}

impl Substitution {
    pub fn empty() -> Substitution {
        Substitution {
            var_map: BTreeMap::new(),
            constraints: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.var_map.is_empty()
    }

    pub fn add_constraint(&mut self, class_id: ClassId, ty: Type) {
        let constraints = self
            .constraints
            .entry(class_id)
            .or_insert_with(|| Vec::new());
        constraints.push(ty);
    }

    pub fn add(&mut self, index: usize, ty: &Type) -> Result<(), Error> {
        if ty.contains(index) {
            Err(Error::RecursiveType)
        } else {
            let stored_ty = self.var_map.entry(index).or_insert_with(|| ty.clone());
            if stored_ty == ty {
                Ok(())
            } else {
                Err(Error::Fail)
            }
        }
    }

    pub fn dump(&self) {
        println!("Sub dump ---->");
        for (index, ty) in &self.var_map {
            println!("{} => {}", index, ty);
        }
        for (index, types) in &self.constraints {
            let s: Vec<_> = types.iter().map(|ty| format!("{}", ty)).collect();
            println!("class {} => ({})", index, s.join(", "));
        }
    }

    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(index, _) => match self.var_map.get(index) {
                Some(ty) => self.apply(ty),
                None => ty.clone(),
            },
            Type::Function(ty1, ty2) => {
                Type::Function(Box::new(self.apply(ty1)), Box::new(self.apply(ty2)))
            }
            Type::Tuple(items) => {
                let items = items.iter().map(|ty| self.apply(ty)).collect();
                Type::Tuple(items)
            }
            Type::Named(n, id, items) => {
                let items = items.iter().map(|ty| self.apply(ty)).collect();
                Type::Named(n.clone(), *id, items)
            }
            Type::FixedTypeArg(_, index, _) => match self.var_map.get(index) {
                Some(ty) => self.apply(ty),
                None => ty.clone(),
            },
            Type::Ref(item) => {
                let item = self.apply(item);
                Type::Ref(Box::new(item))
            }
        }
    }

    pub fn get_changes(&self) -> &BTreeMap<usize, Type> {
        &self.var_map
    }

    pub fn get_constraints(&self) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        for (class_id, types) in &self.constraints {
            for ty in types {
                constraints.push(Constraint {
                    class_id: *class_id,
                    ty: ty.clone(),
                });
            }
        }
        constraints
    }
}
