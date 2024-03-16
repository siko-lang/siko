use super::{DataFlowPath::DataFlowPath, Signature::FunctionOwnershipSignature};

#[derive(Debug, PartialEq, Eq)]
pub struct DataFlowProfile {
    pub paths: Vec<DataFlowPath>,
    pub signature: FunctionOwnershipSignature,
}
