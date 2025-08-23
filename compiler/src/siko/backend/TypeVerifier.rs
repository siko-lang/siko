use std::collections::BTreeMap;

use crate::siko::hir::{
    Function::Function,
    Instantiation::{instantiateEnum, instantiateStruct, instantiateType},
    Instruction::{FieldId, FieldInfo, Instruction, InstructionKind, WithContext},
    Program::Program,
    Substitution::Substitution,
    Type::Type,
    TypeVarAllocator::TypeVarAllocator,
    Unification::unify,
    Variable::{Variable, VariableName},
};

pub fn verifyTypes(program: &Program) {
    for (_, function) in &program.functions {
        let mut verifier = TypeVerifier::new(program, function);
        verifier.verify()
    }
}

pub struct TypeVerifier<'a> {
    program: &'a Program,
    function: &'a Function,
    variableTypes: BTreeMap<VariableName, Type>,
    allocator: TypeVarAllocator,
    substitution: Substitution,
}

impl<'a> TypeVerifier<'a> {
    pub fn new(program: &'a Program, function: &'a Function) -> TypeVerifier<'a> {
        TypeVerifier {
            program,
            function,
            variableTypes: BTreeMap::new(),
            allocator: TypeVarAllocator::new(),
            substitution: Substitution::new(),
        }
    }

    pub fn verify(&mut self) {
        //println!("Verifying function: {}", self.function.name);
        //println!("Function: {}", self.function);
        if let Some(body) = &self.function.body {
            for (_, block) in &body.blocks {
                for instruction in &block.instructions {
                    self.verifyInstruction(instruction);
                }
            }
        }
    }

    fn checkVariable(&mut self, var: &Variable) {
        let varType = var.getType();
        if let Some(existing_type) = self.variableTypes.get(&var.name) {
            assert_eq!(
                existing_type, varType,
                "Variable {} has inconsistent types: was {:?}, now {:?} in function {}",
                var.name, existing_type, varType, self.function.name
            );
        } else {
            self.variableTypes.insert(var.name.clone(), varType.clone());
        }
    }

    fn checkFieldInfo(&mut self, mut rootType: Type, fieldInfo: &FieldInfo) -> Type {
        let mut isRef = false;
        if let Type::Reference(inner, _) = rootType {
            isRef = true;
            // If the root type is a reference, we need to follow it
            rootType = *inner.clone();
        }
        if let Type::Ptr(inner) = rootType {
            rootType = *inner.clone();
        }
        match &fieldInfo.name {
            FieldId::Named(name) => {
                let structName = rootType.getName().expect("Field info root type should have a name");
                let structDef = self.program.getStruct(&structName).expect("Struct should exist");
                let structDef = instantiateStruct(&mut self.allocator, &structDef, &rootType);
                let (f, _) = structDef.getField(&name);
                let mut targetType = f.ty.clone();
                if isRef {
                    targetType = Type::Reference(Box::new(targetType), None);
                }
                self.unify(
                    &targetType,
                    &fieldInfo.ty.clone().expect("Field info should have a type"),
                );
            }
            FieldId::Indexed(index) => {
                let types = rootType.getTupleTypes();
                let mut targetType = types
                    .get(*index as usize)
                    .cloned()
                    .expect("Field info index should be valid");
                if isRef {
                    targetType = Type::Reference(Box::new(targetType), None);
                }
                self.unify(
                    &targetType,
                    &fieldInfo.ty.clone().expect("Field info should have a type"),
                );
            }
        }
        fieldInfo.ty.clone().expect("Field info should have a type")
    }

    fn verifyInstruction(&mut self, instruction: &Instruction) {
        //println!("Verifying instruction: {}", instruction);
        match &instruction.kind {
            InstructionKind::FunctionCall(dest, info) => {
                //println!("Function call: {} with args {:?}", fname, args);
                let argTypes = info.args.iter().map(|arg| arg.getType().clone()).collect::<Vec<_>>();
                let f = self.program.getFunction(&info.name).expect("Function not found");
                let mut fnType = f.getType();
                //println!("Orig Function type: {}", fnType);
                let (fnArgTypes, fnResult) = fnType.clone().splitFnType().expect("Function type should be splitable");
                let destType = dest.getType();
                if fnResult.hasSelfType() {
                    let mut selfType = destType
                        .getTupleTypes()
                        .get(0)
                        .cloned()
                        .expect("Destination type should have a self type");
                    selfType = instantiateType(&mut self.allocator, selfType);
                    //println!("Replacing self type: {}", selfType);
                    let fnResult = fnResult.changeSelfType(selfType);
                    fnType = Type::Function(fnArgTypes, Box::new(fnResult));
                }
                //println!("Function type: {}", fnType);
                fnType = instantiateType(&mut self.allocator, fnType);
                //println!("After substitution: {}", fnType);
                self.unify(&fnType, &Type::Function(argTypes, Box::new(destType.clone())));
                self.unify(destType, &fnType.getResult());
                // println!(
                //     "Function call {}: expected type {:?}, got {:?}",
                //     fname,
                //     fnType,
                //     dest.getType()
                // );
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                unreachable!("Method call found in instruction verification");
            }
            InstructionKind::Bind(_, _, _) => {
                unreachable!("Bind found in instruction verification");
            }
            InstructionKind::Assign(dest, src) => {
                self.unify(src.getType(), dest.getType());
            }
            InstructionKind::Ref(dest, src) => {
                let ty = Type::Reference(Box::new(src.getType().clone()), None);
                self.unify(&ty, dest.getType());
            }
            InstructionKind::PtrOf(dest, src) => {
                let ty = Type::Ptr(Box::new(src.getType().clone()));
                self.unify(&ty, dest.getType());
            }
            InstructionKind::FieldRef(dest, src, fields) => {
                //println!("FieldRef: {} src {}", dest, src.getType());
                let mut rootType = src.getType().clone();
                for f in fields {
                    rootType = self.checkFieldInfo(rootType, f);
                }
                self.unify(
                    dest.getType(),
                    &fields.last().expect("msg").ty.clone().expect("field type not found"),
                );
            }
            InstructionKind::Tuple(dest, items) => {
                //println!("Tuple create: {:?} dest {}", items, dest.getType());
                let items = items.iter().map(|item| item.getType().clone()).collect::<Vec<_>>();
                let ty = Type::Tuple(items);
                self.unify(&ty, dest.getType());
            }
            InstructionKind::Transform(dest, src, index) => {
                let mut srcTy = src.getType().clone();
                let name = srcTy.getName().expect("Transform source type should have a name");
                let enumDef = self.program.getEnum(&name).expect("Enum should exist");
                //println!("Enum definition: {}", enumDef);
                //println!("srcType: {}", srcTy);
                let mut isRef = false;
                if let Type::Reference(inner, _) = srcTy {
                    srcTy = *inner.clone();
                    isRef = true;
                }
                let enumDef = instantiateEnum(&mut self.allocator, &enumDef, &srcTy);
                let variant = enumDef.variants.get(*index as usize).expect("Variant should exist");
                let mut ty1 = Type::Tuple(variant.items.clone());
                //println!("Transform type: {}", ty1);
                if isRef {
                    ty1 = Type::Reference(Box::new(ty1), None);
                }
                self.unify(&ty1, dest.getType());
            }
            InstructionKind::Return(_, var) => {
                self.unify(var.getType(), &self.function.result);
            }
            InstructionKind::DynamicFunctionCall(_, _, _) => {
                unimplemented!("Dynamic function calls are not yet supported in type verification");
            }
            InstructionKind::DeclareVar(name, _) => {
                self.checkVariable(name);
            }
            InstructionKind::Converter(_, _) => {
                unreachable!("Converter found in instruction verification");
            }
            InstructionKind::StringLiteral(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::IntegerLiteral(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::CharLiteral(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::DropPath(_) => {
                unreachable!("Drop path found in instruction verification");
            }
            InstructionKind::DropMetadata(_) => {
                unreachable!("Drop metadata found in instruction verification")
            }
            InstructionKind::Drop(_, _) => {
                unreachable!("Drop found in instruction verification")
            }
            InstructionKind::Jump(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::FieldAssign(root, rhs, fields) => {
                let mut rootType = root.getType().clone();
                for f in fields {
                    rootType = self.checkFieldInfo(rootType, f);
                }
                self.unify(&rootType, rhs.getType());
                self.checkVariable(root);
                self.checkVariable(rhs);
            }
            InstructionKind::AddressOfField(dest, root, fields) => {
                let mut rootType = root.getType().clone();
                for f in fields {
                    rootType = self.checkFieldInfo(rootType, f);
                }
                self.checkVariable(dest);
                self.checkVariable(root);
            }
            InstructionKind::EnumSwitch(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::IntegerSwitch(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::BlockStart(_) => {
                // do nothing, block start is just a marker
            }
            InstructionKind::BlockEnd(_) => {
                // do nothing, block end is just a marker
            }
            InstructionKind::With(var, info) => {
                self.checkVariable(var);
                for c in &info.contexts {
                    match c {
                        WithContext::EffectHandler(_) => {}
                        WithContext::Implicit(handler) => {
                            self.checkVariable(&handler.var);
                        }
                    }
                }
            }
            InstructionKind::ReadImplicit(var, _) => {
                self.checkVariable(var);
            }
            InstructionKind::WriteImplicit(_, var) => {
                self.checkVariable(var);
            }
            InstructionKind::LoadPtr(dest, src) => {
                self.checkVariable(dest);
                self.checkVariable(src);
                let ty = Type::Ptr(Box::new(dest.getType().clone()));
                self.unify(&ty, src.getType());
            }
            InstructionKind::StorePtr(dest, src) => {
                self.checkVariable(dest);
                self.checkVariable(src);
                let ty = Type::Ptr(Box::new(src.getType().clone()));
                self.unify(&ty, dest.getType());
            }
        }
    }

    fn unify(&mut self, ty1: &Type, ty2: &Type) {
        if unify(&mut self.substitution, ty1.clone(), ty2.clone(), true).is_err() {
            panic!(
                "Type unification failed: {} and {} in function {}",
                ty1, ty2, self.function.name
            );
        }
    }
}
