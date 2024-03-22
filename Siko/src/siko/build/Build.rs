use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::rc::Weak;

use crate::siko::location::FileManager::FileManager;
use crate::siko::parser::Parser::Parser;
use crate::siko::resolver::ModuleResolver::LocalNames;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use crate::siko::resolver::Resolver::Resolver;
use crate::siko::syntax::Data::{Class, Enum};
use crate::siko::syntax::Module::{Module, ModuleItem};

#[derive(Debug, PartialEq, Eq)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    File(File),
    Module(Module),
    Class(Class),
    Enum(Enum),
    LocalNames(LocalNames),
    ModuleResolver(ModuleResolver),
}

impl ArtifactKind {
    pub fn asModule(&self) -> &Module {
        match &self {
            ArtifactKind::Module(m) => m,
            _ => panic!("Not a module!"),
        }
    }
}

pub struct BuildArtifact {
    kind: ArtifactKind,
    used: RefCell<Vec<Weak<BuildArtifact>>>,
    usedBy: RefCell<Vec<Weak<BuildArtifact>>>,
    createdBy: RefCell<Vec<Weak<BuildArtifact>>>,
}

impl BuildArtifact {
    pub fn new(kind: ArtifactKind) -> BuildArtifact {
        BuildArtifact {
            kind: kind,
            used: RefCell::new(Vec::new()),
            usedBy: RefCell::new(Vec::new()),
            createdBy: RefCell::new(Vec::new()),
        }
    }

    pub fn process(&self, engine: &mut BuildEngine) {
        println!("Processing {:?}", self.key());
        match &self.kind {
            ArtifactKind::File(f) => {
                let fileId = engine.fileManager.add(f.name.clone());
                let mut parser = Parser::new(fileId, f.name.to_string());
                parser.parse();
                let modules = parser.modules();
                for m in modules {
                    engine.add(BuildArtifact::new(ArtifactKind::Module(m)));
                }
            }
            ArtifactKind::Module(m) => {
                for item in &m.items {
                    match &item {
                        ModuleItem::Class(c) => {
                            engine.add(BuildArtifact::new(ArtifactKind::Class(c.clone())));
                        }
                        ModuleItem::Enum(e) => {
                            engine.add(BuildArtifact::new(ArtifactKind::Enum(e.clone())));
                        }
                        _ => {}
                    }
                }
                engine.enqueue(Key::LocalNames(m.name.toString()));
            }
            ArtifactKind::Class(c) => {}
            ArtifactKind::Enum(e) => {}
            ArtifactKind::LocalNames(_) => {}
            ArtifactKind::ModuleResolver(m) => {}
        }
    }

    pub fn key(&self) -> Key {
        match &self.kind {
            ArtifactKind::File(f) => Key::File(f.name.clone()),
            ArtifactKind::Module(n) => Key::Module(n.name.toString()),
            ArtifactKind::Class(c) => Key::Class(c.name.toString()),
            ArtifactKind::Enum(e) => Key::Enum(e.name.toString()),
            ArtifactKind::LocalNames(l) => Key::LocalNames(l.name.clone()),
            ArtifactKind::ModuleResolver(m) => Key::ModuleResolver(m.name.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    File(String),
    Module(String),
    Class(String),
    Enum(String),
    LocalNames(String),
    ModuleResolver(String),
}

impl Key {
    pub fn priority(&self) -> u32 {
        match self {
            Key::File(_) => 1,
            Key::Module(_) => 2,
            Key::Class(_) => 3,
            Key::Enum(_) => 3,
            Key::LocalNames(_) => 4,
            Key::ModuleResolver(_) => 5,
        }
    }

    fn build(&self, engine: &mut BuildEngine) {
        println!("Building {:?}", self);
        match &self {
            Key::File(name) => {
                let content = std::fs::read(name).expect("Read file failed");
                let content = String::from_utf8(content).expect("string conversion failed");
                engine.add(BuildArtifact::new(ArtifactKind::File(File {
                    name: name.clone(),
                    content: content,
                })));
            }
            Key::LocalNames(n) => {
                let m = engine.get(Key::Module(n.clone()));
                let m = m.kind.asModule();
                let localNames = Resolver::buildLocalNames(m);
                let localNames = LocalNames {
                    name: m.name.toString(),
                    localNames: localNames,
                };
                engine.add(BuildArtifact::new(ArtifactKind::LocalNames(localNames)));
            }
            k => panic!("Building of {:?} NYI", k),
        }
    }
}

pub struct BuildEngine {
    fileManager: FileManager,
    artifacts: BTreeMap<Key, Rc<BuildArtifact>>,
    buildQueue: BTreeMap<u32, Vec<Key>>,
    queue: BTreeMap<u32, Vec<Rc<BuildArtifact>>>,
    current: Option<Rc<BuildArtifact>>,
}

impl BuildEngine {
    pub fn new() -> BuildEngine {
        BuildEngine {
            fileManager: FileManager::new(),
            artifacts: BTreeMap::new(),
            buildQueue: BTreeMap::new(),
            queue: BTreeMap::new(),
            current: None,
        }
    }

    pub fn get(&self, key: Key) -> Rc<BuildArtifact> {
        self.artifacts
            .get(&key)
            .expect("Artifact not found")
            .clone()
    }

    pub fn enqueue(&mut self, key: Key) {
        let items = self
            .buildQueue
            .entry(key.priority())
            .or_insert_with(|| Vec::new());
        items.push(key);
    }

    pub fn add(&mut self, artifact: BuildArtifact) {
        let key = artifact.key();
        if let Some(prev) = self.artifacts.get(&key) {
            let prev = prev.clone();
            let used = prev.used.borrow();
            let mut outdated = false;
            for u in &*used {
                if u.strong_count() == 0 {
                    outdated = true;
                    println!("Outdated by dep {:?}", key);
                }
            }
            if prev.kind != artifact.kind {
                println!("Changed {:?}", key);
                outdated = true;
            }
            if outdated {
                self.artifacts.remove(&key);
            } else {
                return;
            }
        }
        if let Some(parent) = &self.current {
            println!("Added {:?}, current: {:?}", artifact.key(), parent.key());
            let mut createdBy = artifact.createdBy.borrow_mut();
            createdBy.push(Rc::downgrade(parent));
        } else {
            println!("Added {:?}, current: None", artifact.key());
        }
        let artifact = Rc::new(artifact);
        let items = self
            .queue
            .entry(artifact.key().priority())
            .or_insert_with(|| Vec::new());
        items.push(artifact);
    }

    pub fn process(&mut self) {
        loop {
            let mut prio = u32::max_value();
            if self.buildQueue.is_empty() && self.queue.is_empty() {
                break;
            }
            if let Some((p, _)) = self.buildQueue.first_key_value() {
                prio = std::cmp::min(prio, *p);
            }
            if let Some((p, _)) = self.queue.first_key_value() {
                prio = std::cmp::min(prio, *p);
            }
            if let Some(keys) = self.buildQueue.remove(&prio) {
                for k in keys {
                    k.build(self);
                }
                continue;
            }
            if let Some(items) = self.queue.remove(&prio) {
                for item in items {
                    self.current = Some(item.clone());
                    item.process(self);
                    self.current = None;
                    self.artifacts.insert(item.key(), item);
                }
            }
        }
    }
}
