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
            match event {
                Event::Usage(usage) => {
                    // println!(
                    //     "Checking usage: {} against assign path: {} {}",
                    //     usage.path,
                    //     assignPath,
                    //     usage.path.contains(assignPath)
                    // );
                    usage.path.contains(assignPath)
                }
                Event::Assign(path) => path.contains(assignPath),
                _ => false,
            }
        }

        let mut pruned: EventSeries = self.clone();
        //println!("Prune events (original): {:?}", pruned.events);
        let mut index = limit;
        while index > 0 {
            match pruned.events[index].clone() {
                Event::Assign(path) => {
                    for i in (0..index) {
                        if usage_overwritten(&pruned.events[i], &path) {
                            pruned.events[i] = Event::Noop;
                        }
                    }
                }
                Event::Usage(usage) if usage.isMove() => {
                    for i in (0..index) {
                        if usage_overwritten(&pruned.events[i], &usage.path) {
                            pruned.events[i] = Event::Noop;
                        }
                    }
                }
                Event::Usage(usage) if !usage.isMove() => {
                    for i in (0..index) {
                        if let Event::Usage(prevUsage) = &pruned.events[i] {
                            if !prevUsage.isMove() && prevUsage.path == usage.path {
                                pruned.events[i] = Event::Noop;
                            }
                        }
                    }
                }
                _ => {}
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
        //println!("Validating event series: {:?}", self.events);
        let mut collisions = Vec::new();
        for (index, event) in self.events.iter().enumerate() {
            if let Event::Usage(usage) = event {
                let compressed = self.prune(index);
                for prev in compressed.events.iter().take(index) {
                    if let Event::Usage(prevUsage) = prev {
                        if prevUsage.path.sharesPrefixWith(&usage.path) && prevUsage.kind == UsageKind::Move {
                            //println!(
                            //    "Collision detected: {} with previous usage: {}",
                            //    usage.path, prevUsage.path
                            //);
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

impl Display for EventSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventSeries: [")?;
        for (i, event) in self.events.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", event)?;
        }
        write!(f, "]")
    }
}

pub struct Collision {
    pub path: Path,
    pub prev: Path,
}
