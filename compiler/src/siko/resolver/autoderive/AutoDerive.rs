use crate::siko::{
    resolver::autoderive::{
        Clone::{deriveCloneForEnum, deriveCloneForStruct},
        Copy::{deriveCopyForEnum, deriveCopyForStruct},
        Discriminator::{deriveDiscriminatorForEnum, deriveDiscriminatorForStruct},
        Eq::{deriveEqForEnum, deriveEqForStruct},
        FromInt::deriveFromIntForEnum,
        Ord::{deriveOrdForEnum, deriveOrdForStruct},
        PartialEq::{derivePartialEqForEnum, derivePartialEqForStruct},
        PartialOrd::{derivePartialOrdForEnum, derivePartialOrdForStruct},
    },
    syntax::Module::{Module, ModuleItem},
};

pub fn processModule(module: &Module) -> Module {
    //println!("Processing module for auto-derives: {}", module.name);
    let mut instances = Vec::new();
    for item in &module.items {
        match item {
            ModuleItem::Struct(structDef) => {
                for derive in &structDef.derives {
                    //println!("  Derive: {} struct {}", derive.name, structDef.name);
                    match derive.name.name().as_ref() {
                        "Clone" => {
                            let i = deriveCloneForStruct(structDef, &module.name.name());
                            instances.push(i);
                        }
                        "Copy" => {
                            let i = deriveCopyForStruct(structDef);
                            instances.push(i);
                        }
                        "Eq" => {
                            let i = deriveEqForStruct(structDef);
                            instances.push(i);
                        }
                        "PartialEq" => {
                            let i = derivePartialEqForStruct(structDef);
                            instances.push(i);
                        }
                        "PartialOrd" => {
                            let i = derivePartialOrdForStruct(structDef);
                            instances.push(i);
                        }
                        "Ord" => {
                            let i = deriveOrdForStruct(structDef);
                            instances.push(i);
                        }
                        "Discriminator" => {
                            let i = deriveDiscriminatorForStruct(structDef);
                            instances.push(i);
                        }
                        _ => {
                            //
                        }
                    }
                }
            }
            ModuleItem::Enum(enumDef) => {
                let mut discriminatorNeeded = false;
                let mut discriminatorPresent = false;
                for derive in &enumDef.derives {
                    //println!("  Derive: {} enum {}", derive.name, enumDef.name);
                    match derive.name.name().as_ref() {
                        "PartialEq" => {
                            let i = derivePartialEqForEnum(enumDef);
                            instances.push(i);
                        }
                        "Eq" => {
                            let i = deriveEqForEnum(enumDef);
                            instances.push(i);
                        }
                        "Clone" => {
                            let i = deriveCloneForEnum(enumDef);
                            instances.push(i);
                        }
                        "Copy" => {
                            let i = deriveCopyForEnum(enumDef);
                            instances.push(i);
                        }
                        "PartialOrd" => {
                            let i = derivePartialOrdForEnum(enumDef);
                            instances.push(i);
                            discriminatorNeeded = true;
                        }
                        "Ord" => {
                            let i = deriveOrdForEnum(enumDef);
                            instances.push(i);
                            discriminatorNeeded = true;
                        }
                        "Discriminator" => {
                            let i = deriveDiscriminatorForEnum(enumDef);
                            instances.push(i);
                            discriminatorPresent = true;
                        }
                        "FromInt" => {
                            let i = deriveFromIntForEnum(enumDef);
                            instances.push(i);
                        }
                        _ => {
                            //
                        }
                    }
                }
                if discriminatorNeeded && !discriminatorPresent {
                    let i = deriveDiscriminatorForEnum(enumDef);
                    instances.push(i);
                }
            }
            _ => {}
        }
    }
    let items = module
        .items
        .iter()
        .cloned()
        .chain(instances.into_iter().map(ModuleItem::Instance))
        .collect();
    Module {
        name: module.name.clone(),
        items,
        attributes: module.attributes.clone(),
    }
}
