use super::Identifier::Identifier;

pub enum Type {
    Named(Identifier, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
}
