use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::drop::{
        Misc::{MoveKind, PossibleCollision},
        Path::Path,
        SyntaxBlock::SyntaxBlock,
        Usage::{Usage, UsageKind},
    },
    hir::Variable::{Variable, VariableName},
    location::Location::Location,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub live: Vec<Variable>,
    pub usages: Vec<Usage>,
    pub rootBlock: SyntaxBlock,
}

// impl Context {
//     pub fn new() -> Context {
//         let rootBlock = SyntaxBlock::new(format!("0"));
//         Context {
//             live: Vec::new(),
//             usages: Vec::new(),
//             rootBlock,
//         }
//     }

//     pub fn isLive(&self, var: &VariableName) -> bool {
//         for v in &self.live {
//             if v.value == *var {
//                 return true;
//             }
//         }
//         false
//     }

//     pub fn addLive(&mut self, var: &Variable) {
//         // println!(
//         //     "    addLive {} in block {}",
//         //     var.value,
//         //     self.rootBlock.getCurrentBlockId()
//         // );
//         if !self.live.contains(var) {
//             self.live.push(var.clone());
//         }
//         self.usages.retain(|usage| usage.path.root.value != var.value);
//     }

//     pub fn removeSpecificMoveByRoot(&mut self, var: &Variable) {
//         self.usages.retain(|usage| usage.path.root.value != var.value);
//     }

//     pub fn removeSpecificMoveByPath(&mut self, path: &Path) {
//         self.usages.retain(|usage| !usage.path.contains(path));
//     }

//     fn removeSpecificMove(&mut self, var: &Variable) {
//         self.usages.retain(|usage| usage.var != *var);
//     }

//     pub fn isMoved(&self, path: &Path) -> MoveKind {
//         for usage in &self.usages {
//             if usage.path.sharesPrefixWith(path) && usage.isMove() {
//                 //println!("paths {} {}", usage.path, path,);
//                 if path.contains(&usage.path) {
//                     return MoveKind::Fully(usage.var.clone());
//                 } else {
//                     return MoveKind::Partially;
//                 }
//             }
//         }
//         MoveKind::NotMoved
//     }

//     pub fn addUsage(
//         &mut self,
//         paths: &BTreeMap<VariableName, Path>,
//         var: &Variable,
//         kind: UsageKind,
//         collisions: &mut BTreeSet<PossibleCollision>,
//         usages: &mut BTreeMap<Variable, Usage>,
//     ) {
//         //println!("    addUsage {} {}", var, kind);
//         let currentPath = if let Some(path) = paths.get(&var.value) {
//             path.clone()
//         } else {
//             Path::new(var.clone(), Location::empty())
//         };

//         let mut alreadyAdded = false;

//         for usage in &self.usages {
//             //println!("checking {}/{} and {}/{}", usage.var, usage.path, currentPath.root, var);
//             if usage.path.sharesPrefixWith(&currentPath) && usage.isMove() {
//                 collisions.insert(PossibleCollision {
//                     first: usage.var.clone(),
//                     second: var.clone(),
//                 });
//             }
//             if usage.var == *var {
//                 alreadyAdded = true;
//             }
//         }

//         if alreadyAdded {
//             //println!("    already added");
//             return;
//         }

//         let usage = Usage {
//             path: currentPath,
//             kind: kind,
//         };
//         //println!("    addUsage {}", usage);
//         self.usages.push(usage.clone());
//         usages.insert(var.clone(), usage);
//     }

//     pub fn merge(&mut self, terminal_context: &Context) {
//         for var in &terminal_context.live {
//             self.addLive(var);
//         }
//         for usage in &terminal_context.usages {
//             if self.usages.contains(usage) {
//                 continue;
//             }
//             self.usages.push(usage.clone());
//         }
//         self.rootBlock = terminal_context.rootBlock.clone();
//     }
// }

// impl Display for Context {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             " live {}, moved {}, block {}",
//             self.live
//                 .iter()
//                 .map(|v| v.to_string())
//                 .collect::<Vec<String>>()
//                 .join(", "),
//             self.usages
//                 .iter()
//                 .map(|u| u.to_string())
//                 .collect::<Vec<String>>()
//                 .join(", "),
//             self.rootBlock.getCurrentBlockId()
//         )
//     }
// }
