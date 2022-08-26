use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::value::ValueCore;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct ReadTextFile {}

impl ExternFunction for ReadTextFile {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let path = environment.get_arg_by_index(0).core.as_string();
        let content = std::fs::read(&path).expect("ReadTextFile failed");
        let content = String::from_utf8_lossy(&content).to_string();
        return Value::new(ValueCore::String(content), ty);
    }
}

pub struct Investigate {}

impl ExternFunction for Investigate {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0);
        match &*value.core {
            ValueCore::Record(id, _items) => {
                println!("Record ! {:?}", id);
            }
            ValueCore::Variant(id, _, _items) => {
                println!("Variant ! {:?}", id);
            }
            _ => {
                println!("Something else ");
            }
        }
        return Value::new(ValueCore::Tuple(Vec::new()), ty);
    }
}

pub struct WriteTextFile {}

impl ExternFunction for WriteTextFile {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let path = environment.get_arg_by_index(0).core.as_string();
        let content = environment.get_arg_by_index(1).core.as_string();
        std::fs::write(&path, content).expect("WriteTextFile failed");
        return Value::new(ValueCore::Tuple(vec![]), ty);
    }
}

pub struct GetArgs {}

impl ExternFunction for GetArgs {
    fn call(&self, _: &Environment, _: Option<ExprId>, _: &NamedFunctionKind, ty: Type) -> Value {
        let string_ty = Interpreter::get_string_type();
        let mut args = vec![Value::new(
            ValueCore::String("siko".to_string()),
            string_ty.clone(),
        )];
        let mut after = false;
        for arg in std::env::args() {
            if after {
                let v = Value::new(ValueCore::String(arg), string_ty.clone());
                args.push(v);
            } else {
                if arg == "--" {
                    after = true;
                }
            }
        }
        return Value::new(ValueCore::List(args.into_iter().collect()), ty);
    }
}

pub struct ListDir {}

impl ExternFunction for ListDir {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let path = environment.get_arg_by_index(0).core.as_string();
        let string_ty = Interpreter::get_string_type();
        let mut args = vec![];
        for entry in std::fs::read_dir(&path).expect("read_dir failed") {
            let path = entry
                .expect("readdir entry failed")
                .path()
                .to_string_lossy()
                .into_owned();
            let v = Value::new(ValueCore::String(path), string_ty.clone());
            args.push(v);
        }
        return Value::new(ValueCore::List(args.into_iter().collect()), ty);
    }
}

pub struct IsDir {}

impl ExternFunction for IsDir {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let path = environment.get_arg_by_index(0).core.as_string();
        return Interpreter::get_bool_value(std::path::Path::new(&path).is_dir());
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function("Hack", "readTextFile", Box::new(ReadTextFile {}));
    interpreter.add_extern_function("Hack", "writeTextFile", Box::new(WriteTextFile {}));
    interpreter.add_extern_function("Hack", "getArgs", Box::new(GetArgs {}));
    interpreter.add_extern_function("Hack", "listDir", Box::new(ListDir {}));
    interpreter.add_extern_function("Hack", "isDir", Box::new(IsDir {}));
    interpreter.add_extern_function("Hack", "investigate", Box::new(Investigate {}));
}
