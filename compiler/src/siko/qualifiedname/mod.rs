use core::panic;
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::siko::{
    hir::Type::{formatTypes, formatTypesBracket, Type},
    monomorphizer::Context::Context,
};
pub mod builtins;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualifiedName {
    Module(Rc<String>),
    Item(Rc<QualifiedName>, Rc<String>),
    Monomorphized(Box<QualifiedName>, Context),
    Canonical(Box<QualifiedName>, Box<QualifiedName>, Vec<Type>),
    Lambda(Box<QualifiedName>, u32),
    Closure(Vec<Type>, Box<Type>),
    ClosureInstance(Box<QualifiedName>, u32),
    ClosureInstanceEnvStruct(Box<QualifiedName>),
    ClosureCallHandler(Box<QualifiedName>),
    Coroutine(Box<Type>, Box<Type>),                           // yielded, return
    CoroutineInstance(Box<QualifiedName>, Box<QualifiedName>), // coroutine name, state machine name
    CoroutineStateMachineEnum(Box<QualifiedName>),             // yielding function name
    CoroutineStateMachineVariant(Box<QualifiedName>, u32),     // yielding function name, entry index
    CoroutineStateMachineResume(Box<QualifiedName>),           // yielding function name
    CoroutineStateMachineIsCompleted(Box<QualifiedName>),      // yielding function name
    DefaultArgFn(Box<QualifiedName>, u32),                     // function name, argument index
}

impl QualifiedName {
    pub fn add(&self, item: String) -> QualifiedName {
        QualifiedName::Item(Rc::new(self.clone()), Rc::new(item))
    }

    pub fn canonical(&self, traitName: QualifiedName, types: Vec<Type>) -> QualifiedName {
        QualifiedName::Canonical(Box::new(self.clone()), Box::new(traitName), types)
    }

    pub fn module(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Item(p, _) => p.module(),
            QualifiedName::Monomorphized(p, _) => p.module(),
            QualifiedName::Canonical(p, _, _) => p.module(),
            QualifiedName::Lambda(p, _) => p.module(),
            QualifiedName::Closure(_, _) => panic!("Closure names are not supported"),
            QualifiedName::ClosureInstance(_, _) => panic!("ClosureInstance names are not supported"),
            QualifiedName::ClosureInstanceEnvStruct(_) => panic!("ClosureStruct names are not supported"),
            QualifiedName::ClosureCallHandler(_) => panic!("ClosureCallHandler names are not supported"),
            QualifiedName::Coroutine(_, _) => panic!("Coroutine names are not supported"),
            QualifiedName::CoroutineInstance(_, _) => panic!("CoroutineInstance names are not supported"),
            QualifiedName::CoroutineStateMachineEnum(_) => panic!("CoroutineStateMachine names are not supported"),
            QualifiedName::CoroutineStateMachineVariant(_, _) => {
                panic!("CoroutineStateMachineEntry names are not supported")
            }
            QualifiedName::CoroutineStateMachineResume(_) => {
                panic!("CoroutineStateMachineResume names are not supported")
            }
            QualifiedName::CoroutineStateMachineIsCompleted(_) => {
                panic!("CoroutineStateMachineIsCompleted names are not supported")
            }
            QualifiedName::DefaultArgFn(p, _) => p.module(),
        }
    }

    pub fn base(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Item(p, _) => (**p).clone(),
            QualifiedName::Monomorphized(p, _) => *p.clone(),
            QualifiedName::Canonical(p, _, _) => *p.clone(),
            QualifiedName::Lambda(p, _) => *p.clone(),
            QualifiedName::Closure(_, _) => panic!("Closure names are not supported"),
            QualifiedName::ClosureInstance(_, _) => panic!("ClosureInstance names are not supported"),
            QualifiedName::ClosureInstanceEnvStruct(_) => panic!("ClosureStruct names are not supported"),
            QualifiedName::ClosureCallHandler(_) => panic!("ClosureCallHandler names are not supported"),
            QualifiedName::Coroutine(_, _) => panic!("Coroutine names are not supported"),
            QualifiedName::CoroutineInstance(_, _) => panic!("CoroutineInstance names are not supported"),
            QualifiedName::CoroutineStateMachineEnum(_) => panic!("CoroutineStateMachine names are not supported"),
            QualifiedName::CoroutineStateMachineVariant(_, _) => {
                panic!("CoroutineStateMachineEntry names are not supported")
            }
            QualifiedName::CoroutineStateMachineResume(_) => {
                panic!("CoroutineStateMachineResume names are not supported")
            }
            QualifiedName::CoroutineStateMachineIsCompleted(_) => {
                panic!("CoroutineStateMachineIsCompleted names are not supported")
            }
            QualifiedName::DefaultArgFn(p, _) => *p.clone(),
        }
    }

    pub fn monomorphized(&self, context: Context) -> QualifiedName {
        match self {
            QualifiedName::Monomorphized(_, _) => {
                panic!("Cannot monomorphize a monomorphized name")
            }
            _ => QualifiedName::Monomorphized(Box::new(self.clone()), context),
        }
    }

    pub fn split(&self) -> (QualifiedName, Context) {
        match self {
            QualifiedName::Monomorphized(p, context) => (*p.clone(), context.clone()),
            p => (p.clone(), Context::new()),
        }
    }

    pub fn toString(&self) -> String {
        format!("{}", self)
    }

    pub fn getShortName(&self) -> String {
        match &self {
            QualifiedName::Module(name) => name.as_ref().clone(),
            QualifiedName::Item(_, name) => name.as_ref().clone(),
            QualifiedName::Monomorphized(p, _) => p.getShortName(),
            QualifiedName::Canonical(_, _, _) => {
                panic!("Canonical names are not supported")
            }
            QualifiedName::Lambda(_, _) => panic!("Lambda names are not supported"),
            QualifiedName::Closure(_, _) => panic!("Closure names are not supported"),
            QualifiedName::ClosureInstance(_, _) => panic!("ClosureInstance names are not supported"),
            QualifiedName::ClosureInstanceEnvStruct(_) => panic!("ClosureStruct names are not supported"),
            QualifiedName::ClosureCallHandler(_) => panic!("ClosureCallHandler names are not supported"),
            QualifiedName::Coroutine(_, _) => panic!("Coroutine names are not supported"),
            QualifiedName::CoroutineInstance(_, _) => panic!("CoroutineInstance names are not supported"),
            QualifiedName::CoroutineStateMachineEnum(_) => panic!("CoroutineStateMachine names are not supported"),
            QualifiedName::CoroutineStateMachineVariant(_, _) => {
                panic!("CoroutineStateMachineEntry names are not supported")
            }
            QualifiedName::CoroutineStateMachineResume(_) => {
                panic!("CoroutineStateMachineResume names are not supported")
            }
            QualifiedName::CoroutineStateMachineIsCompleted(_) => {
                panic!("CoroutineStateMachineIsCompleted names are not supported")
            }
            QualifiedName::DefaultArgFn(_, index) => format!("default_arg_{}", index),
        }
    }

    pub fn isMonomorphized(&self) -> bool {
        match self {
            QualifiedName::Monomorphized(_, _) => true,
            _ => false,
        }
    }

    pub fn getUnmonomorphized(&self) -> (QualifiedName, Option<Context>) {
        match self {
            QualifiedName::Monomorphized(p, context) => (*p.clone(), Some(context.clone())),
            n => (n.clone(), None),
        }
    }

    pub fn getCoroutineKey(&self) -> (Type, Type) {
        match self {
            QualifiedName::Coroutine(yielded, returnTy) => ((**yielded).clone(), (**returnTy).clone()),
            QualifiedName::CoroutineInstance(coroutineName, _) => coroutineName.getCoroutineKey(),
            _ => panic!("getCoroutineKey: not a coroutine"),
        }
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            QualifiedName::Module(i) => write!(f, "{}", i),
            QualifiedName::Item(p, i) => write!(f, "{}.{}", p, i),
            QualifiedName::Monomorphized(p, context) => {
                write!(f, "{}#{}", p, context)
            }
            QualifiedName::Canonical(p, t, types) => {
                write!(f, "{}/{}{}", p, t, formatTypesBracket(types))
            }
            QualifiedName::Lambda(p, index) => write!(f, "{}.lambda/{}", p, index),
            QualifiedName::Closure(params, result) => {
                write!(f, "closure({} -> {})", formatTypes(params), result)
            }
            QualifiedName::ClosureInstance(p, index) => write!(f, "{}.closure_instance/{}", p, index),
            QualifiedName::ClosureInstanceEnvStruct(p) => write!(f, "{}.closure_env", p),
            QualifiedName::ClosureCallHandler(p) => write!(f, "{}.closure_call_handler", p),
            QualifiedName::Coroutine(yielded, returnTy) => {
                write!(f, "coroutine(({}) -> {})", yielded, returnTy)
            }
            QualifiedName::CoroutineInstance(coroutineName, stateMachineName) => {
                write!(f, "{}.instance/{}", coroutineName, stateMachineName)
            }
            QualifiedName::CoroutineStateMachineEnum(p) => write!(f, "{}.coroutine_state_machine", p),
            QualifiedName::CoroutineStateMachineVariant(p, index) => {
                write!(f, "{}.coroutine_state_machine_variant/{}", p, index)
            }
            QualifiedName::CoroutineStateMachineResume(p) => write!(f, "{}.coroutine_resume", p),
            QualifiedName::CoroutineStateMachineIsCompleted(p) => write!(f, "{}.coroutine_is_completed", p),
            QualifiedName::DefaultArgFn(p, index) => write!(f, "{}.default_arg_{}", p, index),
        }
    }
}

impl Debug for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn build(m: &str, name: &str) -> QualifiedName {
    QualifiedName::Item(
        Rc::new(QualifiedName::Module(Rc::new(m.to_string()))),
        Rc::new(name.to_string()),
    )
}

pub fn buildModule(m: &str) -> QualifiedName {
    QualifiedName::Module(Rc::new(m.to_string()))
}
