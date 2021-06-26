use serde_json::{Map, Result, Value};

use crate::mir::*;

fn parse_adt(adt: &Value) -> Adt {
    let adt = adt.as_object().expect("Adt is not an object");
    let name = adt
        .get("name")
        .expect("Adt does not have a name")
        .as_str()
        .expect("Name is not a str");
    let variants = adt
        .get("variants")
        .expect("variants not found")
        .as_array()
        .expect("variants is not a list");
    let mut vs = Vec::new();
    for v in variants {
        let v = v.as_object().expect("variant is not an object");
        let variant_name = v
            .get("name")
            .expect("variant does not have a name")
            .as_str()
            .expect("Name is not a str");
        //println!("name :{}", variant_name);
        let variant_type = v
            .get("type")
            .expect("variant does not have a type")
            .as_str()
            .expect("type is not a str");
        //println!("type :{}", variant_type);
        vs.push(Variant {
            name: variant_name.to_string(),
            ty: variant_type.to_string(),
        });
    }
    //println!("{} has {} variants", name, variants.len());
    Adt {
        name: name.to_string(),
        variants: vs,
    }
}

fn parse_record(record: &Value) -> Record {
    let record = record.as_object().expect("Record is not an object");
    let name = record
        .get("name")
        .expect("Record does not have a name")
        .as_str()
        .expect("Name is not a str");
    let fields = record
        .get("fields")
        .expect("fields not found")
        .as_array()
        .expect("fields is not a list");
    let externals = match record.get("externals") {
        Some(externals) => {
            let externals = externals.as_array().expect("Externals is not a list");
            let externals: Vec<_> = externals
                .iter()
                .map(|e| e.as_str().expect("External is not a str").to_string())
                .collect();
            Some(externals)
        }
        None => None,
    };
    let mut fs = Vec::new();
    for f in fields {
        let f = f.as_object().expect("field is not an object");
        let field_name = f
            .get("name")
            .expect("field does not have a name")
            .as_str()
            .expect("Name is not a str");
        //println!("name :{}", field_name);
        let field_type = f
            .get("type")
            .expect("field does not have a type")
            .as_str()
            .expect("type is not a str");
        //println!("type :{}", field_type);
        fs.push(Field {
            name: field_name.to_string(),
            ty: field_type.to_string(),
        });
    }
    //println!("{} has {} fields", name, fields.len());
    Record {
        name: name.to_string(),
        fields: fs,
        externals: externals,
    }
}

fn parse_args(v: &Map<String, Value>) -> Vec<i64> {
    let args = v
        .get("args")
        .expect("args not found")
        .as_array()
        .expect("args not a list");
    args.iter()
        .map(|arg| {
            arg.as_str()
                .expect("arg is not a str")
                .parse()
                .expect("arg is not i64")
        })
        .collect()
}

fn parse_value(v: &Map<String, Value>, name: &str) -> String {
    v.get(name)
        .expect("value not found")
        .as_str()
        .expect("value not a string")
        .to_string()
}

fn parse_expr(expr: &Value) -> Expr {
    let expr = expr.as_object().expect("Expr is not an object");
    let ty = expr
        .get("type")
        .expect("Expr does not have a type")
        .as_str()
        .expect("Type is not a str");
    let id = expr
        .get("id")
        .expect("Expr does not have a id")
        .as_str()
        .expect("id is not a str");
    let kind = expr
        .get("kind")
        .expect("Expr does not have a kind")
        .as_str()
        .expect("kind is not a str");
    let kind = match kind {
        "do" => {
            let subs = parse_args(expr);
            ExprKind::Do(subs)
        }
        "staticfunctioncall" => {
            let f_id = parse_value(expr, "f_id");
            if expr.get("args").is_some() {
                let subs = parse_args(expr);
                ExprKind::StaticFunctionCall(f_id, subs)
            } else {
                ExprKind::StaticFunctionCall(f_id, Vec::new())
            }
        }
        "integerliteral" => {
            let value = parse_value(expr, "value");
            ExprKind::IntegerLiteral(value)
        }
        "stringliteral" => {
            let value = parse_value(expr, "value");
            ExprKind::StringLiteral(value)
        }
        "floatliteral" => {
            let value = parse_value(expr, "value");
            ExprKind::FloatLiteral(value)
        }
        "charliteral" => {
            let value = parse_value(expr, "value");
            ExprKind::CharLiteral(value)
        }
        "vardecl" => {
            let subs = parse_args(expr);
            let var = parse_value(expr, "var");
            ExprKind::VarDecl(var, subs[0])
        }
        "varref" => {
            let var = parse_value(expr, "var");
            ExprKind::VarRef(var)
        }
        "fieldaccess" => {
            let subs = parse_args(expr);
            let field = parse_value(expr, "field");
            ExprKind::FieldAccess(field, subs[0])
        }
        "if" => {
            let subs = parse_args(expr);
            ExprKind::If(subs[0], subs[1], subs[2])
        }
        "list" => {
            if expr.get("args").is_some() {
                let subs = parse_args(expr);
                ExprKind::List(subs)
            } else {
                ExprKind::List(Vec::new())
            }
        }
        "return" => {
            let subs = parse_args(expr);
            ExprKind::Return(subs[0])
        }
        "continue" => {
            let subs = parse_args(expr);
            ExprKind::Continue(subs[0])
        }
        "break" => {
            let subs = parse_args(expr);
            ExprKind::Break(subs[0])
        }
        "loop" => {
            let subs = parse_args(expr);
            let var = parse_value(expr, "var");
            ExprKind::Loop(var, subs[0], subs[1])
        }
        "caseof" => {
            let subs = parse_args(expr);
            let cases = expr
                .get("cases")
                .expect("cases not found")
                .as_array()
                .expect("cases not an array");
            let mut cs = Vec::new();
            for c in cases {
                let checker = c
                    .get("checker")
                    .expect("checker not found")
                    .as_str()
                    .expect("checker not a str")
                    .to_string();
                let body = c
                    .get("body")
                    .expect("case body not found")
                    .as_str()
                    .expect("case body not a str")
                    .parse()
                    .expect("case body not i64");
                let c = Case {
                    checker: checker,
                    body: body,
                };
                cs.push(c);
            }
            ExprKind::CaseOf(subs[0], cs)
        }
        "converter" => {
            let subs = parse_args(expr);
            ExprKind::Converter(subs[0])
        }
        e => {
            panic!("Kind {} not expected", e);
        }
    };
    Expr {
        id: id.to_string(),
        ty: ty.to_string(),
        kind: kind,
        type_args: Vec::new(),
    }
}

fn parse_function(function: &Value) -> Function {
    let function = function.as_object().expect("Function is not an object");
    let name = function
        .get("name")
        .expect("Function does not have a name")
        .as_str()
        .expect("Name is not a str");
    let kind = function
        .get("kind")
        .expect("Function does not have a kind")
        .as_str()
        .expect("Kind is not a str");
    let result = function
        .get("result")
        .expect("Function does not have a result")
        .as_str()
        .expect("Result is not a str");
    let args = function
        .get("args")
        .expect("Function does not have args")
        .as_array()
        .expect("Args is not a list");
    let mut vargs = Vec::new();
    for arg in args {
        vargs.push(
            arg.as_str()
                .expect("Function arg is not a string")
                .to_string(),
        );
    }
    let kind = match kind {
        "normal" => {
            let body = function.get("body").expect("Normal function has no body");
            let mut exprs = Vec::new();
            match body {
                Value::Object(_) => {
                    let e = parse_expr(body);
                    exprs.push(e);
                }
                Value::Array(body) => {
                    for e in body {
                        let e = parse_expr(e);
                        exprs.push(e);
                    }
                }
                _ => panic!("Body is not a single item, nor a list"),
            }

            FunctionKind::Normal(exprs)
        }
        "variant" => {
            let index: i64 = function
                .get("index")
                .expect("variant has no index")
                .as_str()
                .expect("variant index is not str")
                .parse()
                .expect("index is not i64");
            FunctionKind::VariantCtor(index)
        }
        "record" => FunctionKind::RecordCtor,
        "extern" => FunctionKind::External,
        e => panic!("Unexpected function kind {}", e),
    };
    Function {
        name: name.to_string(),
        args: vargs,
        result: result.to_string(),
        kind: kind,
    }
}

pub fn load_mir(f: String) -> Result<Program> {
    let data: String =
        String::from_utf8_lossy(&std::fs::read(f).expect("File read failed")).to_string();
    let v: Value = serde_json::from_str(&data)?;
    let mir = v.as_object().expect("MIR is not an object");
    let adts = mir
        .get("adts")
        .expect("adts not found")
        .as_array()
        .expect("Adts is not a list");
    let records = mir
        .get("records")
        .expect("records not found")
        .as_array()
        .expect("Records is not a list");
    let functions = mir
        .get("functions")
        .expect("functions not found")
        .as_array()
        .expect("Functions is not a list");
    // println!(
    //     "adts: {}, records: {}, functions: {}",
    //     adts.len(),
    //     records.len(),
    //     functions.len()
    // );
    let mut program = Program::new();
    for v in adts {
        let adt = parse_adt(v);
        program.data.insert(adt.name.clone(), Data::Adt(adt));
    }
    for v in records {
        let record = parse_record(v);
        program
            .data
            .insert(record.name.clone(), Data::Record(record));
    }
    for v in functions {
        let function = parse_function(v);
        program.functions.insert(function.name.clone(), function);
    }
    Ok(program)
}
