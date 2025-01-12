use std::fmt::Display;

use crate::siko::location::Location::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    pub name: String,
    pub location: Location,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Identifier {
    pub fn toString(&self) -> String {
        format!("{}", self)
    }

    pub fn merge(&mut self, other: Identifier) {
        self.name += &other.name;
        self.location.clone().merge(other.location);
    }

    pub fn dot(&mut self, location: Location) {
        self.name += ".";
        self.location.clone().merge(location);
    }

    pub fn new(s: &str, location: Location) -> Identifier {
        Identifier {
            name: s.to_string(),
            location: location,
        }
    }
}
