use std::fmt::{Debug, Display};

use crate::siko::backend::drop::{
    Error::Error,
    Path::Path,
    Usage::{Usage, UsageKind},
};

#[derive(Clone)]
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

#[derive(Clone)]
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

    pub fn compress(&self, limit: usize) -> EventSeries {
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

        let mut compressed = self.clone();
        //println!("Compressing events (original): {:?}", compressed.events);
        let mut index = limit;
        while index > 0 {
            if let Event::Assign(path) = compressed.events[index].clone() {
                for i in (0..index) {
                    if usage_overwritten(&compressed.events[i], &path) {
                        compressed.events[i] = Event::Noop;
                    }
                }
            }
            index -= 1;
        }
        //println!("Compressed events: {:?}", compressed.events);
        compressed
    }

    pub fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        for (index, event) in self.events.iter().enumerate() {
            if let Event::Usage(usage) = event {
                let compressed = self.compress(index);
                for prev in compressed.events.iter().take(index) {
                    if let Event::Usage(prevUsage) = prev {
                        if prevUsage.path.sharesPrefixWith(&usage.path) && prevUsage.kind == UsageKind::Move {
                            errors.push(Error::AlreadyMoved {
                                path: usage.path.clone(),
                                prevMove: prevUsage.path.clone(),
                            });
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
