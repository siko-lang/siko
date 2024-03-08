use crate::siko::location::Location::*;

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
    pub location: Location,
}

impl Identifier {
    pub fn merge(&mut self, other: Identifier) {
        self.location.merge(other.location);
    }

    pub fn dot(&mut self, location: Location) {
        self.name += ".";
        self.location.merge(location);
    }
}
