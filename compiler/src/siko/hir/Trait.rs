use std::{cmp::Ordering, collections::BTreeSet, fmt};

use crate::siko::{hir::Type::formatTypes, qualifiedname::QualifiedName};

use super::{ConstraintContext::ConstraintContext, Type::Type};

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub name: String,
    pub fullName: QualifiedName,
    pub default: bool,
    pub memberType: Type,
}

impl fmt::Display for MemberInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {} => ({}) / {}", self.name, self.fullName, self.memberType)
    }
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: QualifiedName,
    pub params: Vec<Type>,
    pub associatedTypes: Vec<String>,
    pub members: Vec<MemberInfo>,
    pub constraint: ConstraintContext,
}

impl Trait {
    pub fn new(
        name: QualifiedName,
        params: Vec<Type>,
        associatedTypes: Vec<String>,
        constraint: ConstraintContext,
    ) -> Trait {
        Trait {
            name: name,
            params: params,
            associatedTypes: associatedTypes,
            members: Vec::new(),
            constraint: constraint,
        }
    }
}

impl fmt::Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let associated_types_str = if !self.associatedTypes.is_empty() {
            let types = self.associatedTypes.join(", ");
            format!("\n    Associated Types: {}", types)
        } else {
            String::new()
        };

        let members_str = if !self.members.is_empty() {
            let members = self
                .members
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join("\n    ");
            format!("\n    Members:\n    {}", members)
        } else {
            String::new()
        };

        write!(
            f,
            "trait {}{}: => {}{}{}",
            self.name,
            formatTypes(&self.params),
            self.constraint,
            associated_types_str,
            members_str,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Protocol {
    pub name: QualifiedName,
    pub params: Vec<Type>,
    pub associatedTypes: Vec<String>,
    pub members: Vec<MemberInfo>,
    pub constraint: ConstraintContext,
}

impl Protocol {
    pub fn new(
        name: QualifiedName,
        params: Vec<Type>,
        associatedTypes: Vec<String>,
        constraint: ConstraintContext,
    ) -> Protocol {
        Protocol {
            name: name,
            params: params,
            associatedTypes: associatedTypes,
            members: Vec::new(),
            constraint: constraint,
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let associated_types_str = if !self.associatedTypes.is_empty() {
            let types = self.associatedTypes.join(", ");
            format!("\n    Associated Types: {}", types)
        } else {
            String::new()
        };

        let members_str = if !self.members.is_empty() {
            let members = self
                .members
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join("\n    ");
            format!("\n    Members:\n    {}", members)
        } else {
            String::new()
        };

        write!(
            f,
            "trait {}{}: => {}{}{}",
            self.name,
            formatTypes(&self.params),
            self.constraint,
            associated_types_str,
            members_str,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedType {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for AssociatedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub id: u64,
    pub traitName: QualifiedName,
    pub types: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub constraintContext: ConstraintContext,
    pub members: Vec<MemberInfo>,
}

impl Instance {
    pub fn new(
        id: u64,
        traitName: QualifiedName,
        types: Vec<Type>,
        associatedTypes: Vec<AssociatedType>,
        constraintContext: ConstraintContext,
    ) -> Instance {
        Instance {
            id: id,
            traitName: traitName,
            types: types,
            associatedTypes: associatedTypes,
            constraintContext: constraintContext,
            members: Vec::new(),
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let methods = self
            .members
            .iter()
            .map(|m| format!("{}", m))
            .collect::<Vec<_>>()
            .join(",\n    ");
        let associatedTypes = self
            .associatedTypes
            .iter()
            .map(|m| format!("{}", m))
            .collect::<Vec<_>>()
            .join(",\n    ");
        write!(
            f,
            "instance #{} of {} [{}] {} {{\n    {}\n    {}\n}}",
            self.id,
            self.traitName,
            formatTypes(&self.types),
            self.constraintContext,
            associatedTypes,
            methods
        )
    }
}

pub trait CompareSpecificity {
    fn compare(&self, other: &Self) -> BTreeSet<Ordering>;
}

impl<T: CompareSpecificity> CompareSpecificity for Vec<T> {
    fn compare(&self, other: &Self) -> BTreeSet<Ordering> {
        assert_eq!(self.len(), other.len());
        let mut result = BTreeSet::new();
        for (i1, i2) in self.iter().zip(other.iter()) {
            result.extend(i1.compare(i2));
        }
        return result;
    }
}

impl CompareSpecificity for Type {
    fn compare(&self, other: &Self) -> BTreeSet<Ordering> {
        fn res(o: Ordering) -> BTreeSet<Ordering> {
            let mut r = BTreeSet::new();
            r.insert(o);
            r
        }
        match (self, other) {
            (Type::Named(n1, args1), Type::Named(n2, args2)) if n1 == n2 => CompareSpecificity::compare(args1, args2),
            (Type::Tuple(args1), Type::Tuple(args2)) => CompareSpecificity::compare(args1, args2),
            (Type::Function(args1, r1), Type::Function(args2, r2)) => {
                let mut r = BTreeSet::new();
                r.extend(CompareSpecificity::compare(args1, args2));
                r.extend(CompareSpecificity::compare(&**r1, r2));
                r
            }
            (Type::Reference(t1, _), Type::Reference(t2, _)) => CompareSpecificity::compare(&**t1, t2),
            (Type::Var(_), Type::Var(_)) => res(Ordering::Equal),
            (_, Type::Var(_)) => res(Ordering::Greater),
            (Type::Var(_), _) => res(Ordering::Less),
            _ => BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Implementation {
    pub name: QualifiedName,
    pub protocolName: QualifiedName,
    pub types: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub constraintContext: ConstraintContext,
    pub members: Vec<MemberInfo>,
}

impl Implementation {
    pub fn new(
        name: QualifiedName,
        protocolName: QualifiedName,
        types: Vec<Type>,
        associatedTypes: Vec<AssociatedType>,
        constraintContext: ConstraintContext,
    ) -> Implementation {
        Implementation {
            name,
            protocolName: protocolName,
            types: types,
            associatedTypes: associatedTypes,
            constraintContext: constraintContext,
            members: Vec::new(),
        }
    }
}

impl fmt::Display for Implementation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let methods = self
            .members
            .iter()
            .map(|m| format!("{}", m))
            .collect::<Vec<_>>()
            .join(",\n    ");
        let associatedTypes = self
            .associatedTypes
            .iter()
            .map(|m| format!("{}", m))
            .collect::<Vec<_>>()
            .join(",\n    ");
        write!(
            f,
            "implementation #{} of {} [{}] {} {{\n    {}\n    {}\n}}",
            self.name,
            self.protocolName,
            formatTypes(&self.types),
            self.constraintContext,
            associatedTypes,
            methods
        )
    }
}
