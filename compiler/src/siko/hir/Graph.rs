use super::Function::{BlockId, Body, Function};
use super::Instruction::InstructionKind;
use crate::siko::util::Dot::Graph;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug)]
pub enum InstructionFilter {
    All,
    DropflagOnly,
    ControlFlowOnly,
}

pub struct GraphBuilder<'a> {
    function: &'a Function,
    postfix: String,
    filter: InstructionFilter,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(function: &'a Function) -> Self {
        GraphBuilder {
            function,
            postfix: "graph".to_string(),
            filter: InstructionFilter::All,
        }
    }

    pub fn withPostfix(mut self, postfix: &str) -> Self {
        self.postfix = postfix.to_string();
        self
    }

    pub fn withDropFilter(mut self) -> Self {
        self.filter = InstructionFilter::DropflagOnly;
        self
    }

    pub fn withControlFlowOnly(mut self) -> Self {
        self.filter = InstructionFilter::ControlFlowOnly;
        self
    }

    pub fn with_filter(mut self, filter: InstructionFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn build(self) -> Graph {
        let mut graph = Graph::new(format!("{}_{}", self.function.name, self.postfix));

        if let Some(body) = &self.function.body {
            buildGraph(body, &mut graph, self.filter);
        }

        graph
    }
}

// Backward compatibility function
pub fn buildDot(function: &Function, postfix: &str) -> Graph {
    GraphBuilder::new(function).withPostfix(postfix).build()
}

fn buildGraph(body: &Body, graph: &mut Graph, filter: InstructionFilter) {
    let mut block_nodes: BTreeMap<BlockId, String> = BTreeMap::new();
    let mut edges: Vec<(BlockId, BlockId, Option<String>)> = Vec::new();

    // Create nodes for each block
    for (block_id, block) in &body.blocks {
        let title = format!("Block {}", block.id);
        let instructions: Vec<String> = block
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(i, instruction)| {
                if shouldIncludeInstruction(&instruction.kind, filter) {
                    Some(format!("{}: {}", i, instruction.kind.dump()))
                } else {
                    None
                }
            })
            .collect();

        let node_name = graph.addRecordNode(title, instructions);
        block_nodes.insert(*block_id, node_name);
    }

    // Analyze instructions to find edges between blocks
    for (block_id, block) in &body.blocks {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::Jump(_, target_block) => {
                    edges.push((*block_id, *target_block, Some("jump".to_string())));
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    for case in cases {
                        edges.push((*block_id, case.branch, Some(format!("enum_{}", case.index))));
                    }
                }
                InstructionKind::IntegerSwitch(_, cases) => {
                    for case in cases {
                        let label = match &case.value {
                            Some(val) => format!("int_{}", val),
                            None => "default".to_string(),
                        };
                        edges.push((*block_id, case.branch, Some(label)));
                    }
                }
                _ => {}
            }
        }
    }

    // Add sequential flow edges (blocks that flow into the next block)
    addSequentialEdges(body, &mut edges);

    // Add edges to the graph
    for (from_block, to_block, label) in edges {
        if let (Some(from_node), Some(to_node)) = (block_nodes.get(&from_block), block_nodes.get(&to_block)) {
            graph.addEdge(from_node.clone(), to_node.clone(), label);
        }
    }
}

fn addSequentialEdges(body: &Body, edges: &mut Vec<(BlockId, BlockId, Option<String>)>) {
    // Find blocks that don't have explicit jumps and should flow to the next block
    let block_ids: Vec<BlockId> = body.blocks.keys().cloned().collect();

    let mut blocks_with_explicit_exits: BTreeSet<BlockId> = BTreeSet::new();

    // Mark blocks that have explicit exits (jumps, switches, returns)
    for (block_id, block) in &body.blocks {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::Jump(_, _)
                | InstructionKind::EnumSwitch(_, _)
                | InstructionKind::IntegerSwitch(_, _)
                | InstructionKind::Return(_, _) => {
                    blocks_with_explicit_exits.insert(*block_id);
                    break;
                }
                _ => {}
            }
        }
    }

    // Add sequential flow edges for blocks without explicit exits
    for i in 0..block_ids.len() - 1 {
        let current_block = block_ids[i];
        let next_block = block_ids[i + 1];

        if !blocks_with_explicit_exits.contains(&current_block) {
            // Only add sequential edge if it doesn't already exist
            let edge_exists = edges
                .iter()
                .any(|(from, to, _)| *from == current_block && *to == next_block);
            if !edge_exists {
                edges.push((current_block, next_block, Some("flow".to_string())));
            }
        }
    }
}

fn shouldIncludeInstruction(instruction: &InstructionKind, filter: InstructionFilter) -> bool {
    match filter {
        InstructionFilter::All => true,
        InstructionFilter::ControlFlowOnly => {
            // Only show control flow and block structure instructions
            match instruction {
                InstructionKind::Jump(_, _)
                | InstructionKind::EnumSwitch(_, _)
                | InstructionKind::IntegerSwitch(_, _)
                | InstructionKind::Return(_, _)
                | InstructionKind::BlockStart(_)
                | InstructionKind::BlockEnd(_) => true,
                _ => false,
            }
        }
        InstructionFilter::DropflagOnly => {
            // Always include control flow instructions
            match instruction {
                InstructionKind::Jump(_, _)
                | InstructionKind::EnumSwitch(_, _)
                | InstructionKind::IntegerSwitch(_, _)
                | InstructionKind::Return(_, _)
                | InstructionKind::BlockStart(_)
                | InstructionKind::BlockEnd(_) => true,

                // For other instructions, check if they involve dropflag variables
                _ => instructionReferencesDropflag(instruction),
            }
        }
    }
}

fn instructionReferencesDropflag(instruction: &InstructionKind) -> bool {
    let variables = instruction.collectVariables();
    variables.iter().any(|var| var.name().isDropFlag())
}
