use crate::data::TypeDef;
use crate::data::TypeDefId;
use crate::function::FunctionId;
use crate::program::Program;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    Named(TypeDefId),
    Function(Box<Type>, Box<Type>),
    Closure(Box<Type>),
    Boxed(Box<Type>),
    Ref(Box<Type>),
}

impl Type {
    pub fn get_args(&self, args: &mut Vec<Type>) {
        match self {
            Type::Named(..) => {}
            Type::Function(from, to) => {
                args.push(*from.clone());
                to.get_args(args);
            }
            Type::Closure(ty) => ty.get_args(args),
            Type::Boxed(ty) => ty.get_args(args),
            Type::Ref(ty) => ty.get_args(args),
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Type::Function(..) => true,
            _ => false,
        }
    }

    pub fn is_boxed(&self) -> bool {
        match self {
            Type::Boxed(ty) => true,
            _ => false,
        }
    }

    pub fn get_result_type(&self, arg_count: usize) -> Type {
        match self {
            Type::Named(..) => self.clone(),
            Type::Function(_, to) => {
                if arg_count == 1 {
                    *to.clone()
                } else {
                    if arg_count == 0 {
                        self.clone()
                    } else {
                        to.get_result_type(arg_count - 1)
                    }
                }
            }
            Type::Closure(ty) => {
                if arg_count == 0 {
                    self.clone()
                } else {
                    ty.get_result_type(arg_count)
                }
            }
            Type::Boxed(ty) => ty.get_result_type(arg_count),
            Type::Ref(..) => self.clone(),
        }
    }

    pub fn get_typedef_id_opt(&self) -> Option<TypeDefId> {
        match self {
            Type::Named(id) => Some(*id),
            Type::Function(_, _) => None,
            Type::Closure(..) => None,
            Type::Boxed(ty) => ty.get_typedef_id_opt(),
            Type::Ref(..) => None,
        }
    }

    pub fn get_typedef_id(&self) -> TypeDefId {
        match self {
            Type::Named(id) => *id,
            Type::Function(_, _) => unreachable!(),
            Type::Closure(ty) => ty.get_typedef_id(),
            Type::Boxed(ty) => ty.get_typedef_id(),
            Type::Ref(..) => unreachable!(),
        }
    }

    pub fn get_from_to(&self) -> (Type, Type) {
        match self {
            Type::Function(from, to) => (*from.clone(), *to.clone()),
            Type::Named(_) => unreachable!(),
            Type::Closure(ty) => ty.get_from_to(),
            Type::Boxed(ty) => ty.get_from_to(),
            Type::Ref(..) => unreachable!(),
        }
    }

    pub fn to_string(&self, program: &Program) -> String {
        match self {
            Type::Function(from, to) => {
                let from = from.to_string(program);
                let to = to.to_string(program);
                format!("{} -> {}", from, to)
            }
            Type::Named(id) => {
                let typedef = program.typedefs.get(id);
                match typedef {
                    TypeDef::Adt(adt) => format!("{}.{}", adt.module, adt.name),
                    TypeDef::Record(record) => format!("{}.{}", &record.module, record.name),
                }
            }
            Type::Closure(ty) => {
                let closure = program.get_closure_type(ty);
                closure.get_name()
            }
            Type::Boxed(ty) => format!("Boxed({})", ty.to_string(program)),
            Type::Ref(item) => format!("&{}", item.to_string(program)),
        }
    }
}

pub struct Closure {
    pub name: String,
    pub ty: Type,
    pub from_ty: Type,
    pub to_ty: Type,
}

impl Closure {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub enum DynamicCallTrait {
    RealCall {
        from: Type,
        to: Type,
    },
    ArgSave {
        from: Type,
        to: Type,
        field_index: usize,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialFunctionCallId {
    pub id: usize,
}

impl fmt::Display for PartialFunctionCallId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.id)
    }
}

impl From<usize> for PartialFunctionCallId {
    fn from(id: usize) -> PartialFunctionCallId {
        PartialFunctionCallId { id: id }
    }
}

pub struct PartialFunctionCallField {
    pub ty: Type,
}

pub struct PartialFunctionCall {
    pub id: PartialFunctionCallId,
    pub fields: Vec<PartialFunctionCallField>,
    pub traits: Vec<DynamicCallTrait>,
    pub function: FunctionId,
    pub closure_type: Type,
}

impl PartialFunctionCall {
    pub fn get_name(&self) -> String {
        format!("PartialFunctionCall{}", self.id.id)
    }
}
