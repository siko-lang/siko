use crate::error::ResolverError;
use crate::export_import_pattern::check_item;
use crate::export_import_pattern::check_member;
use crate::export_import_pattern::process_patterns;
use crate::export_import_pattern::MemberPatternKind;
use crate::import::ImportedItemInfo;
use crate::import::ImportedMemberInfo;
use crate::item::Item;
use crate::module::Module;
use siko_location_info::location_id::LocationId;
use siko_syntax::import::ImportKind;
use siko_syntax::program::Program;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ImportMode {
    NameAndNamespace,
    NamespaceOnly,
}

fn get_names(namespace: &str, item_name: &str, mode: ImportMode) -> Vec<String> {
    match mode {
        ImportMode::NamespaceOnly => vec![format!("{}.{}", namespace, item_name)],
        ImportMode::NameAndNamespace => vec![
            item_name.to_string(),
            format!("{}.{}", namespace, item_name),
        ],
    }
}

fn is_hidden(
    item_name: &str,
    source_module: &String,
    all_hidden_items: &mut BTreeMap<String, Vec<HidePattern>>,
) -> bool {
    match all_hidden_items.get_mut(item_name) {
        Some(items) => {
            for item in items.iter_mut() {
                if item.source_module == *source_module && item.name == item_name {
                    item.matched = true;
                    return true;
                }
            }
            return false;
        }
        None => false,
    }
}

fn import_item(
    name: &str,
    source_module: &String,
    namespace: &str,
    mode: ImportMode,
    item: &Item,
    imported_items: &mut BTreeMap<String, Vec<ImportedItemInfo>>,
    implicit: bool,
) {
    let names = get_names(namespace, name, mode);
    for name in &names {
        let iis = imported_items
            .entry(name.clone())
            .or_insert_with(|| Vec::new());
        iis.push(ImportedItemInfo {
            item: item.clone(),
            source_module: source_module.clone(),
            implicit: implicit,
        })
    }
}

fn import_local_items_and_members(
    module_name: &String,
    module: &Module,
    imported_items: &mut BTreeMap<String, Vec<ImportedItemInfo>>,
    imported_members: &mut BTreeMap<String, Vec<ImportedMemberInfo>>,
) {
    for (name, items) in &module.items {
        for item in items {
            import_item(
                name,
                module_name,
                module_name,
                ImportMode::NameAndNamespace,
                item,
                imported_items,
                false,
            );
        }
    }

    for (name, members) in &module.members {
        for member in members {
            let ims = imported_members
                .entry(name.clone())
                .or_insert_with(|| Vec::new());
            ims.push(ImportedMemberInfo {
                member: member.clone(),
                source_module: module_name.clone(),
            })
        }
    }
}

struct HidePattern {
    name: String,
    matched: bool,
    source_module: String,
    location_id: LocationId,
}

impl HidePattern {
    fn new(name: String, source_module: String, location_id: LocationId) -> HidePattern {
        HidePattern {
            name: name,
            matched: false,
            source_module: source_module,
            location_id: location_id,
        }
    }
}

pub fn process_imports(
    modules: &mut BTreeMap<String, Module>,
    program: &Program,
    errors: &mut Vec<ResolverError>,
) {
    let mut all_imported_items = Vec::new();
    let mut all_imported_members = Vec::new();

    for (module_name, module) in modules.iter() {
        // println!("Processing imports for module {}", module_name);
        let mut all_hidden_items = BTreeMap::new();
        let mut imported_items = BTreeMap::new();
        let mut imported_members = BTreeMap::new();

        import_local_items_and_members(
            module_name,
            module,
            &mut imported_items,
            &mut imported_members,
        );

        let ast_module = program.modules.get(&module.id);
        for import_id in &ast_module.imports {
            let import = program.imports.get(import_id);
            if import.module_path == *module_name {
                continue;
            }
            if modules.get(&import.module_path).is_none() {
                let err = ResolverError::ImportedModuleNotFound(
                    import.module_path.clone(),
                    import.get_location(),
                );
                errors.push(err);
                continue;
            }
            match &import.kind {
                ImportKind::Hiding(hidden_items) => {
                    for hidden_item in hidden_items {
                        let hs = all_hidden_items
                            .entry(hidden_item.name.clone())
                            .or_insert_with(|| Vec::new());
                        let hide_pattern = HidePattern::new(
                            hidden_item.name.clone(),
                            import.module_path.clone(),
                            hidden_item.location_id,
                        );
                        hs.push(hide_pattern);
                    }
                }
                ImportKind::ImportList { .. } => {}
            }
        }

        for import_id in &ast_module.imports {
            let import = program.imports.get(import_id);
            if import.module_path == *module_name {
                continue;
            }
            let source_module = match modules.get(&import.module_path.clone()) {
                Some(source_module) => source_module,
                None => {
                    continue;
                }
            };

            match &import.kind {
                ImportKind::Hiding(..) => {}
                ImportKind::ImportList {
                    items,
                    alternative_name,
                } => {
                    let (namespace, mode) = match &alternative_name {
                        Some(n) => (n.clone(), ImportMode::NamespaceOnly),
                        None => (import.module_path.clone(), ImportMode::NameAndNamespace),
                    };

                    let (mut item_patterns, mut member_patterns) = process_patterns(items);

                    let mut local_imported_items = BTreeMap::new();
                    let mut matched_classes = BTreeSet::new();

                    for (item_name, items) in &source_module.exported_items {
                        for item in items {
                            check_item(
                                &mut item_patterns,
                                &mut member_patterns,
                                item_name,
                                item,
                                program,
                                &mut local_imported_items,
                                &mut matched_classes,
                            );
                        }
                    }

                    for (name, items) in &source_module.exported_items {
                        for item in items {
                            if let Item::ClassMember(class_id, _, _) = item {
                                if matched_classes.contains(&class_id) {
                                    let is = local_imported_items
                                        .entry(name.clone())
                                        .or_insert_with(|| BTreeSet::new());
                                    is.insert(item.clone());
                                }
                            }
                        }
                    }

                    for (name, items) in local_imported_items {
                        if is_hidden(&name, &source_module.name, &mut all_hidden_items) {
                            continue;
                        }
                        for item in items {
                            import_item(
                                &name,
                                &source_module.name,
                                &namespace,
                                mode,
                                &item,
                                &mut imported_items,
                                import.implicit,
                            );
                        }
                    }

                    for pattern in item_patterns {
                        match &pattern.name {
                            Some(name) => {
                                if !pattern.matched {
                                    let err = ResolverError::ImportNoMatch(
                                        source_module.name.clone(),
                                        name.clone(),
                                        pattern.location_id.expect("No location"),
                                    );
                                    errors.push(err);
                                }
                            }
                            None => {}
                        }
                    }

                    let mut local_imported_members = BTreeMap::new();

                    for (member_name, members) in &source_module.exported_members {
                        for member in members {
                            check_member(
                                &mut member_patterns,
                                member_name,
                                member,
                                program,
                                &mut local_imported_members,
                            );
                        }
                    }

                    for (name, members) in local_imported_members {
                        if is_hidden(&name, &source_module.name, &mut all_hidden_items) {
                            continue;
                        }
                        for member in members {
                            let ims = imported_members
                                .entry(name.clone())
                                .or_insert_with(|| Vec::new());
                            ims.push(ImportedMemberInfo {
                                member: member.clone(),
                                source_module: module_name.clone(),
                            })
                        }
                    }

                    for pattern_kind in member_patterns {
                        match pattern_kind {
                            MemberPatternKind::ImplicitAll => {}
                            MemberPatternKind::Specific(pattern) => match &pattern.name {
                                Some(name) => {
                                    if !pattern.matched {
                                        let err = ResolverError::ImportNoMatch(
                                            module_name.clone(),
                                            name.clone(),
                                            pattern.location_id,
                                        );
                                        errors.push(err);
                                    }
                                }
                                None => {}
                            },
                        }
                    }
                }
            }
        }

        for (name, hidden_items) in all_hidden_items {
            for hidden_item in hidden_items {
                if !hidden_item.matched {
                    let err = ResolverError::UnusedHiddenItem(
                        name.clone(),
                        hidden_item.source_module.clone(),
                        hidden_item.location_id,
                    );
                    errors.push(err);
                }
            }
        }
        /*
        println!("Module {} imports:", module_name);
        println!(
            "{} imported items {} imported members",
            imported_items.len(),
            imported_members.len(),
        );
        for (name, import) in &imported_items {
            println!("Item: {} => {:?}", name, import);
        }
        for (name, import) in &imported_members {
            println!("Member: {} => {:?}", name, import);
        }
        */
        all_imported_items.push((module_name.clone(), imported_items));
        all_imported_members.push((module_name.clone(), imported_members));
    }

    for (module_name, items) in all_imported_items {
        let module = modules.get_mut(&module_name).expect("Module not found");
        module.imported_items = items;
    }

    for (module_name, members) in all_imported_members {
        let module = modules.get_mut(&module_name).expect("Module not found");
        module.imported_members = members;
    }
}
