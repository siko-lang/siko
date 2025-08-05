use super::Type::Type;

#[derive(Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    pub size: u32,
    pub alignment: u32,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}
