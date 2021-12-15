use crate::item::DataMember;
use crate::item::Item;
use siko_location_info::location_id::LocationId;
use siko_syntax::class::ClassId;
use siko_syntax::export_import::EIItem;
use siko_syntax::export_import::EIList;
use siko_syntax::export_import::EIMember;
use siko_syntax::program::Program;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct ItemPattern {
    pub name: Option<String>,
    group: bool,
    pub matched: bool,
    pub location_id: Option<LocationId>,
}

impl ItemPattern {
    fn new(name: Option<String>, group: bool, location_id: Option<LocationId>) -> ItemPattern {
        ItemPattern {
            name: name,
            group: group,
            matched: false,
            location_id: location_id,
        }
    }
}

#[derive(Debug)]
pub struct MemberPattern {
    group_name: String,
    pub name: Option<String>,
    pub matched: bool,
    pub location_id: LocationId,
}

#[derive(Debug)]
pub enum MemberPatternKind {
    ImplicitAll,
    Specific(MemberPattern),
}

impl MemberPattern {
    fn new(group_name: String, name: Option<String>, location_id: LocationId) -> MemberPattern {
        MemberPattern {
            group_name: group_name,
            name: name,
            matched: false,
            location_id: location_id,
        }
    }
}

pub fn process_patterns(item_list: &EIList) -> (Vec<ItemPattern>, Vec<MemberPatternKind>) {
    let mut item_patterns = Vec::new();
    let mut member_patterns = Vec::new();
    match item_list {
        EIList::ImplicitAll => {
            item_patterns.push(ItemPattern::new(None, false, None));
            member_patterns.push(MemberPatternKind::ImplicitAll);
        }
        EIList::Explicit(pattern_items) => {
            for pattern_item in pattern_items {
                let item = &pattern_item.item;
                match item {
                    EIItem::Named(entity_name) => {
                        item_patterns.push(ItemPattern::new(
                            Some(entity_name.clone()),
                            false,
                            Some(pattern_item.location_id),
                        ));
                    }
                    EIItem::Group(group) => {
                        item_patterns.push(ItemPattern::new(
                            Some(group.name.clone()),
                            true,
                            Some(pattern_item.location_id),
                        ));
                        for member_info in &group.members {
                            match &member_info.member {
                                EIMember::All => {
                                    let pattern = MemberPattern::new(
                                        group.name.clone(),
                                        None,
                                        member_info.location_id,
                                    );
                                    member_patterns.push(MemberPatternKind::Specific(pattern));
                                }
                                EIMember::Specific(name) => {
                                    let pattern = MemberPattern::new(
                                        group.name.clone(),
                                        Some(name.clone()),
                                        member_info.location_id,
                                    );
                                    member_patterns.push(MemberPatternKind::Specific(pattern));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (item_patterns, member_patterns)
}

fn match_item(name: &str, group: bool, item: &Item, program: &Program) -> bool {
    match item {
        Item::Function(id, _) => {
            let function = program.functions.get(&id);
            function.name == name && !group
        }
        Item::Record(id, _) => {
            let record = program.records.get(&id);
            record.name == name
        }
        Item::Adt(id, _) => {
            let adt = program.adts.get(&id);
            adt.name == name
        }
        Item::Variant(..) => {
            // cannot match on a single variant
            false
        }
        Item::Class(id, _) => {
            let class = program.classes.get(&id);
            class.name == name && !group
        }
        Item::ClassMember(_, _, _) => false,
    }
}

fn match_member(
    group_name: &str,
    name: Option<&String>,
    member: &DataMember,
    program: &Program,
) -> bool {
    match member {
        DataMember::RecordField(field) => {
            let record = program.records.get(&field.record_id);
            for field_id in &record.fields {
                let field = program.record_fields.get(field_id);
                if record.name == group_name {
                    if let Some(n) = name {
                        if *n == field.name {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        DataMember::Variant(variant) => {
            let adt = program.adts.get(&variant.adt_id);
            let ast_variant = program.variants.get(&variant.variant_id);
            if adt.name == group_name {
                if let Some(n) = name {
                    if *n == ast_variant.name {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
    }
    false
}

pub fn check_item(
    item_patterns: &mut Vec<ItemPattern>,
    member_patterns: &mut Vec<MemberPatternKind>,
    item_name: &str,
    item: &Item,
    program: &Program,
    matched_items: &mut BTreeMap<String, BTreeSet<Item>>,
    matched_classes: &mut BTreeSet<ClassId>,
) {
    let mut matched_item = false;
    for pattern in item_patterns.iter_mut() {
        match &pattern.name {
            Some(name) => {
                if match_item(name, pattern.group, item, program) {
                    matched_item = true;
                    pattern.matched = true;
                }
            }
            None => {
                // implicit
                matched_item = true;
            }
        }
    }
    for pattern_kind in member_patterns.iter_mut() {
        match item {
            Item::Variant(adt_id, variant_id, _, _) => match pattern_kind {
                MemberPatternKind::ImplicitAll => {
                    matched_item = true;
                }
                MemberPatternKind::Specific(pattern) => {
                    let adt = program.adts.get(&adt_id);
                    let ast_variant = program.variants.get(&variant_id);
                    if adt.name == pattern.group_name {
                        if let Some(n) = &pattern.name {
                            if *n == ast_variant.name {
                                matched_item = true;
                            }
                        } else {
                            matched_item = true;
                        }
                    }
                }
            },
            _ => {}
        }
    }
    if matched_item {
        if let Item::Class(id, _) = item {
            matched_classes.insert(*id);
        }
        let items = matched_items
            .entry(item_name.to_string())
            .or_insert_with(|| BTreeSet::new());
        items.insert(item.clone());
    }
}

pub fn check_member(
    member_patterns: &mut Vec<MemberPatternKind>,
    member_name: &str,
    member: &DataMember,
    program: &Program,
    matched_members: &mut BTreeMap<String, Vec<DataMember>>,
) {
    let mut matched_member = false;
    for pattern_kind in member_patterns.iter_mut() {
        match pattern_kind {
            MemberPatternKind::ImplicitAll => matched_member = true,
            MemberPatternKind::Specific(pattern) => {
                if match_member(&pattern.group_name, pattern.name.as_ref(), member, program) {
                    matched_member = true;
                    pattern.matched = true;
                }
            }
        }
    }
    if matched_member {
        let members = matched_members
            .entry(member_name.to_string())
            .or_insert_with(|| Vec::new());
        members.push(member.clone());
    }
}
