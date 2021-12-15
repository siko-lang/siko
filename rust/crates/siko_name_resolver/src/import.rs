use crate::item::DataMember;
use crate::item::Item;

#[derive(Debug, Clone)]
pub struct ImportedItemInfo {
    pub item: Item,
    pub source_module: String,
    pub implicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Type,
    Value,
}

fn filter_implicits(items: Vec<&ImportedItemInfo>) -> Vec<&ImportedItemInfo> {
    let mut implicit_count = 0;
    for item in &items {
        if item.implicit {
            implicit_count += 1;
        }
    }
    let items: Vec<_> = if implicit_count != items.len() {
        items
            .iter()
            .filter(|item| !item.implicit)
            .cloned()
            .collect()
    } else {
        items.to_vec()
    };
    items
}

impl ImportedItemInfo {
    pub fn resolve_ambiguity(
        items: &[ImportedItemInfo],
        namespace: Namespace,
    ) -> Option<&ImportedItemInfo> {
        match namespace {
            Namespace::Type => {
                let items: Vec<_> = items
                    .into_iter()
                    .filter(|item| item.item.is_type_level())
                    .collect();
                let mut items = filter_implicits(items);
                if items.len() > 1 {
                    return None;
                }
                return items.pop();
            }
            Namespace::Value => {
                let items: Vec<_> = items
                    .into_iter()
                    .filter(|item| item.item.is_value_level())
                    .collect();
                let mut items = filter_implicits(items);
                if items.len() > 1 {
                    return None;
                }
                return items.pop();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportedMemberInfo {
    pub member: DataMember,
    pub source_module: String,
}
