use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnershipVar {
    value: i64,
}

impl Debug for OwnershipVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#o{}", self.value)
    }
}

impl Display for OwnershipVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnershipVarInfo {
    pub args: Vec<OwnershipVar>,
}

impl OwnershipVarInfo {
    pub fn new() -> OwnershipVarInfo {
        OwnershipVarInfo { args: Vec::new() }
    }

    pub fn add(&mut self, l: OwnershipVar) {
        self.args.push(l);
    }

    pub fn allocate(&mut self) -> OwnershipVar {
        let next = self.args.len();
        let l = OwnershipVar { value: next as i64 };
        self.add(l);
        l
    }
}

impl Debug for OwnershipVarInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for OwnershipVarInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            Ok(())
        } else {
            write!(f, "[")?;
            for (index, a) in self.args.iter().enumerate() {
                if index == 0 {
                    write!(f, "{}", a)?;
                } else {
                    write!(f, ", {}", a)?;
                }
            }
            write!(f, "]")?;
            Ok(())
        }
    }
}
