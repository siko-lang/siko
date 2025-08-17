use std::fmt::Display;

use crate::siko::location::Location::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Fragment {
    pub name: String,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    fragments: Vec<Fragment>,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Identifier {
    pub fn toString(&self) -> String {
        format!("{}", self)
    }

    pub fn merge(&mut self, other: Identifier) {
        for fragment in other.fragments {
            self.fragments.push(fragment);
        }
    }

    pub fn dot(&mut self, location: Location) {
        self.fragments.push(Fragment {
            name: ".".to_string(),
            location,
        });
    }

    pub fn new(s: String, location: Location) -> Identifier {
        Identifier {
            fragments: vec![Fragment { name: s, location }],
        }
    }

    pub fn name(&self) -> String {
        self.fragments
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn location(&self) -> Location {
        if self.fragments.is_empty() {
            Location::empty()
        } else {
            let mut l = self.fragments[0].location.clone();
            for fragment in &self.fragments[1..] {
                l = l.merge(fragment.location.clone())
            }
            l
        }
    }

    pub fn split(&self) -> Option<(Identifier, Identifier)> {
        if self.fragments.len() < 3 {
            None
        } else {
            let first = &self.fragments[0..self.fragments.len() - 2];
            let second = &self.fragments[self.fragments.len() - 1..];
            Some((
                Identifier {
                    fragments: first.to_vec(),
                },
                Identifier {
                    fragments: second.to_vec(),
                },
            ))
        }
    }
}
