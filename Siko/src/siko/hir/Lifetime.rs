use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lifetime {
    Named(i64),
}

impl Debug for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lifetime::Named(i) => {
                write!(f, "'l{}", i)
            }
        }
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifetimeInfo {
    pub args: Vec<Lifetime>,
}

impl LifetimeInfo {
    pub fn new() -> LifetimeInfo {
        LifetimeInfo { args: Vec::new() }
    }

    pub fn add(&mut self, l: Lifetime) {
        self.args.push(l);
    }

    pub fn allocate(&mut self) -> Lifetime {
        let next = self.args.len();
        let l = Lifetime::Named(next as i64);
        self.add(l);
        l
    }
}

impl Debug for LifetimeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for LifetimeInfo {
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
