use crate::siko::{
    resolver::autoderive::PartialEq::derivePartialEqForEnum,
    syntax::Module::{Module, ModuleItem},
};

pub fn processModule(module: &Module) -> Module {
    //println!("Processing module for auto-derives: {}", module.name);
    let mut instances = Vec::new();
    for item in &module.items {
        match item {
            ModuleItem::Struct(_) => {
                //println!("Found struct: {}", structDef.name);
            }
            ModuleItem::Enum(enumDef) => {
                for derive in &enumDef.derives {
                    //println!("  Derive: {} enum {}", derive.name, enumDef.name);
                    match derive.name.name().as_ref() {
                        "PartialEq" => {
                            let i = derivePartialEqForEnum(enumDef);
                            instances.push(i);
                        }
                        _ => {
                            //
                        }
                    }
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
    }
}
