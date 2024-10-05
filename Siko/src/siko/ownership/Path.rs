use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Path {
    WholePath(String, bool),                // is ref
    PartialPath(String, Vec<String>, bool), // is ref
}

impl Path {
    pub fn getValue(&self) -> &String {
        match self {
            Path::WholePath(v, _) => v,
            Path::PartialPath(v, _, _) => v,
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Path::WholePath(v, isRef) => {
                write!(f, "WholePath({}{})", if *isRef { "&" } else { "" }, v)
            }
            Path::PartialPath(v, _, isRef) => {
                write!(f, "PartialPath({}{})", if *isRef { "&" } else { "" }, v)
            }
        }
    }
}
