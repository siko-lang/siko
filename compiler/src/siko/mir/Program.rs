use std::{collections::BTreeMap, fmt};

use crate::siko::util::DependencyProcessor;

use super::{
    Data::{Field, Struct, Union},
    Function::Function,
    MiniCLowering::MinicBuilder,
    Type::Type,
};

use crate::siko::minic::Program::Program as CProgram;

pub struct Program {
    pub functions: BTreeMap<String, Function>,
    pub structs: BTreeMap<String, Struct>,
    pub unions: BTreeMap<String, Union>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Program:\n")?;
        write!(f, "\nFunctions:\n")?;
        for (_, function) in &self.functions {
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
            functions: BTreeMap::new(),
            structs: BTreeMap::new(),
            unions: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) {
        self.calculateSizeAndAlignment();

        self.convertUnions();
    }

    pub fn toMiniC(&self) -> CProgram {
        //println!("MIR before C gen {}", self);
        let mut builder = MinicBuilder::new(self);
        builder.lower()
    }

    pub fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("struct {} not found", n),
        }
    }

    pub fn getUnion(&self, n: &String) -> Union {
        match self.unions.get(n) {
            Some(u) => u.clone(),
            None => panic!("union {} not found", n),
        }
    }

    fn convertUnions(&mut self) {
        for (n, u) in &self.unions {
            let tag = Field {
                name: format!("tag"),
                ty: Type::Int32,
            };
            let itemSize = u.alignment * 8;
            let itemTy = match itemSize / 8 {
                1 => Type::Int8,
                2 => Type::Int16,
                4 => Type::Int32,
                8 => Type::Int64,
                _ => panic!("unsupported alignment {}", itemSize),
            };
            //println!("payloadsize {}", u.payloadSize);
            let payload = Field {
                name: format!("payload"),
                ty: Type::Array(Box::new(itemTy), u.payloadSize * 8 / itemSize),
            };
            let s = Struct {
                name: n.clone(),
                originalName: u.originalName.clone(),
                fields: vec![tag.clone(), payload],
                size: u.size,
                alignment: u.alignment,
            };
            //println!("{}: size: {} alignment {}", n, s.size, s.alignment);
            self.structs.insert(n.clone(), s);
            for v in &u.variants {
                let payload = Field {
                    name: format!("payload"),
                    ty: v.ty.clone(),
                };
                let s = Struct {
                    name: v.name.clone(),
                    originalName: format!("{}::{}", u.originalName, v.originalName),
                    fields: vec![tag.clone(), payload],
                    size: u.size,
                    alignment: u.alignment,
                };
                //println!("{}: size: {} alignment {}", v.name, s.size, s.alignment);
                self.structs.insert(v.name.clone(), s);
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
                    Type::Array(ty, _) => {
                        if let Type::Struct(n) = &**ty {
                            let deps = allDeps.get_mut(&s.name).expect("deps not found");
                            deps.push(n.clone());
                        }
                        if let Type::Union(n) = &**ty {
                            let deps = allDeps.get_mut(&s.name).expect("deps not found");
                            deps.push(n.clone());
                        }
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
                    Type::Array(ty, _) => {
                        if let Type::Struct(n) = &**ty {
                            let deps = allDeps.get_mut(&u.name).expect("deps not found");
                            deps.push(n.clone());
                        }
                        if let Type::Union(n) = &**ty {
                            let deps = allDeps.get_mut(&u.name).expect("deps not found");
                            deps.push(n.clone());
                        }
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
                    let size = self.getTypesize(&f.ty);
                    let alignment = self.getAlignment(&f.ty);
                    totalAlignment = std::cmp::max(totalAlignment, alignment);
                    offset += size;
                    let padding = (alignment - (offset % alignment)) % alignment;
                    offset += padding;
                }
                let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
                offset += padding;
                item.alignment = totalAlignment;
                item.size = offset;
                //println!("{} size: {}, alignment {}", item.name, item.size, item.alignment);
                self.structs.insert(item.name.clone(), item);
            }

            if self.unions.contains_key(itemName) {
                let mut item = self.getUnion(itemName);
                let mut offset = 4;
                let mut totalAlignment = 4;
                let mut maxSize = 0;
                for v in &item.variants {
                    let size = self.getTypesize(&v.ty);
                    let alignment = self.getAlignment(&v.ty);
                    totalAlignment = std::cmp::max(totalAlignment, alignment);
                    //println!("variant {} size {} alignment {}", v.name, size, alignment);
                    maxSize = std::cmp::max(maxSize, size);
                }
                offset += maxSize;
                let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
                offset += padding;
                item.alignment = totalAlignment;
                item.size = offset;
                item.payloadSize = maxSize;
                //println!("Union {} size: {}, alignment {}", item.name, item.size, item.alignment);
                self.unions.insert(item.name.clone(), item);
            }
        }
    }

    fn getAlignment(&mut self, ty: &Type) -> u32 {
        let alignment = match &ty {
            Type::VoidPtr => 8,
            Type::Void => 1,
            Type::UInt8 => 1,
            Type::UInt16 => 2,
            Type::UInt32 => 4,
            Type::UInt64 => 8,
            Type::Int8 => 1,
            Type::Int16 => 2,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::Struct(n) => self.getStruct(n).alignment,
            Type::Union(n) => self.getUnion(n).alignment,
            Type::Ptr(_) => 8,
            Type::Array(ty, _) => self.getAlignment(ty),
            Type::FunctionPtr(_, _) => 8,
        };
        alignment
    }

    fn getTypesize(&self, ty: &Type) -> u32 {
        match ty {
            Type::VoidPtr => 8,
            Type::Void => 0,
            Type::UInt8 => 1,
            Type::UInt16 => 2,
            Type::UInt32 => 4,
            Type::UInt64 => 8,
            Type::Int8 => 1,
            Type::Int16 => 2,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::Struct(n) => self.getStruct(n).size,
            Type::Union(n) => self.getUnion(n).size,
            Type::Ptr(_) => 8,
            Type::Array(item, size) => self.getTypesize(item) * *size,
            Type::FunctionPtr(_, _) => 8,
        }
    }
}
