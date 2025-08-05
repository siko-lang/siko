use std::fmt::Display;

use crate::siko::backend::drop::Path::Path;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageKind {
    Move,
    Ref,
}

impl Display for UsageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageKind::Move => write!(f, "move"),
            UsageKind::Ref => write!(f, "ref"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Usage {
    pub path: Path,
    pub kind: UsageKind,
}

impl Usage {
    pub fn isMove(&self) -> bool {
        self.kind == UsageKind::Move
    }
}

impl Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.kind, self.path)
    }
}
