use crate::siko::ownership::TypeVariableInfo::TypeVariableInfo;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataFlowPath {
    pub index: u32,
    pub arg: TypeVariableInfo,
}
