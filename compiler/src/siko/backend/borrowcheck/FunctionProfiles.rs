use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    backend::borrowcheck::DataGroups::{DataGroups, EnumDef, ExtendedType, FieldDef, StructDef, VariantDef},
    hir::{
        Apply::Apply,
        Block::Block,
        Function::{Function, FunctionKind},
        Instantiation::instantiateTypes,
        Instruction::{FieldId, FieldInfo, InstructionKind},
        Program::Program,
        Substitution::Substitution,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

#[derive(Clone)]
pub struct FunctionProfile {
    pub name: QualifiedName,
    pub args: Vec<ExtendedType>,
    pub result: ExtendedType,
}

impl Display for FunctionProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| format!("{}", a))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}({}) -> {}", self.name, args, self.result)
    }
}

pub struct FunctionProfileBuilder<'a> {
    f: &'a Function,
    program: &'a Program,
    dataGroups: &'a DataGroups<'a>,
    allocator: TypeVarAllocator,
    unifier: Unifier,
    varTypes: BTreeMap<VariableName, ExtendedType>,
    profile: FunctionProfile,
}

impl<'a> FunctionProfileBuilder<'a> {
    pub fn new(f: &'a Function, program: &'a Program, dataGroups: &'a DataGroups<'a>) -> Self {
        FunctionProfileBuilder {
            f,
            program,
            dataGroups,
            allocator: TypeVarAllocator::new(),
            unifier: Unifier::new(),
            varTypes: BTreeMap::new(),
            profile: FunctionProfile {
                name: f.name.clone(),
                args: Vec::new(),
                result: ExtendedType::new(Type::getUnitType()),
            },
        }
    }

    fn extendType(&self, ty: &Type) -> ExtendedType {
        let mut extTy = self.dataGroups.extendType(ty);
        self.allocator.useTypes(&extTy.vars);
        let sub = instantiateTypes(&self.allocator, &extTy.vars);
        extTy.vars = extTy.vars.clone().apply(&sub);
        extTy
    }

    fn instantiateStructDef(&self, def: &StructDef) -> StructDef {
        let mut newDef = def.clone();
        self.allocator.useTypes(&newDef.ty.vars);
        let sub = instantiateTypes(&self.allocator, &newDef.ty.vars);
        newDef.ty.vars = newDef.ty.vars.clone().apply(&sub);
        for f in newDef.fields.iter_mut() {
            f.ty.vars = f.ty.vars.clone().apply(&sub);
        }
        newDef
    }

    fn instantiateEnumDef(&self, def: &EnumDef) -> EnumDef {
        let mut newDef = def.clone();
        for v in newDef.ty.vars.iter() {
            self.allocator.useType(v);
        }
        let sub = instantiateTypes(&self.allocator, &newDef.ty.vars);
        newDef.ty.vars = newDef.ty.vars.clone().apply(&sub);
        for v in newDef.variants.iter_mut() {
            v.ty.vars = v.ty.vars.clone().apply(&sub);
        }
        newDef
    }

    fn unifyExtendedTypes(&mut self, a: &ExtendedType, b: &ExtendedType) {
        //println!("Unifying extended types: {} and {}", a, b);
        assert_eq!(a.vars.len(), b.vars.len());
        for (va, vb) in a.vars.iter().zip(b.vars.iter()) {
            self.unifier.unify(va.clone(), vb.clone(), Location::empty());
        }
    }

    fn getVarType(&self, var: &Variable) -> ExtendedType {
        self.varTypes
            .get(&var.name())
            .expect(&format!("Variable type not found: {}", var.name()))
            .clone()
    }

    pub fn process(&mut self) {
        //println!("Building function profile for: {}", self.f.name);
        self.profile = FunctionProfile {
            name: self.f.name.clone(),
            args: Vec::new(),
            result: self.extendType(&self.f.result),
        };
        for p in &self.f.params {
            let ty = p.getType();
            let extTy = self.extendType(&ty);
            self.profile.args.push(extTy);
        }
        //println!("Function profile: {}", profile);
        let body = match &self.f.body {
            Some(body) => body,
            None => {
                match self.f.kind {
                    FunctionKind::StructCtor => {
                        let profile = self.profile.clone();
                        let structDef = self.dataGroups.getStruct(&self.f.name);
                        let instantiatedDef = self.instantiateStructDef(structDef);
                        for (index, field) in instantiatedDef.fields.iter().enumerate() {
                            // println!(
                            //     "Unifying struct field: {} with arg type: {}",
                            //     field.ty, profile.args[index]
                            // );
                            let mut fieldTy = field.ty.clone();
                            if fieldTy.vars.len() < profile.args[index].vars.len() {
                                fieldTy.vars.insert(0, self.allocator.next());
                            }
                            self.unifyExtendedTypes(&fieldTy, &profile.args[index]);
                        }
                        self.unifyExtendedTypes(&instantiatedDef.ty, &profile.result);
                        self.profile = self.unifier.apply(profile);
                        //println!("Struct ctor profile: {}", self.profile);
                    }
                    _ => {
                        //println!("Function has no body, skipping body processing");
                    }
                }
                return;
            }
        };
        for (_, b) in &body.blocks {
            self.processBlock(b);
        }
    }

    pub fn processBlock(&mut self, block: &Block) {
        let inner = block.getInner();
        let b = inner.borrow();
        for instr in &b.instructions {
            let vars = instr.kind.collectVariables();
            for var in vars {
                if self.varTypes.contains_key(&var.name()) {
                    continue;
                }
                let extTy = self.extendType(&var.getType());
                self.varTypes.insert(var.name(), extTy);
            }
            //println!("Processing instruction: {}", instr);
            match &instr.kind {
                InstructionKind::FunctionCall(dest, info) => {}
                InstructionKind::Converter(_, _) => panic!("Converter in borrow checker"),
                InstructionKind::MethodCall(_, _, _, _) => {
                    panic!("MethodCall in borrow checker")
                }
                InstructionKind::DynamicFunctionCall(_, _, _) => {
                    panic!("DynamicFunctionCall in borrow checker")
                }
                InstructionKind::FieldRef(dest, receiver, infos) => {
                    let currenTy = self.resolveFieldInfos(receiver, infos);
                    let destTy = self.getVarType(dest);
                    //println!("FieldRef: {} {} {}", currenTy, destTy, receiver.getType());
                    self.unifyExtendedTypes(&currenTy, &destTy);
                }
                InstructionKind::Bind(_, _, _) => panic!("Bind in borrow checker"),
                InstructionKind::Tuple(_, _) => panic!("Tuple in borrow checker"),
                InstructionKind::StringLiteral(_, _) => {}
                InstructionKind::IntegerLiteral(_, _) => {}
                InstructionKind::CharLiteral(_, _) => {}
                InstructionKind::Return(_, arg) => {
                    let varType = self.getVarType(arg);
                    self.unifyExtendedTypes(&varType, &self.profile.result.clone());
                }
                InstructionKind::Ref(dest, src) => {
                    let srcType = self.getVarType(src);
                    let mut destType = self.getVarType(dest);
                    destType.base();
                    //println!("Ref types: src: {} dest: {}", srcType, destType);
                    self.unifyExtendedTypes(&srcType, &destType);
                }
                InstructionKind::PtrOf(dest, var) => {
                    let varType = self.getVarType(var);
                    let destType = self.getVarType(dest);
                    //println!("PtrOf types: var: {} dest: {}", varType, destType);
                    self.unifyExtendedTypes(&varType, &destType);
                }
                InstructionKind::DropPath(_) => panic!("DropPath in borrow checker"),
                InstructionKind::DropMetadata(_) => panic!("DropMetadata in borrow checker"),
                InstructionKind::Drop(_, _) => panic!("Drop in borrow checker"),
                InstructionKind::Jump(_, _) => {}
                InstructionKind::Assign(dest, src) => {
                    let srcType = self.getVarType(src);
                    let destType = self.getVarType(dest);
                    self.unifyExtendedTypes(&srcType, &destType);
                }
                InstructionKind::FieldAssign(root, value, infos) => {
                    let valueType = self.getVarType(value);
                    let currenTy = self.resolveFieldInfos(root, infos);
                    //println!("FieldAssign: {} {}", currenTy, valueType);
                    self.unifyExtendedTypes(&currenTy, &valueType);
                }
                InstructionKind::DeclareVar(_, _) => {}
                InstructionKind::Transform(dest, root, index) => {
                    let destType = self.getVarType(dest);
                    let rootTy = root.getType();
                    let name = rootTy.getName().expect("Transform root must have name");
                    let enumDef = self.dataGroups.getEnum(&name);
                    let enumDef = self.instantiateEnumDef(enumDef);
                    let v = &enumDef.variants[*index as usize];
                    let mut variantTy = v.ty.clone();
                    if rootTy.isReference() {
                        variantTy.vars.push(self.allocator.next());
                    }
                    self.unifyExtendedTypes(&destType, &variantTy);
                }
                InstructionKind::EnumSwitch(_, _) => {}
                InstructionKind::IntegerSwitch(_, _) => {}
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::With(_, _) => panic!("With in borrow checker"),
                InstructionKind::ReadImplicit(_, _) => panic!("ReadImplicit in borrow checker"),
                InstructionKind::WriteImplicit(_, _) => panic!("WriteImplicit in borrow checker"),
                InstructionKind::LoadPtr(v1, v2) => {
                    let v1Type = self.getVarType(v1);
                    let v2Type = self.getVarType(v2);
                    //println!("LoadPtr: {} {}", v1Type, v2Type);
                    self.unifyExtendedTypes(&v1Type, &v2Type);
                }
                InstructionKind::StorePtr(v1, v2) => {
                    let v1Type = self.getVarType(v1);
                    let v2Type = self.getVarType(v2);
                    //println!("StorePtr: {} {}", v1Type, v2Type);
                    self.unifyExtendedTypes(&v1Type, &v2Type);
                }
                InstructionKind::CreateClosure(_, _) => {
                    panic!("CreateClosure in borrow checker")
                }
                InstructionKind::ClosureReturn(_, _, _) => {
                    panic!("ClosureReturn in borrow checker")
                }
                InstructionKind::AddressOfField(dest, root, fieldInfos) => {
                    let currenTy = self.resolveFieldInfos(root, fieldInfos);
                    let mut destType = self.getVarType(dest);
                    if currenTy.vars.len() < destType.vars.len() {
                        destType.base();
                    }
                    //println!("AddressOfField: {} {}", currenTy, destType);
                    self.unifyExtendedTypes(&currenTy, &destType);
                }
            }
        }
    }

    fn resolveFieldInfos(&self, root: &Variable, infos: &Vec<FieldInfo>) -> ExtendedType {
        let mut currenTy = self.getVarType(root);
        for info in infos {
            let refVar = if currenTy.ty.isReference() {
                Some(currenTy.vars[0].clone())
            } else {
                None
            };
            match &info.name {
                FieldId::Named(name) => {
                    let structName = currenTy
                        .ty
                        .clone()
                        .unpackPtr()
                        .unpackRef()
                        .getName()
                        .expect("resolveFieldInfos root must have name");
                    //println!("resolveFieldInfos root type: {} {}", currenTy, name);
                    let mut structDef = self.dataGroups.getStruct(&structName).clone();
                    structDef = self.instantiateStructDef(&structDef);
                    let field = structDef.getField(&name);
                    currenTy = field.ty.clone();
                    if let Some(refVar) = refVar {
                        currenTy.ty = currenTy.ty.asRef();
                        currenTy.vars.insert(0, refVar);
                    }
                }
                _ => {
                    panic!("Non named fieldId in borrow checker");
                }
            }
        }
        currenTy
    }
}

impl Apply for ExtendedType {
    fn apply(self, sub: &Substitution) -> Self {
        let newVars = self.vars.clone().apply(sub);
        ExtendedType {
            ty: self.ty.clone(),
            vars: newVars,
        }
    }
}

impl Apply for FieldDef {
    fn apply(self, sub: &Substitution) -> Self {
        FieldDef {
            name: self.name.clone(),
            ty: self.ty.clone().apply(sub),
            inGroup: self.inGroup,
        }
    }
}

impl Apply for StructDef {
    fn apply(self, sub: &Substitution) -> Self {
        let newTy = self.ty.clone().apply(sub);
        let newFields = self.fields.clone().apply(sub);
        StructDef {
            name: self.name.clone(),
            ty: newTy,
            fields: newFields,
        }
    }
}

impl Apply for VariantDef {
    fn apply(self, sub: &Substitution) -> Self {
        VariantDef {
            name: self.name.clone(),
            ty: self.ty.clone().apply(sub),
            inGroup: self.inGroup,
        }
    }
}

impl Apply for EnumDef {
    fn apply(self, sub: &Substitution) -> Self {
        let newTy = self.ty.clone().apply(sub);
        let newVariants = self.variants.clone().apply(sub);
        EnumDef {
            name: self.name.clone(),
            ty: newTy,
            variants: newVariants,
        }
    }
}

impl Apply for FunctionProfile {
    fn apply(self, sub: &Substitution) -> Self {
        let newArgs = self.args.clone().apply(sub);
        let newResult = self.result.clone().apply(sub);
        FunctionProfile {
            name: self.name.clone(),
            args: newArgs,
            result: newResult,
        }
    }
}
