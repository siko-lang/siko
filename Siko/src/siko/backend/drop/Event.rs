use std::fmt::{Debug, Display};

use crate::siko::backend::drop::{
    Error::Error,
    Path::Path,
    Usage::{Usage, UsageKind},
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Usage(Usage),
    Assign(Path),
    Noop,
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event: {}", self)
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Usage(usage) => write!(f, "Usage: {}", usage),
            Event::Assign(path) => write!(f, "Assign: {}", path.userPath()),
            Event::Noop => write!(f, "Noop"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventSeries {
    events: Vec<Event>,
}

impl EventSeries {
    pub fn new() -> EventSeries {
        EventSeries { events: Vec::new() }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn prune(&self, limit: usize) -> EventSeries {
        fn usage_overwritten(event: &Event, assignPath: &Path) -> bool {
            if let Event::Usage(usage) = event {
                // println!(
                //     "Checking usage: {} against assign path: {} {}",
                //     usage.path,
                //     assignPath,
                //     usage.path.contains(assignPath)
                // );
                usage.path.contains(assignPath)
            } else {
                false
            }
        }

        let mut pruned: EventSeries = self.clone();
        //println!("Prune events (original): {:?}", pruned.events);
        let mut index = limit;
        while index > 0 {
            if let Event::Assign(path) = pruned.events[index].clone() {
                for i in (0..index) {
                    if usage_overwritten(&pruned.events[i], &path) {
                        pruned.events[i] = Event::Noop;
                    }
                }
            }
            index -= 1;
        }
        //println!("Pruned events: {:?}", pruned.events);
        pruned
    }

    pub fn compress(&self) -> EventSeries {
        //println!("Before compression: {:?}", self.events);
        let mut compressed = self.prune(self.len() - 1);
        compressed.events.retain(|e| *e != Event::Noop);
        //println!("After compression: {:?}", compressed.events);
        compressed
    }

    pub fn validate(&self) -> Vec<Collision> {
        let mut collisions = Vec::new();
        for (index, event) in self.events.iter().enumerate() {
            if let Event::Usage(usage) = event {
                let compressed = self.prune(index);
                for prev in compressed.events.iter().take(index) {
                    if let Event::Usage(prevUsage) = prev {
                        if prevUsage.path.sharesPrefixWith(&usage.path) && prevUsage.kind == UsageKind::Move {
                            collisions.push(Collision {
                                path: usage.path.clone(),
                                prev: prevUsage.path.clone(),
                            });
                        }
                    }
                }
            }
        }
        collisions
    }
}

pub struct Collision {
    pub path: Path,
    pub prev: Path,
}
