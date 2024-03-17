use super::{super::Signature::FunctionOwnershipSignature, DataFlowPath::DataFlowPath};

#[derive(Debug, PartialEq, Eq)]
pub struct DataFlowProfile {
    pub paths: Vec<DataFlowPath>,
    pub signature: FunctionOwnershipSignature,
}

impl DataFlowProfile {
    pub fn new() -> DataFlowProfile {
        DataFlowProfile {
            paths: Vec::new(),
            signature: FunctionOwnershipSignature::new(),
        }
    }
}
