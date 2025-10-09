use std::fmt::Display;

use crate::siko::backend::path::SimplePath::SimplePath;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path {
    pub p: SimplePath,
}

impl Path {
    pub fn userVisible(&self) -> String {
        let mut s = self.p.root.visibleName();
        for item in &self.p.items {
            s.push('.');
            s.push_str(&item.to_string());
        }
        s
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p)
    }
}
