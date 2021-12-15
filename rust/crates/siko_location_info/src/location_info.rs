use crate::item::Item;
use crate::location_id::LocationId;
use crate::location_set::LocationSet;
use siko_util::Counter;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct LocationInfo {
    items: BTreeMap<LocationId, Item>,
    id: Counter,
}

impl LocationInfo {
    pub fn new() -> LocationInfo {
        LocationInfo {
            items: BTreeMap::new(),
            id: Counter::new(),
        }
    }

    pub fn add_item(&mut self, item: Item) -> LocationId {
        let id = self.id.next();
        let id = LocationId { id: id };
        self.items.insert(id, item);
        id
    }

    pub fn get_item_location(&self, id: &LocationId) -> &LocationSet {
        &self.items.get(id).expect("Item not found").location
    }
}
