use std::collections::BTreeMap;

use crate::siko::{
    ir::Function::{Function, InstructionId, InstructionKind},
    ownership::MemberInfo::{MemberInfo, MemberKind},
    qualifiedname::QualifiedName,
};

use super::{
    dataflowprofile::{
        DataFlowProfile::DataFlowProfile, DataFlowProfileStore::DataFlowProfileStore,
    },
    Signature::FunctionOwnershipSignature,
    TypeVariableInfo::{Substitution, TypeVariableInfo},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
}

pub struct EqualityEngine<'a> {
    function: Function,
    profileStore: &'a DataFlowProfileStore,
    substitution: Substitution,
    groupProfiles: &'a BTreeMap<QualifiedName, DataFlowProfile>,
    profiles: BTreeMap<QualifiedName, DataFlowProfile>,
    tvInfos: BTreeMap<TypedId, TypeVariableInfo>,
}

impl<'a> EqualityEngine<'a> {
    pub fn new(
        function: Function,
        profileStore: &'a DataFlowProfileStore,
        groupProfiles: &'a BTreeMap<QualifiedName, DataFlowProfile>,
    ) -> EqualityEngine<'a> {
        EqualityEngine {
            function: function,
            profileStore: profileStore,
            substitution: Substitution::new(),
            groupProfiles: groupProfiles,
            profiles: BTreeMap::new(),
            tvInfos: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) {
        self.initialize();
    }

    fn initialize(&mut self) {
        if self.function.signature.is_none() {
            let mut signature = FunctionOwnershipSignature::new();
            for param in &self.function.params {
                let tvInfo = signature.allocator.nextTypeVariableInfo();
                self.tvInfos
                    .insert(TypedId::Value(param.getName()), tvInfo.clone());
                signature.args.push(tvInfo);
            }
            self.function.signature = Some(signature);
        }
        for block in &mut self.function.body.as_mut().unwrap().blocks {
            for instruction in &mut block.instructions {
                instruction.tvInfo = Some(
                    self.function
                        .signature
                        .as_mut()
                        .unwrap()
                        .allocator
                        .nextTypeVariableInfo(),
                );
                match &instruction.kind {
                    InstructionKind::ValueRef(_, _, indices) => {
                        let mut root = self
                            .function
                            .signature
                            .as_mut()
                            .unwrap()
                            .allocator
                            .nextGroupVar();
                        for index in indices {
                            let member = MemberInfo::new(
                                root,
                                MemberKind::Field,
                                *index,
                                self.function
                                    .signature
                                    .as_mut()
                                    .unwrap()
                                    .allocator
                                    .nextTypeVariableInfo(),
                            );
                            root = member.info.group;
                            instruction.members.push(member);
                            //i.members.append(member_info)
                            // if len(i.members) != 0:
                            //     i.members[-1].info.group_var = i.tv_info.group_var
                        }
                        instruction.members.last_mut().unwrap().info.group =
                            instruction.tvInfo.unwrap().group;
                    }
                    _ => {}
                }
            }
        }
        // if self.fn.ownership_signature is None:
        //     self.fn.ownership_signature = Signatures.FunctionOwnershipSignature()
        //     self.fn.ownership_signature.name = self.fn.name
        //     self.fn.ownership_signature.result = self.nextTypeVariableInfo()
        //     for param in self.fn.params:
        //         self.fn.ownership_signature.args.append(self.nextTypeVariableInfo())
        // for (index, param) in enumerate(self.fn.params):
        //     self.tv_info_vars[param.name] = self.fn.ownership_signature.args[index]
        // for block in self.fn.body.blocks:
        //     for i in block.instructions:
        //         i.tv_info = self.nextTypeVariableInfo()
        //         if isinstance(i, Instruction.Bind):
        //             tv_info = self.nextTypeVariableInfo()
        //             self.tv_info_vars[i.name] = tv_info
        //         if isinstance(i, Instruction.ValueRef):
        //             root = self.nextGroupVar()
        //             for index in i.indices:
        //                 member_info = MemberInfo.MemberInfo()
        //                 member_info.root = root
        //                 member_info.kind = MemberInfo.MemberKind()
        //                 member_info.kind.type = MemberInfo.FieldKind
        //                 member_info.kind.index = index
        //                 member_info.info = self.nextTypeVariableInfo()
        //                 root = member_info.info.group_var
        //                 i.members.append(member_info)
        //             if len(i.members) != 0:
        //                 i.members[-1].info.group_var = i.tv_info.group_var
        //         if isinstance(i, Instruction.NamedFunctionCall):
        //             if i.ctor:
        //                 for (index, param) in enumerate(i.args):
        //                     member_info = MemberInfo.MemberInfo()
        //                     member_info.root = i.tv_info.group_var
        //                     member_info.kind = MemberInfo.MemberKind()
        //                     member_info.kind.type = MemberInfo.FieldKind
        //                     member_info.kind.index = index
        //                     member_info.info = self.nextTypeVariableInfo()
        //                     i.members.append(member_info)
    }
}
