use crate::location_id::LocationId;
use crate::location_set::LocationSet;

#[derive(Clone)]
pub struct Item {
    pub location: LocationSet,
}

impl Item {
    pub fn new(location: LocationSet) -> Item {
        Item { location: location }
    }
}

#[derive(Debug, Clone)]
pub struct ItemInfo<T> {
    pub item: T,
    pub location_id: LocationId,
}

impl<T> ItemInfo<T> {
    pub fn new(item: T, location_id: LocationId) -> ItemInfo<T> {
        ItemInfo {
            item: item,
            location_id: location_id,
        }
    }
}
