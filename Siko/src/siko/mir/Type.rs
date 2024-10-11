use crate::siko::qualifiedname::QualifiedName;

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    I8,
    I32,
    Named(QualifiedName),
}
