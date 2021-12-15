use siko_location_info::location_id::LocationId;

#[derive(Debug)]
pub enum TypecheckError {
    ConflictingInstances(String, LocationId, LocationId),
    DeriveFailureNoInstanceFound(String, String, LocationId),
    DeriveFailureInstanceNotGeneric(String, String, LocationId),
    UntypedExternFunction(String, LocationId),
    FunctionArgAndSignatureMismatch(String, usize, usize, LocationId, bool),
    MainNotFound,
    IncorrectTypeForMain(String, LocationId),
    TypeMismatch(LocationId, String, String),
    FunctionArgumentMismatch(LocationId, String, String),
    InvalidVariantPattern(LocationId, String, usize, usize),
    InvalidRecordPattern(LocationId, String, usize, usize),
    TypeAnnotationNeeded(LocationId),
    InvalidFormatString(LocationId),
    CyclicClassDependencies(LocationId, String),
    MissingInstance(String, LocationId),
    ClassNotAutoDerivable(String, LocationId),
    UnreachablePattern(LocationId),
    NonExhaustivePattern(LocationId),
}

#[derive(Debug)]
pub struct Error {
    pub errors: Vec<TypecheckError>,
}

impl Error {
    pub fn typecheck_err(errors: Vec<TypecheckError>) -> Error {
        Error { errors: errors }
    }
}
