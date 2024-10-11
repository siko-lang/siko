use crate::siko::qualifiedname::QualifiedName;

pub enum Type {
    Void,
    I8,
    I32,
    Named(QualifiedName),
}
