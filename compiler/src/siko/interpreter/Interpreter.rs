use crate::siko::{
    hir::{Block::BlockId, Body::Body, Function::FunctionKind, Instruction::InstructionKind, Program::Program},
    interpreter::{
        Frame::Frame,
        Value::{EnumValue, StructValue, Value},
    },
    qualifiedname::QualifiedName,
};

pub struct Interpreter<'a> {
    program: &'a Program,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program) -> Self {
        Interpreter { program }
    }

    pub fn executeFunction(&self, name: &QualifiedName, args: Vec<Value>) -> Value {
        // Find the function in the program
        let function = self.program.functions.get(name);
        match function {
            Some(func) => {
                // Create a new frame for the function execution
                let mut frame = Frame::new();

                // Bind the function parameters to the arguments
                for (param, arg) in func.params.iter().zip(args) {
                    frame.bind(param.getName(), arg);
                }

                match &func.kind {
                    FunctionKind::UserDefined(_) => {
                        if let Some(ret) = &func.body {
                            let mut executor = FunctionExecutor::new(ret, frame, self);
                            return executor.execute();
                        } else {
                            panic!("Function body is empty for user-defined function: {}", name);
                        }
                    }
                    FunctionKind::VariantCtor(_) => {
                        let enum_name = name.base().to_string();
                        let variant_name = name.to_string();
                        let enum_value = Value::Enum(EnumValue {
                            name: enum_name,
                            variant: variant_name,
                            fields: func
                                .params
                                .iter()
                                .map(|param| {
                                    frame
                                        .get(&param.getName())
                                        .expect("Parameter not found in frame")
                                        .clone()
                                })
                                .collect(),
                        });
                        return enum_value;
                    }
                    FunctionKind::StructCtor => {
                        let struct_value = Value::Struct(StructValue {
                            name: name.to_string(),
                            fields: func
                                .params
                                .iter()
                                .map(|param| {
                                    let value = frame
                                        .get(&param.getName())
                                        .expect("Parameter not found in frame")
                                        .clone();
                                    (param.getName(), value)
                                })
                                .collect(),
                        });
                        return struct_value;
                    }
                    FunctionKind::Extern(_) => match name {
                        _ => {
                            panic!("Unsupported extern function call in interpreter: {}", name);
                        }
                    },
                    FunctionKind::TraitMemberDecl(qualified_name) => {
                        unreachable!("Trait member found in interpreter stage: {}", qualified_name)
                    }
                    FunctionKind::TraitMemberDefinition(qualified_name) => {
                        unreachable!("Trait member definition found in interpreter stage: {}", qualified_name)
                    }
                    FunctionKind::EffectMemberDecl(qualified_name) => {
                        unreachable!("Effect member found in interpreter stage: {}", qualified_name)
                    }
                    FunctionKind::EffectMemberDefinition(qualified_name) => {
                        unreachable!(
                            "Effect member definition found in interpreter stage: {}",
                            qualified_name
                        )
                    }
                }
            }
            None => {
                panic!("Function not found: {}", name);
            }
        }
    }
}

struct FunctionExecutor<'a> {
    body: &'a Body,
    frame: Frame,
    interpreter: &'a Interpreter<'a>,
    returnValue: Option<Value>,
}

impl<'a> FunctionExecutor<'a> {
    fn new(body: &'a Body, frame: Frame, interpreter: &'a Interpreter<'a>) -> Self {
        FunctionExecutor {
            body,
            frame,
            interpreter,
            returnValue: None,
        }
    }

    fn execute(&mut self) -> Value {
        self.executeBlock(BlockId::first());
        self.returnValue.clone().expect("No return value found")
    }

    fn executeBlock(&mut self, blockId: BlockId) {
        let block = self.body.getBlockById(blockId);
        let inner = block.getInner();
        let b = inner.borrow();
        for instruction in &b.instructions {
            match &instruction.kind {
                InstructionKind::DeclareVar(_, _) => {}
                InstructionKind::FunctionCall(dest, call_info) => {
                    let args: Vec<Value> = call_info
                        .args
                        .getVariables()
                        .iter()
                        .map(|arg| {
                            self.frame
                                .get(&arg.name().to_string())
                                .expect("Argument not found in frame")
                                .clone()
                        })
                        .collect();
                    let result = self.interpreter.executeFunction(&call_info.name, args);
                    self.frame.bind(dest.name().to_string(), result);
                }
                InstructionKind::Converter(_, _) => {
                    unreachable!("Converter instruction not supported in interpreter")
                }
                InstructionKind::MethodCall(_, _, _, _) => {
                    unreachable!("Method call instruction not supported in interpreter")
                }
                InstructionKind::DynamicFunctionCall(_, _, _) => {
                    unreachable!("Dynamic function call instruction not supported in interpreter")
                }
                InstructionKind::FieldRef(_, _, _) => {}
                InstructionKind::Bind(_, _, _) => {
                    panic!("Bind instruction not supported in interpreter")
                }
                InstructionKind::Tuple(_, _) => {
                    panic!("Tuple instruction not supported in interpreter")
                }
                InstructionKind::StringLiteral(dest, lit) => {
                    self.frame.bind(dest.name().to_string(), Value::Str(lit.clone()));
                }
                InstructionKind::IntegerLiteral(dest, lit) => {
                    self.frame.bind(dest.name().to_string(), Value::Int(lit.clone()));
                }
                InstructionKind::CharLiteral(dest, lit) => {
                    self.frame.bind(dest.name().to_string(), Value::Char(lit.clone()));
                }
                InstructionKind::Return(_, arg) => {
                    let value = self.frame.get(&arg.name().to_string()).cloned();
                    self.returnValue = value;
                }
                InstructionKind::Ref(dest, arg) => {
                    let value = self
                        .frame
                        .get(&arg.name().to_string())
                        .cloned()
                        .expect("Variable not found in frame");
                    self.frame.bind(dest.name().to_string(), value);
                }
                InstructionKind::PtrOf(_, _) => {
                    panic!("PtrOf instruction not supported in interpreter")
                }
                InstructionKind::DropPath(_) => {
                    panic!("DropPath instruction not supported in interpreter")
                }
                InstructionKind::DropMetadata(_) => {
                    panic!("DropMetadata instruction not supported in interpreter")
                }
                InstructionKind::Drop(_, _) => {
                    panic!("Drop instruction not supported in interpreter")
                }
                InstructionKind::Jump(_, targetBlock) => {
                    self.executeBlock(*targetBlock);
                    return;
                }
                InstructionKind::Assign(dest, src) => {
                    let value = self
                        .frame
                        .get(&src.name().to_string())
                        .expect("Variable not found")
                        .clone();
                    self.frame.bind(dest.name().to_string(), value);
                }
                InstructionKind::FieldAssign(_, _, _) => {}
                InstructionKind::AddressOfField(_, _, _) => {}
                InstructionKind::Transform(_, _, _) => {}
                InstructionKind::EnumSwitch(_, _) => {}
                InstructionKind::IntegerSwitch(_, _) => {}
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::With(_, _) => {
                    panic!("With instruction not supported in interpreter")
                }
                InstructionKind::ReadImplicit(_, _) => {}
                InstructionKind::WriteImplicit(_, _) => {}
                InstructionKind::LoadPtr(_, _) => {}
                InstructionKind::StorePtr(_, _) => {}
                InstructionKind::CreateClosure(_, _) => {
                    panic!("CreateClosure instruction not supported in interpreter")
                }
                InstructionKind::ClosureReturn(_, _, _) => {
                    panic!("ClosureReturn instruction not supported in interpreter")
                }
                InstructionKind::IntegerOp(_, _, _, _) => {
                    unimplemented!("IntegerOp instruction not supported in interpreter")
                }
                InstructionKind::Yield(_, _) => {
                    unimplemented!("Yield instruction not supported in interpreter")
                }
                InstructionKind::FunctionPtr(_, _) => {
                    unimplemented!("FunctionPtr instruction not supported in interpreter")
                }
                InstructionKind::FunctionPtrCall(_, _, _) => {
                    unimplemented!("FunctionPtrCall instruction not supported in interpreter")
                }
                InstructionKind::Sizeof(_, _) => {
                    unimplemented!("Sizeof instruction not supported in interpreter")
                }
                InstructionKind::Transmute(_, _) => {
                    unimplemented!("Transmute instruction not supported in interpreter")
                }
                InstructionKind::CreateUninitializedArray(_) => {
                    unimplemented!("CreateArray instruction not supported in interpreter")
                }
                InstructionKind::ArrayLen(_, _) => {
                    unimplemented!("ArrayLen instruction not supported in interpreter")
                }
            }
        }
    }
}
