use std::fmt::Debug;
use std::fmt::Display;
use std::io::Write;

use crate::siko::hir::Block::Block;
use crate::siko::hir::Block::BlockId;
use crate::siko::hir::Body::Body;
use crate::siko::qualifiedname::QualifiedName;

use super::{ConstraintContext::ConstraintContext, Type::Type};

#[derive(Debug, Clone)]
pub enum Parameter {
    Named(String, Type, bool), // mutable
    SelfParam(bool, Type),     // mutable
}

impl Parameter {
    pub fn getName(&self) -> String {
        match &self {
            Parameter::Named(n, _, _) => n.clone(),
            Parameter::SelfParam(_, _) => "self".to_string(),
        }
    }

    pub fn getType(&self) -> Type {
        match &self {
            Parameter::Named(_, ty, _) => ty.clone(),
            Parameter::SelfParam(_, ty) => ty.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternKind {
    C,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    UserDefined,
    VariantCtor(i64),
    StructCtor,
    Extern(ExternKind),
    TraitMemberDecl(QualifiedName),
    TraitMemberDefinition(QualifiedName),
    EffectMemberDecl(QualifiedName),
    EffectMemberDefinition(QualifiedName),
}

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionKind::UserDefined => write!(f, "UserDefined"),
            FunctionKind::VariantCtor(id) => write!(f, "VariantCtor({})", id),
            FunctionKind::StructCtor => write!(f, "StructCtor"),
            FunctionKind::Extern(kind) => write!(f, "Extern({:?})", kind),
            FunctionKind::TraitMemberDecl(qn) => {
                write!(f, "TraitMemberDecl({})", qn)
            }
            FunctionKind::TraitMemberDefinition(qn) => {
                write!(f, "TraitMemberDefinition({})", qn)
            }
            FunctionKind::EffectMemberDecl(qn) => {
                write!(f, "EffectMemberDecl({})", qn)
            }
            FunctionKind::EffectMemberDefinition(qn) => {
                write!(f, "EffectMemberDefinition({})", qn)
            }
        }
    }
}

impl FunctionKind {
    pub fn isExternC(&self) -> bool {
        match self {
            FunctionKind::Extern(kind) => *kind == ExternKind::C,
            _ => false,
        }
    }

    pub fn isBuiltin(&self) -> bool {
        match self {
            FunctionKind::Extern(kind) => *kind == ExternKind::Builtin,
            _ => false,
        }
    }

    pub fn isCtor(&self) -> bool {
        match self {
            FunctionKind::VariantCtor(_) | FunctionKind::StructCtor => true,
            _ => false,
        }
    }

    pub fn isTraitCall(&self) -> bool {
        match self {
            FunctionKind::TraitMemberDecl(_) | FunctionKind::TraitMemberDefinition(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Body>,
    pub constraintContext: ConstraintContext,
    pub kind: FunctionKind,
}

impl Function {
    pub fn new(
        name: QualifiedName,
        params: Vec<Parameter>,
        result: Type,
        body: Option<Body>,
        constraintContext: ConstraintContext,
        kind: FunctionKind,
    ) -> Function {
        Function {
            name: name,
            params: params,
            result: result,
            body: body,
            constraintContext: constraintContext,
            kind: kind,
        }
    }

    pub fn isPure(&self) -> bool {
        match self.kind {
            FunctionKind::VariantCtor(_) | FunctionKind::StructCtor => true,
            _ => false,
        }
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        if let Some(body) = &self.body {
            body.getBlockById(id)
        } else {
            panic!("getBlockById: no body found");
        }
    }

    pub fn getFirstBlock(&self) -> &Block {
        if let Some(body) = &self.body {
            &body.blocks.get(&BlockId::first()).expect("Block not found")
        } else {
            panic!("getFirstBlock: no body found");
        }
    }

    pub fn getType(&self) -> Type {
        let mut args = Vec::new();
        for param in &self.params {
            match &param {
                Parameter::Named(_, ty, _) => args.push(ty.clone()),
                Parameter::SelfParam(_, ty) => args.push(ty.clone()),
            }
        }
        Type::Function(args, Box::new(self.result.clone()))
    }

    pub fn dump(&self) {
        println!("{}", self.name);
        match &self.body {
            Some(body) => body.dump(),
            None => println!("  <no body>"),
        }
    }

    pub fn dumpToFile(&self, name: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(name).map_err(|e| {
            eprintln!("Error creating file {}: {}", name, e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to create function file")
        })?;
        writeln!(file, "{}", self).map_err(|e| {
            eprintln!("Error writing to file {}: {}", name, e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to write function name")
        })?;
        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.name, self.getType())?;
        writeln!(f, "constraints {}", self.constraintContext)?;
        match &self.body {
            Some(body) => write!(f, "{}", body),
            None => write!(f, "  <no body>"),
        }
    }
}
