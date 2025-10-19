use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::siko::util::Config::Config;

pub struct Statistics {
    pub instanceLookup: u64,
    pub instanceCacheLookup: u64,
    pub instanceCacheHit: u64,
    pub instanceCacheMiss: u64,
    pub maxSCCSizeInUnusedAssignmentEliminator: usize,
    pub maxFixPointIterationCountInAssignmentEliminator: u32,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            instanceLookup: 0,
            instanceCacheLookup: 0,
            instanceCacheHit: 0,
            instanceCacheMiss: 0,
            maxSCCSizeInUnusedAssignmentEliminator: 0,
            maxFixPointIterationCountInAssignmentEliminator: 0,
        }
    }
}

struct StageData {
    total: Duration,
    count: u32,
}

impl StageData {
    fn new() -> Self {
        StageData {
            total: Duration::new(0, 0),
            count: 0,
        }
    }
}

struct Core {
    order: Vec<String>,
    stages: BTreeMap<String, StageData>,
}

#[derive(Clone)]
pub struct Runner {
    name: String,
    config: Rc<Config>,
    core: Rc<RefCell<Core>>,
    pub statistics: Rc<RefCell<Statistics>>,
}

impl Runner {
    pub fn new(config: Config, name: String) -> Self {
        Runner {
            name,
            config: Rc::new(config),
            core: Rc::new(RefCell::new(Core {
                order: Vec::new(),
                stages: BTreeMap::new(),
            })),
            statistics: Rc::new(RefCell::new(Statistics::new())),
        }
    }

    pub fn child(&self, name: &str) -> Runner {
        Runner {
            name: format!("{}.{}", self.name, name),
            config: self.config.clone(),
            core: self.core.clone(),
            statistics: self.statistics.clone(),
        }
    }

    pub fn getConfig(&self) -> Config {
        self.config.as_ref().clone()
    }

    pub fn run<T, F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if self.config.passDetails > 1 {
            print!("{}...\n", self.name);
        }
        let start = Instant::now();
        {
            let core = &mut self.core.borrow_mut();

            if !core.order.contains(&self.name.to_string()) {
                core.order.push(self.name.to_string());
            }
        }

        let value = f();

        let elapsed = start.elapsed();
        if self.config.passDetails > 1 {
            println!("Done (took {elapsed:?})");
        }

        let core = &mut self.core.borrow_mut();
        let entry = core.stages.entry(self.name.to_string()).or_insert_with(StageData::new);
        entry.total += elapsed;
        entry.count += 1;

        value
    }

    pub fn report(&self) {
        if self.config.passDetails == 0 {
            return;
        }
        let core = self.core.borrow();
        if core.stages.is_empty() {
            println!("No stages recorded.");
            return;
        }

        // find max width of stage names
        let max_name_len = core.stages.iter().map(|(name, _)| name.len()).max().unwrap_or(5);

        // header
        let header = format!(
            "{:<width$} |    Time (ms) | Avg. Time (ms) | Count    ",
            "Stage",
            width = max_name_len
        );
        println!("{}", header);
        println!("{}", "-".repeat(header.len()));

        // rows
        for name in &core.order {
            let data = core.stages.get(name).unwrap();
            let ms = data.total.as_secs_f64() * 1000.0;
            let avg = ms / data.count as f64;
            println!(
                "{:<width$} | {:>9.3} ms | {:>11.3} ms | {:>9.3}",
                name,
                ms,
                avg,
                data.count,
                width = max_name_len
            );
        }

        println!("Statistics:");
        let stats = self.statistics.borrow();
        println!("  Instance lookups: {}", stats.instanceLookup);
        println!("  Instance cache lookups: {}", stats.instanceCacheLookup);
        println!("  Instance cache hits: {}", stats.instanceCacheHit);
        println!("  Instance cache misses: {}", stats.instanceCacheMiss);
        println!(
            "  Max SCC size in unused assignment elim: {}",
            stats.maxSCCSizeInUnusedAssignmentEliminator
        );
        println!(
            "  Max fixpoint iterations in unused assignment elim: {}",
            stats.maxFixPointIterationCountInAssignmentEliminator
        );
    }
}
