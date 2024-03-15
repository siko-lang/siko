#[derive(Debug)]
pub enum Path {
    WholePath(String),
    PartialPath(String, Vec<String>),
}
