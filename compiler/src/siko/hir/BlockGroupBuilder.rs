use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    vec,
};

use crate::siko::{
    hir::{Block::BlockId, Function::Function, Instruction::InstructionKind},
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct BlockGroupInfo {
    pub groups: Vec<DependencyGroup<BlockId>>,
    pub deps: BTreeMap<BlockId, Vec<BlockId>>,
    pub inverseDeps: BTreeMap<BlockId, Vec<BlockId>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReachabilityMap {
    pub map: BTreeMap<BlockId, BTreeSet<BlockId>>,
}

impl Display for ReachabilityMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (block_id, reachable) in &self.map {
            let reachables: Vec<String> = reachable.iter().map(|b| format!("{:?}", b)).collect();
            writeln!(f, "{:?}: {}", block_id, reachables.join(", "))?;
        }
        Ok(())
    }
}

impl ReachabilityMap {
    pub fn buildReverseReachabilityMap(&self) -> ReachabilityMap {
        let mut reverse: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for block in self.map.keys() {
            reverse.entry(block.clone()).or_insert_with(BTreeSet::new);
        }
        for (from, reachables) in &self.map {
            for to in reachables {
                reverse.entry(to.clone()).or_insert_with(BTreeSet::new).insert(from.clone());
            }
        }
        ReachabilityMap { map: reverse }
    }

    pub fn canReach(&self, from: &BlockId, to: &BlockId) -> bool {
        self.map
            .get(from)
            .map(|reachable| reachable.contains(to))
            .unwrap_or(false)
    }

    pub fn intermediateBlocks(
        &self,
        source: &BlockId,
        target: &BlockId,
        reverseReachability: &ReachabilityMap,
    ) -> Vec<BlockId> {
        let sourceReachable = self.map.get(source);
        let canReachTarget = reverseReachability.map.get(target);
        match (sourceReachable, canReachTarget) {
            (Some(reachable), Some(preds)) => reachable
                .intersection(preds)
                .cloned()
                .collect::<Vec<BlockId>>(),
            _ => Vec::new(),
        }
    }
}

impl BlockGroupInfo {
    pub fn buildReachabilityMap(&self) -> ReachabilityMap {
        // Build a reachability map from block IDs to the set of block IDs they can reach, recursively.
        let mut map: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for (block_id, _) in &self.inverseDeps {
            let mut reachable = BTreeSet::new();
            let mut queue = vec![block_id.clone()];
            while let Some(current) = queue.pop() {
                if !reachable.contains(&current) {
                    reachable.insert(current.clone());
                    let neighbors = self
                        .inverseDeps
                        .get(&current)
                        .expect("Block ID not found in inverseDeps");
                    for neighbor in neighbors {
                        queue.push(neighbor.clone());
                    }
                }
            }
            // Remove the block itself from its reachability set
            reachable.remove(block_id);
            map.insert(block_id.clone(), reachable);
        }
        ReachabilityMap { map }
    }

    pub fn buildReachabilityMap2(&self) -> ReachabilityMap {
        // Build a reachability map from block IDs to the set of block IDs they can reach, recursively.
        // This version uses the SCC groups to optimize the process.
        // First, build a map from block ID to its group (the group will be represented by its index in self.groups)
        let mut group_map: BTreeMap<BlockId, usize> = BTreeMap::new();
        for (index, group) in self.groups.iter().enumerate() {
            for item in &group.items {
                group_map.insert(item.clone(), index);
            }
        }
        // Then, build a map from group to the set of groups it can reach
        let mut group_deps: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        for (block_id, neighbors) in &self.inverseDeps {
            let group = group_map.get(block_id).expect("Block ID not found in group_map");
            let entry = group_deps.entry(*group).or_insert_with(BTreeSet::new);
            for neighbor in neighbors {
                let neighbor_group = group_map
                    .get(neighbor)
                    .expect("Neighbor Block ID not found in group_map");
                if neighbor_group != group {
                    entry.insert(*neighbor_group);
                }
            }
        }
        // Now, for each group, find all reachable groups using DFS
        let mut group_reachability: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        for (index, _) in self.groups.iter().enumerate() {
            let mut reachable = BTreeSet::new();
            let mut stack = vec![index];
            while let Some(current) = stack.pop() {
                if !reachable.contains(&current) {
                    reachable.insert(current);
                    if let Some(neighbors) = group_deps.get(&current) {
                        for neighbor in neighbors {
                            stack.push(*neighbor);
                        }
                    }
                }
            }
            group_reachability.insert(index, reachable);
        }
        // Finally, build the reachability map for each block ID based on its group's reachability
        let mut map: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for (block_id, group) in &group_map {
            let mut reachable_blocks = BTreeSet::new();
            if let Some(reachable_groups) = group_reachability.get(group) {
                for reachable_group in reachable_groups {
                    let group = &self.groups[*reachable_group];
                    for item in &group.items {
                        if item != block_id {
                            reachable_blocks.insert(item.clone());
                            if let Some(neighbors) = group_deps.get(reachable_group) {
                                for neighbor in neighbors {
                                    for neighbor_item in &self.groups[*neighbor].items {
                                        reachable_blocks.insert(neighbor_item.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            map.insert(block_id.clone(), reachable_blocks);
        }
        ReachabilityMap { map }
    }
}

pub struct BlockGroupBuilder<'a> {
    f: &'a Function,
}

impl<'a> BlockGroupBuilder<'a> {
    pub fn new(f: &'a Function) -> Self {
        BlockGroupBuilder { f }
    }

    pub fn process(&self) -> BlockGroupInfo {
        // println!("Building block groups for function: {}", self.f.name);
        // println!("Function: {}", self.f);
        let mut allDeps: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
        let mut inverseDeps: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
        if let Some(body) = &self.f.body {
            for (id, block) in &body.blocks {
                allDeps.entry(id.clone()).or_insert_with(Vec::new);
                inverseDeps.entry(id.clone()).or_insert_with(Vec::new);
                let inner = block.getInner();
                let b = inner.borrow();
                for instruction in &b.instructions {
                    match &instruction.kind {
                        InstructionKind::Jump(_, target) => {
                            allDeps.entry(target.clone()).or_insert_with(Vec::new).push(id.clone());
                            inverseDeps
                                .entry(id.clone())
                                .or_insert_with(Vec::new)
                                .push(target.clone());
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(c.branch.clone())
                                    .or_insert_with(Vec::new)
                                    .push(id.clone());
                                inverseDeps
                                    .entry(id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(c.branch.clone());
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(c.branch.clone())
                                    .or_insert_with(Vec::new)
                                    .push(id.clone());
                                inverseDeps
                                    .entry(id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(c.branch.clone());
                            }
                        }
                        InstructionKind::With(_, info) => {
                            allDeps
                                .entry(info.blockId.clone())
                                .or_insert_with(Vec::new)
                                .push(id.clone());
                            inverseDeps
                                .entry(id.clone())
                                .or_insert_with(Vec::new)
                                .push(info.blockId.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        let groups = processDependencies(&allDeps);
        BlockGroupInfo {
            groups,
            deps: allDeps,
            inverseDeps,
        }
    }
}
