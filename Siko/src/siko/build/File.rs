use super::Build::{ArtifactKind, BuildArtifact, BuildEngine};

#[derive(Debug, PartialEq, Eq)]
pub struct File {
    pub name: String,
    pub content: String,
}

pub fn buildFile(name: String, engine: &mut BuildEngine) {
    let content = std::fs::read(name.clone()).expect("Read file failed");
    let content = String::from_utf8(content).expect("string conversion failed");
    engine.add(BuildArtifact::new(ArtifactKind::File(File {
        name: name,
        content: content,
    })));
}
