#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum BuiltinOperator {
    Add,
    Sub,
    Mul,
    Div,
    PipeForward,
    And,
    Or,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessOrEqualThan,
    GreaterOrEqualThan,
    Not,
    Minus,
    Bind,
    Arrow,
}

impl BuiltinOperator {
    pub fn get_function_name(&self) -> String {
        match self {
            BuiltinOperator::Add => format!("Std.Ops.opAdd"),
            BuiltinOperator::Sub => format!("Std.Ops.opSub"),
            BuiltinOperator::Mul => format!("Std.Ops.opMul"),
            BuiltinOperator::Div => format!("Std.Ops.opDiv"),
            BuiltinOperator::Equals => format!("Std.Ops.opEq"),
            BuiltinOperator::NotEquals => format!("Std.Ops.opNotEq"),
            BuiltinOperator::LessThan => format!("Std.Ops.opLessThan"),
            BuiltinOperator::LessOrEqualThan => format!("Std.Ops.opLessEqual"),
            BuiltinOperator::GreaterThan => format!("Std.Ops.opGreaterThan"),
            BuiltinOperator::GreaterOrEqualThan => format!("Std.Ops.opGreaterEqual"),
            BuiltinOperator::And => format!("Std.Ops.opAnd"),
            BuiltinOperator::Or => format!("Std.Ops.opOr"),
            BuiltinOperator::Not => format!("Std.Ops.opNot"),
            _ => panic!("Op {:?} has no func name", self),
        }
    }
}

pub const MAIN_MODULE_NAME: &str = "Main";
pub const MAIN_FUNCTION: &str = "main";
pub const BOOL_MODULE_NAME: &str = "Bool";
pub const BOOL_TYPE_NAME: &str = "Bool";
pub const INT_MODULE_NAME: &str = "Int";
pub const INT_TYPE_NAME: &str = "Int";
pub const FLOAT_MODULE_NAME: &str = "Float";
pub const FLOAT_TYPE_NAME: &str = "Float";
pub const CHAR_MODULE_NAME: &str = "Char";
pub const CHAR_TYPE_NAME: &str = "Char";
pub const OPTION_MODULE_NAME: &str = "Option";
pub const OPTION_TYPE_NAME: &str = "Option";
pub const RESULT_MODULE_NAME: &str = "Result";
pub const RESULT_TYPE_NAME: &str = "Result";
pub const MAP_MODULE_NAME: &str = "Map";
pub const MAP_TYPE_NAME: &str = "Map";
pub const ORDERING_MODULE_NAME: &str = "Ordering";
pub const ORDERING_TYPE_NAME: &str = "Ordering";
pub const STRING_MODULE_NAME: &str = "String";
pub const STRING_TYPE_NAME: &str = "String";
pub const LIST_MODULE_NAME: &str = "List";
pub const LIST_TYPE_NAME: &str = "List";
pub const ITERATOR_MODULE_NAME: &str = "Iterator";
pub const TRUE_NAME: &str = "True";
pub const FALSE_NAME: &str = "False";
pub const SOME_NAME: &str = "Some";
pub const NONE_NAME: &str = "None";
pub const EQUAL_NAME: &str = "Equal";
pub const LESS_NAME: &str = "Less";
pub const GREATER_NAME: &str = "Greater";
pub const SHOW_CLASS_NAME: &str = "Show";
pub const PARTIALEQ_CLASS_NAME: &str = "PartialEq";
pub const EQ_CLASS_NAME: &str = "Eq";
pub const PARTIALORD_CLASS_NAME: &str = "PartialOrd";
pub const PARTIALEQ_OP_NAME: &str = "opEq";
pub const ORD_CLASS_NAME: &str = "Ord";
pub const ORD_OP_NAME: &str = "cmp";
pub const STD_OPS_MODULE_NAME: &str = "Std.Ops";
pub const STD_UTIL_BASIC_MODULE_NAME: &str = "Std.Util.Basic";
pub const MIR_INTERNAL_MODULE_NAME: &str = "__siko__";
pub const MIR_FUNCTION_TRAIT_NAME: &str = "Function";

pub fn get_qualified_list_type_name() -> String {
    format!("{}.{}", LIST_MODULE_NAME, LIST_TYPE_NAME)
}

pub fn get_implicit_module_list() -> Vec<&'static str> {
    let implicit_modules = vec![
        INT_MODULE_NAME,
        FLOAT_MODULE_NAME,
        STRING_MODULE_NAME,
        BOOL_MODULE_NAME,
        ORDERING_MODULE_NAME,
        OPTION_MODULE_NAME,
        RESULT_MODULE_NAME,
        LIST_MODULE_NAME,
        ITERATOR_MODULE_NAME,
        STD_UTIL_BASIC_MODULE_NAME,
        STD_OPS_MODULE_NAME,
        CHAR_MODULE_NAME,
    ];
    implicit_modules
}

pub fn get_auto_derivable_classes() -> Vec<&'static str> {
    vec![
        PARTIALEQ_CLASS_NAME,
        EQ_CLASS_NAME,
        PARTIALORD_CLASS_NAME,
        ORD_CLASS_NAME,
        SHOW_CLASS_NAME,
    ]
}
