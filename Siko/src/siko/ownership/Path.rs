use std::fmt::Display;

#[derive(Debug)]
pub enum Path {
    WholePath(String),
    PartialPath(String, Vec<String>),
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Path::WholePath(v) => write!(f, "WholePath({})", v),
            Path::PartialPath(v, _) => write!(f, "PartialPath({})", v),
        }
    }
}
