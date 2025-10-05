use std::collections::BTreeMap;

use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfile::{FunctionProfile, Link},
            FunctionProfileStore::FunctionProfileStore,
        },
        DataGroups::{DataGroups, EnumDef, ExtendedType, StructDef},
    },
    hir::{
        Apply::Apply,
        Block::Block,
        Function::{Function, FunctionKind},
        Instantiation::instantiateTypes,
        Instruction::{FieldId, FieldInfo, InstructionKind},
        Program::Program,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
    util::Runner::Runner,
};

pub struct FunctionProfileBuilder<'a> {
    pub f: &'a Function,
    pub program: &'a Program,
    pub dataGroups: &'a DataGroups<'a>,
    pub allocator: TypeVarAllocator,
    pub unifier: Unifier,
    pub varTypes: BTreeMap<VariableName, ExtendedType>,
    pub profile: FunctionProfile,
    pub paramNameMap: BTreeMap<String, usize>,
    pub profileStore: &'a mut FunctionProfileStore,
    pub functionGroup: Vec<QualifiedName>,
}

impl<'a> FunctionProfileBuilder<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dataGroups: &'a DataGroups<'a>,
        profileStore: &'a mut FunctionProfileStore,
        functionGroup: Vec<QualifiedName>,
        runner: Runner,
    ) -> Self {
        FunctionProfileBuilder {
            f,
            program,
            dataGroups,
            allocator: TypeVarAllocator::new(),
            unifier: Unifier::new(runner.child("unifier")),
            varTypes: BTreeMap::new(),
            profile: FunctionProfile {
                name: f.name.clone(),
                args: Vec::new(),
                result: ExtendedType::new(Type::getUnitType()),
                links: Vec::new(),
            },
            paramNameMap: BTreeMap::new(),
            profileStore,
            functionGroup,
        }
    }

    fn extendType(&self, ty: &Type) -> ExtendedType {
        let mut extTy = self.dataGroups.extendType(ty);
        self.allocator.useTypes(&extTy.vars);
        let sub = instantiateTypes(&self.allocator, &extTy.vars);
        extTy = extTy.apply(&sub);
        extTy
    }

    fn instantiateStructDef(&self, def: &StructDef) -> StructDef {
        let mut newDef = def.clone();
        self.allocator.useTypes(&newDef.ty.vars);
        let sub = instantiateTypes(&self.allocator, &newDef.ty.vars);
        newDef = newDef.apply(&sub);
        newDef
    }

    fn instantiateEnumDef(&self, def: &EnumDef) -> EnumDef {
        let mut newDef = def.clone();
        for v in newDef.ty.vars.iter() {
            self.allocator.useType(v);
        }
        let sub = instantiateTypes(&self.allocator, &newDef.ty.vars);
        newDef = newDef.apply(&sub);
        newDef
    }

    fn instantiateFunctionProfile(&self, profile: &FunctionProfile) -> FunctionProfile {
        let mut newProfile = profile.clone();
        for v in newProfile.collectVars() {
            self.allocator.useType(&v);
        }
        let sub = instantiateTypes(&self.allocator, &newProfile.collectVars());
        newProfile = newProfile.apply(&sub);
        newProfile
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

    pub fn getFinalVarType(&self, var: &Variable) -> ExtendedType {
        let varType = self.getVarType(var);
        self.unifier.apply(varType)
    }

    pub fn process(&mut self, normalize: bool) -> bool {
        //println!("Building function profile for: {}", self.f.name);
        //println!("Function: {}", self.f);
        self.profile = FunctionProfile {
            name: self.f.name.clone(),
            args: Vec::new(),
            result: self.extendType(&self.f.result.getReturnType()),
            links: Vec::new(),
        };
        for (index, p) in self.f.params.iter().enumerate() {
            let ty = p.getType();
            let extTy = self.extendType(&ty);
            self.profile.args.push(extTy);
            self.paramNameMap.insert(p.getName().clone(), index);
        }
        //println!("Function profile: {}", profile);
        match &self.f.body {
            Some(body) => {
                for (_, b) in &body.blocks {
                    self.processBlock(b);
                }
                self.profile = self.unifier.apply(self.profile.clone());
            }
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
                    FunctionKind::VariantCtor(index) => {
                        let profile = self.profile.clone();
                        let enumName = self
                            .f
                            .result
                            .getReturnType()
                            .getName()
                            .expect("Variant ctor must have enum type");
                        let enumDef = self.dataGroups.getEnum(&enumName);
                        let instantiatedDef = self.instantiateEnumDef(enumDef);
                        let variant = &instantiatedDef.variants[index as usize];
                        // println!(
                        //     "Unifying enum variant: {} with arg type: {}",
                        //     variant.ty, profile.args[0]
                        // );
                        let mut variantTy = variant.ty.clone();
                        if !profile.args.is_empty() {
                            // Variant without argument
                            if variantTy.vars.len() < profile.args[0].vars.len() {
                                variantTy.vars.insert(0, self.allocator.next());
                            }
                            self.unifyExtendedTypes(&variantTy, &profile.args[0]);
                        }
                        self.unifyExtendedTypes(&instantiatedDef.ty, &profile.result);
                        self.profile = self.unifier.apply(profile);
                        //println!("Variant ctor profile: {}", self.profile);
                    }
                    FunctionKind::Extern(_) => {}
                    _ => {
                        unreachable!("Function without body in borrow checker: {}", self.f.name);
                    }
                }
            }
        };
        if normalize {
            self.profile.normalize();
            //println!("Normalized function profile: {}", self.profile);
            self.profile.processLinks();
        }
        //println!("Function profile: {}", self.profile);
        let updated = self.profileStore.addProfile(self.profile.clone());
        updated
    }

    pub fn processBlock(&mut self, block: &Block) {
        let inner = block.getInner();
        let b = inner.borrow();
        for instr in &b.instructions {
            let vars = instr.kind.collectVariables();
            for var in vars {
                let name = var.name();
                if self.varTypes.contains_key(&name) {
                    continue;
                }
                if let VariableName::Arg(argName) = &name {
                    let paramIndex = self
                        .paramNameMap
                        .get(argName)
                        .cloned()
                        .expect(&format!("Argument variable not found in param map: {}", argName));
                    let extTy = self.profile.args[paramIndex as usize].clone();
                    self.varTypes.insert(name, extTy);
                    continue;
                }
                let extTy = self.extendType(&var.getType());
                //println!("Variable type: {} {}", var.name(), extTy);
                self.varTypes.insert(name, extTy);
            }
            //println!("Processing instruction: {}", instr);
            match &instr.kind {
                InstructionKind::FunctionCall(dest, info) => {
                    match self.profileStore.getProfile(&info.name) {
                        Some(calleeProfile) => {
                            let calleeProfile = self.instantiateFunctionProfile(calleeProfile);
                            //println!("Function call to known function: {}", info.name);
                            let destType = self.getVarType(dest);
                            self.unifyExtendedTypes(&destType, &calleeProfile.result);
                            for (index, arg) in info.args.getVariables().iter().enumerate() {
                                let argType = self.getVarType(arg);
                                let mut calleeArgType = calleeProfile.args[index].clone();
                                if argType.vars.len() > calleeArgType.vars.len() {
                                    calleeArgType.vars.insert(0, self.allocator.next());
                                }
                                self.unifyExtendedTypes(&argType, &calleeArgType);
                            }
                            for link in &calleeProfile.links {
                                self.profile.links.push(link.clone());
                            }
                        }
                        None => {
                            if !self.functionGroup.contains(&info.name) {
                                panic!("Missing function profile for function call: {}", info.name);
                            }
                        }
                    }
                }
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
                    let mut destTy = self.getVarType(dest);
                    let mut srcTy = self.getVarType(src);
                    assert_eq!(destTy.ty.isReference(), srcTy.ty.isReference());
                    if destTy.ty.isReference() {
                        let to = destTy.base();
                        let from = srcTy.base();
                        //println!("Link created: {} -> {}", from, to);
                        self.profile.links.push(Link::new(from, to));
                        self.unifyExtendedTypes(&destTy, &srcTy);
                    } else {
                        let srcType = self.getVarType(src);
                        let destType = self.getVarType(dest);
                        self.unifyExtendedTypes(&srcType, &destType);
                    }
                }
                InstructionKind::FieldAssign(root, value, infos) => {
                    let valueType = self.getVarType(value);
                    let currenTy = self.resolveFieldInfos(root, infos);
                    //println!("FieldAssign: {} {}", currenTy, valueType);
                    self.unifyExtendedTypes(&currenTy, &valueType);
                }
                InstructionKind::DeclareVar(_, _) => {}
                InstructionKind::Transform(dest, root, info) => {
                    let destType = self.getVarType(dest);
                    let rootTy = root.getType();
                    let name = rootTy.getName().expect("Transform root must have name");
                    let enumDef = self.dataGroups.getEnum(&name);
                    let enumDef = self.instantiateEnumDef(enumDef);
                    let v = &enumDef.variants[info.variantIndex as usize];
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
                InstructionKind::AddressOfField(dest, root, fieldInfos, _) => {
                    let currenTy = self.resolveFieldInfos(root, fieldInfos);
                    let mut destType = self.getVarType(dest);
                    if currenTy.vars.len() < destType.vars.len() {
                        destType.base();
                    }
                    //println!("AddressOfField: {} {}", currenTy, destType);
                    self.unifyExtendedTypes(&currenTy, &destType);
                }
                InstructionKind::IntegerOp(_, _, _, _) => {}
                InstructionKind::Yield(_, _) => {
                    unreachable!("Yield in borrow checker");
                }
                InstructionKind::FunctionPtr(_, _) => {}
                InstructionKind::FunctionPtrCall(_, _, _) => {}
                InstructionKind::Sizeof(_, _) => {}
                InstructionKind::Transmute(_, _) => {}
                InstructionKind::CreateUninitializedArray(_) => {}
                InstructionKind::ArrayLen(_, _) => {}
            }
        }
    }

    fn resolveFieldInfos(&mut self, root: &Variable, infos: &Vec<FieldInfo>) -> ExtendedType {
        let mut currenTy = self.getVarType(root);
        for info in infos {
            let refVar = if currenTy.ty.isReference() {
                //println!("Reference in field access: {}", currenTy);
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
                    if structDef.ty.vars.len() < currenTy.vars.len() {
                        structDef.ty.vars.insert(0, self.allocator.next());
                    }
                    self.unifyExtendedTypes(&structDef.ty, &currenTy);
                    structDef = self.unifier.apply(structDef);
                    //println!("Instantiated struct def: {}", structDef);
                    let field = structDef.getField(&name);
                    currenTy = field.ty.clone();
                    if let Some(refVar) = refVar {
                        if !currenTy.ty.isReference() {
                            currenTy.ty = currenTy.ty.asRef();
                            currenTy.vars.insert(0, refVar);
                        }
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
