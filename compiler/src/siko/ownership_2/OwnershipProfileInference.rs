use std::{collections::BTreeMap, result};

use crate::siko::{
    hir::{
        Function::{Function, FunctionKind},
        Instruction::InstructionKind,
        Program::Program,
        Type::{formatTypes, Type},
    },
    ownership::{DataOwnershipVar::DataOwnershipVarInference, FunctionGroups},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::DependencyGroup,
};

struct Inferer<'a> {
    program: &'a Program,
    profiles: BTreeMap<QualifiedName, FunctionOwnershipProfile>,
}

impl Inferer<'_> {
    fn new(program: &Program) -> Inferer {
        Inferer {
            program,
            profiles: BTreeMap::new(),
        }
    }

    fn addProfile(&mut self, name: QualifiedName, profile: FunctionOwnershipProfile) {
        self.profiles.insert(name, profile);
    }

    fn processStructCtorFunction(&mut self, f: &Function) {
        let t = f.getType();
        let (args, result) = t.splitFnType().unwrap();
        // println!(
        //     "Processing struct constructor for type: {} {}",
        //     formatTypes(&args),
        //     result
        // );
        let s = if let Type::Named(name, _) = &result {
            let s = self.program.structs.get(name).expect("Struct not found in program");
            s
        } else {
            panic!("Expected a named type for struct constructor result, found: {}", result);
        };
        let args = args
            .into_iter()
            .enumerate()
            .map(|(index, _)| s.fields[index].ty.clone())
            .collect();
        let result = s.ty.clone();
        //println!("Processed args: {}, result: {}", formatTypes(&args), result);
        let profile = FunctionOwnershipProfile::new(args, result);
        self.addProfile(f.name.clone(), profile);
    }

    fn processVariantCtorFunction(&mut self, f: &Function, variantIndex: usize) {
        let t = f.getType();
        let (args, result) = t.splitFnType().unwrap();
        // println!(
        //     "Processing variant constructor for type: {} {}",
        //     formatTypes(&args),
        //     result
        // );
        let e = if let Type::Named(name, _) = &result {
            let e = self.program.enums.get(name).expect("Enum not found in program");
            e
        } else {
            panic!(
                "Expected a named type for variant constructor result, found: {}",
                result
            );
        };
        let args = args
            .into_iter()
            .enumerate()
            .map(|(index, _)| e.variants[variantIndex].items[index].clone())
            .collect();
        let result = e.ty.clone();
        //println!("Processed args: {}, result: {}", formatTypes(&args), result);
        let profile = FunctionOwnershipProfile::new(args, result);
        self.addProfile(f.name.clone(), profile);
    }

    fn processUserDefinedFunctions(&mut self, f: &Function) {
        if let Some(body) = &f.body {
            let t = f.getType();
            let (args, result) = t.splitFnType().unwrap();
            let args = args.into_iter().map(|ty| ty.clone()).collect();
            let blockIds = body.getAllBlockIds();
            for id in blockIds {
                let block = body.getBlockById(id);
                for i in &block.instructions {
                    let vars = i.kind.collectVariables();

                    match &i.kind {
                        InstructionKind::FunctionCall(_, name, _) => {
                            if let Some(dependency) = self.program.functions.get(name) {
                                self.processUserDefinedFunctions(dependency);
                            }
                        }
                        _ => {}
                    }
                }
            }
            let profile = FunctionOwnershipProfile::new(args, result);
            self.addProfile(f.name.clone(), profile);
        } else {
            panic!("Function body is missing for user-defined function: {:?}", f.name);
        }
    }

    fn processFunctionGroup(&mut self, group: &DependencyGroup<QualifiedName>) {
        //println!("Processing function group: {:?}", group);
        if group.items.len() == 1 {
            let item = &group.items[0];
            let function = self.program.functions.get(item).unwrap();
            match &function.kind {
                FunctionKind::UserDefined => {
                    //println!("Processing user-defined function: {:?}", item);
                }
                FunctionKind::VariantCtor(index) => {
                    self.processVariantCtorFunction(function, *index as usize);
                }
                FunctionKind::StructCtor => {
                    self.processStructCtorFunction(function);
                }
                FunctionKind::Extern => {
                    //println!("Processing external function: {:?}", item);
                }
                FunctionKind::TraitMemberDecl(qualified_name) => {
                    panic!("Trait member declaration not expected here: {:?}", qualified_name);
                }
                FunctionKind::TraitMemberDefinition(qualified_name) => {
                    panic!("Trait member definition not expected here: {:?}", qualified_name);
                }
            }
        } else {
        }
    }
}

struct FunctionOwnershipProfile {
    args: Vec<Type>,
    result: Type,
}

impl FunctionOwnershipProfile {
    fn new(args: Vec<Type>, result: Type) -> Self {
        FunctionOwnershipProfile { args, result }
    }
}

pub fn ownershipInference(program: Program) {
    let data_lifetime_inferer = DataOwnershipVarInference::new(program);
    let program = data_lifetime_inferer.process();
    let groups = FunctionGroups::createFunctionGroups(&program.functions);
    let mut inferer = Inferer::new(&program);
    for group in groups {
        inferer.processFunctionGroup(&group);
    }
}
