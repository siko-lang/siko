use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::siko::util::Config::Config;

pub struct StageResult<T> {
    pub name: String,
    pub elapsed: Duration,
    pub value: T,
}

struct Core {
    order: Vec<String>,
    stages: BTreeMap<String, Duration>,
}

#[derive(Clone)]
pub struct Runner {
    name: String,
    config: Rc<Config>,
    core: Rc<RefCell<Core>>,
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
        }
    }

    pub fn child(&self, name: &str) -> Runner {
        Runner {
            name: format!("{}.{}", self.name, name),
            config: self.config.clone(),
            core: self.core.clone(),
        }
    }

    pub fn getConfig(&self) -> Config {
        self.config.as_ref().clone()
    }

    pub fn run<T, F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if self.config.passDetails {
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
        if self.config.passDetails {
            println!("Done (took {elapsed:?})");
        }

        let core = &mut self.core.borrow_mut();
        let entry = core.stages.entry(self.name.to_string()).or_insert(Duration::new(0, 0));
        *entry += elapsed;

        value
    }

    pub fn report(&self) {
        if !self.config.passDetails {
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
        println!("{:<width$} | Time (ms)", "Stage", width = max_name_len);
        println!("{}", "-".repeat(max_name_len + 15));

        // rows
        for name in &core.order {
            let elapsed = core.stages.get(name).unwrap();
            let ms = elapsed.as_secs_f64() * 1000.0;
            println!("{:<width$} | {:>9.3} ms", name, ms, width = max_name_len);
        }
    }
}

#[macro_export]
macro_rules! stage {
    ($runner:expr, $body:block) => {{
        $runner.run(|| $body)
    }};
    ($runner:expr, $expr:expr) => {{
        $runner.run($expr)
    }};
}
