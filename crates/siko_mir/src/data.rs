use crate::types::Type;
use std::fmt;

#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum ExternalDataKind {
    Int,
    String,
    Float,
    Char,
    List,
    Map,
    Iterator,
}

#[derive(Debug, Clone)]
pub enum RecordKind {
    Normal,
    External(ExternalDataKind, Vec<Type>),
}

#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub module: String,
    pub id: TypeDefId,
    pub fields: Vec<RecordField>,
    pub kind: RecordKind,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub items: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Adt {
    pub name: String,
    pub module: String,
    pub id: TypeDefId,
    pub variants: Vec<Variant>,
}

impl Adt {
    pub fn get_variant_index(&self, name: &str) -> usize {
        for (index, variant) in self.variants.iter().enumerate() {
            if variant.name == name {
                return index;
            }
        }
        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub enum TypeDef {
    Record(Record),
    Adt(Adt),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TypeDefId {
    pub id: usize,
}

impl fmt::Display for TypeDefId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TypeDefId({})", self.id)
    }
}

impl From<usize> for TypeDefId {
    fn from(id: usize) -> TypeDefId {
        TypeDefId { id: id }
    }
}

impl TypeDef {
    pub fn get_adt(&self) -> &Adt {
        if let TypeDef::Adt(adt) = self {
            &adt
        } else {
            unreachable!()
        }
    }

    pub fn get_record(&self) -> &Record {
        if let TypeDef::Record(record) = self {
            &record
        } else {
            unreachable!()
        }
    }

    pub fn get_mut_adt(&mut self) -> &mut Adt {
        if let TypeDef::Adt(ref mut adt) = self {
            adt
        } else {
            unreachable!()
        }
    }

    pub fn get_mut_record(&mut self) -> &mut Record {
        if let TypeDef::Record(ref mut record) = self {
            record
        } else {
            unreachable!()
        }
    }
}
