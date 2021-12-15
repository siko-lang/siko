#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FilePath {
    pub path: String,
}

impl FilePath {
    pub fn new(path: String) -> FilePath {
        FilePath { path: path }
    }
}
