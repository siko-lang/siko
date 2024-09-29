use crate::siko::{
    ir::{
        Lifetime::{Lifetime, LifetimeInfo},
        Type::Type,
    },
    util::Instantiator::{Allocator, Instantiable, Instantiator},
};

pub struct LifetimeInstantiator {
    instantiator: Instantiator<Lifetime, LifetimeInfo>,
}

impl LifetimeInstantiator {
    pub fn new() -> LifetimeInstantiator {
        LifetimeInstantiator {
            instantiator: Instantiator::new(LifetimeInfo::new()),
        }
    }
}

impl Allocator for LifetimeInfo {
    type Item = Lifetime;

    fn allocate(&mut self) -> Self::Item {
        LifetimeInfo::allocate(self)
    }
}

impl Instantiable for Lifetime {
    type Item = Lifetime;
    fn instantiate<A: Allocator<Item = Lifetime>>(
        &self,
        instantiator: &mut Instantiator<Lifetime, A>,
    ) -> Self {
        instantiator.instantiate(self)
    }
}

impl Instantiable for LifetimeInfo {
    type Item = Lifetime;
    fn instantiate<A: Allocator<Item = Lifetime>>(
        &self,
        instantiator: &mut Instantiator<Lifetime, A>,
    ) -> Self {
        let mut new = LifetimeInfo::new();
        for arg in &self.args {
            new.add(instantiator.instantiate(arg));
        }
        new
    }
}

impl Instantiable for Type {
    type Item = Lifetime;
    fn instantiate<A: Allocator<Item = Lifetime>>(
        &self,
        instantiator: &mut Instantiator<Lifetime, A>,
    ) -> Self {
        match self {
            Type::Named(qn, args, lifetimes) => {
                let lifetimes = lifetimes.instantiate(instantiator);
                Type::Named(qn.clone(), args.clone(), lifetimes)
            }
            Type::Tuple(args) => Type::Tuple(args.instantiate(instantiator)),
            Type::Function(_, _) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Reference(ty, lifetime) => {
                let ty = ty.instantiate(instantiator);
                let lifetime = lifetime.instantiate(instantiator);
                Type::Reference(Box::new(ty), lifetime)
            }
            Type::SelfType => Type::SelfType,
            Type::Never => Type::Never,
        }
    }
}
