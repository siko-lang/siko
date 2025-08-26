use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    pub name: QualifiedName,
    pub traitName: QualifiedName,
    pub types: Vec<Type>,
    pub typeParams: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub constraintContext: ConstraintContext,
    pub members: Vec<MemberInfo>,
}

impl Instance {
    pub fn new(
        name: QualifiedName,
        traitName: QualifiedName,
        types: Vec<Type>,
        typeParams: Vec<Type>,
        associatedTypes: Vec<AssociatedType>,
        constraintContext: ConstraintContext,
    ) -> Instance {
        Instance {
            name,
            traitName: traitName,
            types: types,
            typeParams: typeParams,
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
            self.name,
            self.traitName,
            formatTypes(&self.types),
            self.constraintContext,
            associatedTypes,
            methods
        )
    }
}
