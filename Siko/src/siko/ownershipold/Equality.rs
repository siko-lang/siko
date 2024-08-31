use std::collections::BTreeMap;

use crate::siko::{
    ir::Function::{Block, Function, InstructionId, InstructionKind, ValueKind},
    ownership::MemberInfo::{MemberInfo, MemberKind},
    qualifiedname::QualifiedName,
};

use super::{
    dataflowprofile::{
        DataFlowProfile::DataFlowProfile, DataFlowProfileStore::DataFlowProfileStore,
    },
    Instantiator::Instantiator,
    OwnershipInferenceInfo::OwnershipInferenceInfo,
    OwnershipInferenceInfo::TypedId,
    TypeVariableInfo::{GroupTypeVariable, OwnershipTypeVariable, Substitution, TypeVariableInfo},
};

pub struct EqualityEngine<'a> {
    function: &'a Function,
    profileStore: &'a DataFlowProfileStore,
    substitution: Substitution,
    groupProfiles: &'a BTreeMap<QualifiedName, DataFlowProfile>,
    profiles: BTreeMap<InstructionId, DataFlowProfile>,
    ownershipInferenceInfo: OwnershipInferenceInfo,
}

impl<'a> EqualityEngine<'a> {
    pub fn new(
        function: &'a Function,
        profileStore: &'a DataFlowProfileStore,
        groupProfiles: &'a BTreeMap<QualifiedName, DataFlowProfile>,
    ) -> EqualityEngine<'a> {
        EqualityEngine {
            function: function,
            profileStore: profileStore,
            substitution: Substitution::new(),
            groupProfiles: groupProfiles,
            profiles: BTreeMap::new(),
            ownershipInferenceInfo: OwnershipInferenceInfo::new(),
        }
    }

    pub fn run(&mut self, initializeSignature: bool) {
        self.initialize(initializeSignature);
        let block = self.function.getFirstBlock();
        self.processBlock(block); // TODO: fix this, no clone!
    }

    fn initialize(&mut self, initializeSignature: bool) {
        if initializeSignature {
            for param in &self.function.params {
                let tvInfo = self.ownershipInferenceInfo.nextTypeVariableInfo();
                self.ownershipInferenceInfo
                    .tvInfos
                    .insert(TypedId::Value(param.getName()), tvInfo.clone());
                self.ownershipInferenceInfo.signature.args.push(tvInfo);
            }
        }
        for block in &self.function.body.as_ref().unwrap().blocks {
            for instruction in &block.instructions {
                let tvInfo = self.ownershipInferenceInfo.nextTypeVariableInfo();
                self.ownershipInferenceInfo
                    .tvInfos
                    .insert(TypedId::Instruction(instruction.id), tvInfo);
                match &instruction.kind {
                    InstructionKind::ValueRef(_, _, indices) => {
                        let mut members = Vec::new();
                        let mut root = self.ownershipInferenceInfo.nextGroupVar();
                        for index in indices {
                            let member = MemberInfo::new(
                                root,
                                MemberKind::Field,
                                *index,
                                self.ownershipInferenceInfo.nextTypeVariableInfo(),
                            );
                            root = member.info.group;
                            members.push(member);
                        }
                        if !members.is_empty() {
                            members.last_mut().unwrap().info.group = tvInfo.group;
                            self.ownershipInferenceInfo
                                .members
                                .insert(instruction.id, members);
                        }
                    }
                    InstructionKind::Bind(name, _) => {
                        let tvInfo = self.ownershipInferenceInfo.nextTypeVariableInfo();
                        self.ownershipInferenceInfo
                            .tvInfos
                            .insert(TypedId::Value(name.clone()), tvInfo);
                    }
                    _ => {}
                }
            }
        }
    }

    fn getInstructionTypeVariableInfo(&self, id: InstructionId) -> TypeVariableInfo {
        self.ownershipInferenceInfo
            .tvInfos
            .get(&TypedId::Instruction(id))
            .unwrap()
            .clone()
    }

    fn getValueTypeVariableInfo(&self, v: String) -> TypeVariableInfo {
        self.ownershipInferenceInfo
            .tvInfos
            .get(&&TypedId::Value(v))
            .unwrap()
            .clone()
    }

    fn unifyOwnership(&mut self, o1: OwnershipTypeVariable, o2: OwnershipTypeVariable) {
        let o1 = self.substitution.applyOwnershipVar(o1);
        let o2 = self.substitution.applyOwnershipVar(o2);
        self.substitution.addOwnershipVar(o1, o2);
    }

    fn unifyGroup(&mut self, g1: GroupTypeVariable, g2: GroupTypeVariable) {
        let g1 = self.substitution.applyGroupVar(g1);
        let g2 = self.substitution.applyGroupVar(g2);
        self.substitution.addGroupVar(g1, g2);
    }

    fn unify(&mut self, info1: TypeVariableInfo, info2: TypeVariableInfo) {
        self.unifyOwnership(info1.owner, info2.owner);
        self.unifyGroup(info1.group, info2.group);
    }

    fn processBlock(&mut self, block: &Block) {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::FunctionCall(name, args) => {
                    let profile;
                    if let Some(p) = self.groupProfiles.get(name) {
                        profile = Some(p.clone());
                    } else {
                        profile = self.profileStore.getProfile(name);
                    }
                    if let Some(profile) = profile {
                        let mut instantiator = Instantiator::new(
                            self.ownershipInferenceInfo.signature.allocator.clone(),
                        );
                        let profile = instantiator.instantiateProfile(profile);
                        self.ownershipInferenceInfo.signature.allocator = instantiator.allocator;
                        let resInfo = self.getInstructionTypeVariableInfo(instruction.id);
                        for path in &profile.paths {
                            let argId = args[path.index as usize];
                            let argInfo = self.getInstructionTypeVariableInfo(argId);
                            self.unify(argInfo, path.arg);
                        }
                        for (index, argId) in args.iter().enumerate() {
                            let sigArgInfo = profile.signature.args[index];
                            let argInfo = self.getInstructionTypeVariableInfo(*argId);
                            self.unify(argInfo, sigArgInfo);
                        }
                        self.unify(resInfo, profile.signature.result);
                        self.profiles.insert(instruction.id, profile);
                    }
                }
                InstructionKind::DynamicFunctionCall(_, _) => {
                    panic!("dynamic function call in equality")
                }
                InstructionKind::If(_, trueBranch, falseBranch) => {
                    let trueBlock = self.function.getBlockById(*trueBranch);
                    self.processBlock(trueBlock);
                    if let Some(falseBranch) = falseBranch {
                        let falseBlock = self.function.getBlockById(*falseBranch);
                        self.processBlock(falseBlock);
                        self.unify(
                            self.getInstructionTypeVariableInfo(trueBlock.getLastId()),
                            self.getInstructionTypeVariableInfo(falseBlock.getLastId()),
                        );
                    }
                }
                InstructionKind::BlockRef(id) => {
                    let block = self.function.getBlockById(*id);
                    self.processBlock(block);
                    self.unify(
                        self.getInstructionTypeVariableInfo(block.getLastId()),
                        self.getInstructionTypeVariableInfo(instruction.id),
                    );
                }
                InstructionKind::Loop(_, _, _) => todo!(),
                InstructionKind::ValueRef(v, _, _) => {
                    if let Some(members) = self.ownershipInferenceInfo.members.get(&instruction.id)
                    {
                        if let Some(name) = v.getValue() {
                            self.unifyGroup(
                                self.getValueTypeVariableInfo(name).group,
                                members[0].root,
                            );
                        } else {
                            match v {
                                ValueKind::Implicit(id) => {
                                    self.unifyGroup(
                                        self.getInstructionTypeVariableInfo(*id).group,
                                        members[0].root,
                                    );
                                }
                                _ => unreachable!(),
                            }
                        }
                    } else {
                        if let Some(name) = v.getValue() {
                            self.unifyGroup(
                                self.getValueTypeVariableInfo(name).group,
                                self.getInstructionTypeVariableInfo(instruction.id).group,
                            );
                        } else {
                            match v {
                                ValueKind::Implicit(id) => {
                                    self.unifyGroup(
                                        self.getInstructionTypeVariableInfo(*id).group,
                                        self.getInstructionTypeVariableInfo(instruction.id).group,
                                    );
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                InstructionKind::Bind(name, rhs) => self.unify(
                    self.getValueTypeVariableInfo(name.clone()),
                    self.getInstructionTypeVariableInfo(*rhs),
                ),
                InstructionKind::Tuple(_) => todo!(),
                InstructionKind::TupleIndex(_, _) => todo!(),
                InstructionKind::StringLiteral(_) => todo!(),
                InstructionKind::IntegerLiteral(_) => todo!(),
                InstructionKind::CharLiteral(_) => todo!(),
                InstructionKind::Continue(_, _) => todo!(),
                InstructionKind::Break(_, _) => todo!(),
                InstructionKind::Return(_) => todo!(),
                InstructionKind::Ref(_) => todo!(),
            }
        }
        // for i in block.instructions:
        //     if isinstance(i, Instruction.Bind):
        //         self.unifyInstrAndVar(i.rhs, i.name)
    }

    pub fn dump(&self) {
        println!("Sig: {:?}", self.ownershipInferenceInfo.signature);
        for profile in self.profiles.values() {
            println!("Profile sig: {:?}", profile.signature);
            println!("Paths {:?}", profile.paths);
        }
        for block in &self.function.body.as_ref().unwrap().blocks {
            println!("#%s block {}", block.id);
            for i in &block.instructions {
                let tvInfo = self
                    .ownershipInferenceInfo
                    .tvInfos
                    .get(&TypedId::Instruction(i.id))
                    .unwrap();
                if let Some(members) = self.ownershipInferenceInfo.members.get(&i.id) {
                    println!("{} - {} {:?}", i, tvInfo, members);
                } else {
                    println!("{} - {}", i, tvInfo);
                }
            }
        }
    }
}
