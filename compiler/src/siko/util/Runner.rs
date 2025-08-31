use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

pub struct StageResult<T> {
    pub name: String,
    pub elapsed: Duration,
    pub value: T,
}

pub struct Runner {
    verbose: bool,
    order: Vec<String>,
    stages: BTreeMap<String, Duration>,
}

impl Runner {
    pub fn new(verbose: bool) -> Self {
        Runner {
            verbose,
            order: Vec::new(),
            stages: BTreeMap::new(),
        }
    }

    pub fn run<T, F>(&mut self, name: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if self.verbose {
            print!("{name}...\n");
        }
        let start = Instant::now();

        if !self.order.contains(&name.to_string()) {
            self.order.push(name.to_string());
        }

        let value = f();

        let elapsed = start.elapsed();
        if self.verbose {
            println!("Done (took {elapsed:?})");
        }

        let entry = self.stages.entry(name.to_string()).or_insert(Duration::new(0, 0));
        *entry += elapsed;

        value
    }

    pub fn report(&self) {
        if !self.verbose {
            return;
        }
        if self.stages.is_empty() {
            println!("No stages recorded.");
            return;
        }

        // find max width of stage names
        let max_name_len = self.stages.iter().map(|(name, _)| name.len()).max().unwrap_or(5);

        // header
        println!("{:<width$} | Time (ms)", "Stage", width = max_name_len);
        println!("{}", "-".repeat(max_name_len + 15));

        // rows
        for name in &self.order {
            let elapsed = self.stages.get(name).unwrap();
            let ms = elapsed.as_secs_f64() * 1000.0;
            println!("{:<width$} | {:>9.3} ms", name, ms, width = max_name_len);
        }
    }
}

#[macro_export]
macro_rules! stage {
    ($runner:expr, $name:expr, $body:block) => {{
        $runner.run($name, || $body)
    }};
    ($runner:expr, $name:expr, $expr:expr) => {{
        $runner.run($name, || $expr)
    }};
}
