use std::{collections::BTreeMap, fmt};

use crate::siko::util::DependencyProcessor;

use super::{
    Data::{Field, Struct, Union},
    Function::Function,
    Lowering::Builder,
    Type::Type,
};

use crate::siko::llvm::Program::Program as LProgram;

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
    pub unions: BTreeMap<String, Union>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Program:\n")?;
        write!(f, "\nFunctions:\n")?;
        for function in &self.functions {
            write!(f, "{}\n", function)?;
        }
        write!(f, "\nStructs:\n")?;
        for (_, s) in &self.structs {
            write!(f, "{}\n", s)?;
        }
        write!(f, "\nUnions:\n")?;
        for (_, u) in &self.unions {
            write!(f, "{}\n", u)?;
        }
        Ok(())
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
            unions: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) -> LProgram {
        self.calculateSizeAndAlignment();

        self.convertUnions();

        let mut builder = Builder::new(self);
        builder.lower()
    }

    pub fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("struct {} not found", n),
        }
    }

    pub fn getUnion(&self, n: &String) -> Union {
        self.unions.get(n).cloned().expect("union not found")
    }

    fn convertUnions(&mut self) {
        for (n, u) in &self.unions {
            let tag = Field {
                name: format!("tag"),
                ty: Type::Int32,
            };
            let payload = Field {
                name: format!("payload"),
                ty: Type::ByteArray(u.payloadSize),
            };
            let s = Struct {
                name: n.clone(),
                fields: vec![tag, payload],
                size: u.size,
                alignment: u.alignment,
            };
            self.structs.insert(n.clone(), s);
            for v in &u.variants {
                let mut variantStruct = if let Type::Struct(vName) = &v.ty {
                    self.getStruct(vName)
                } else {
                    unreachable!()
                };
                variantStruct.name = v.name.clone();
                self.structs.insert(v.name.clone(), variantStruct);
            }
        }
    }

    fn calculateSizeAndAlignment(&mut self) {
        let mut allDeps = BTreeMap::new();

        for (_, s) in &self.structs {
            allDeps.insert(s.name.clone(), Vec::new());
        }

        for (_, u) in &self.unions {
            allDeps.insert(u.name.clone(), Vec::new());
        }

        for (_, s) in &self.structs {
            for f in &s.fields {
                match &f.ty {
                    Type::Struct(n) => {
                        let deps = allDeps.get_mut(&s.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    Type::Union(n) => {
                        let deps = allDeps.get_mut(&s.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    _ => {}
                }
            }
        }

        for (_, u) in &self.unions {
            for v in &u.variants {
                match &v.ty {
                    Type::Struct(n) => {
                        let deps = allDeps.get_mut(&u.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    Type::Union(n) => {
                        let deps = allDeps.get_mut(&u.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    _ => {}
                }
            }
        }

        let groups = DependencyProcessor::processDependencies(&allDeps);

        for group in &groups {
            if group.items.len() != 1 {
                panic!("Minic: cyclic data dependency {:?}", groups);
            }
            let itemName = &group.items[0];
            if self.structs.contains_key(itemName) {
                let mut item = self.getStruct(itemName);
                let mut offset = 0;
                let mut totalAlignment = 4;
                for f in &item.fields {
                    let size = match &f.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).size,
                        Type::Union(n) => self.getUnion(n).size,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(s) => *s,
                    };
                    let alignment = match &f.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).alignment,
                        Type::Union(n) => self.getUnion(n).alignment,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(_) => 1,
                    };
                    totalAlignment = std::cmp::max(totalAlignment, alignment);
                    offset += size;
                    let padding = (alignment - (offset % alignment)) % alignment;
                    offset += padding;
                }
                let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
                offset += padding;
                item.alignment = totalAlignment;
                item.size = offset;
                self.structs.insert(item.name.clone(), item);
            }

            if self.unions.contains_key(itemName) {
                let mut item = self.getUnion(itemName);
                let mut offset = 4;
                let mut totalAlignment = 4;
                let mut maxSize = 0;
                for v in &item.variants {
                    let size = match &v.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).size,
                        Type::Union(n) => self.getUnion(n).size,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(s) => *s,
                    };
                    let alignment = match &v.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).alignment,
                        Type::Union(n) => self.getUnion(n).alignment,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(_) => 1,
                    };
                    totalAlignment = std::cmp::max(totalAlignment, alignment);
                    maxSize = std::cmp::max(maxSize, size);
                }
                offset += maxSize;
                let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
                offset += padding;
                item.alignment = totalAlignment;
                item.size = offset;
                item.payloadSize = maxSize;
                self.unions.insert(item.name.clone(), item);
            }
        }
    }
}
