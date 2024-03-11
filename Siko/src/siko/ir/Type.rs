use crate::siko::qualifiedname::QualifiedName;

pub enum Type {
    Named(QualifiedName, Vec<Type>),
}
