use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Path {
    WholePath(String),
    PartialPath(String, Vec<String>),
}

impl Path {
    pub fn getValue(&self) -> &String {
        match self {
            Path::WholePath(v) => v,
            Path::PartialPath(v, _) => v,
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Path::WholePath(v) => write!(f, "WholePath({})", v),
            Path::PartialPath(v, _) => write!(f, "PartialPath({})", v),
        }
    }
}
