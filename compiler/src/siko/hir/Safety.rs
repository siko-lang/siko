use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe,
    Regular,
}

impl Display for Safety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Safety::Safe => write!(f, "safe"),
            Safety::Unsafe => write!(f, "unsafe"),
            Safety::Regular => write!(f, "regular"),
        }
    }
}

impl Debug for Safety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
