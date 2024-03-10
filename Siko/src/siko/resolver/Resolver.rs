use crate::siko::syntax::Module::Module;

pub struct Resolver {
    modules: Vec<Module>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: Vec::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.push(m);
    }
}
