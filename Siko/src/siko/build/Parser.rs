use crate::siko::{
    parser::Parser::Parser, qualifiedname::QualifiedName, syntax::Data::Class as SyntaxClass,
};

use super::Build::{ArtifactKind, BuildArtifact, BuildEngine};

#[derive(Debug, PartialEq, Eq)]
pub struct Class {
    pub name: QualifiedName,
    pub c: SyntaxClass,
}

pub fn processFile(name: String, engine: &mut BuildEngine) {
    let fileId = engine.fileManager.add(name.clone());
    let mut parser = Parser::new(fileId, name.to_string());
    parser.parse();
    let modules = parser.modules();
    for m in modules {
        engine.add(BuildArtifact::new(ArtifactKind::Module(m)));
    }
}
