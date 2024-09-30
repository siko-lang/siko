use std::collections::BTreeMap;

pub trait Allocator {
    type Item;
    fn allocate(&mut self) -> Self::Item;
}

pub trait Instantiable {
    type Item;
    fn instantiate<A: Allocator<Item = Self::Item>>(
        &self,
        instantiator: &mut Instantiator<Self::Item, A>,
    ) -> Self;
}

pub struct Instantiator<T, A> {
    values: BTreeMap<T, T>,
    allocator: A,
}

impl<T: Clone + Ord, A: Allocator<Item = T>> Instantiator<T, A> {
    pub fn new(allocator: A) -> Instantiator<T, A> {
        Instantiator {
            values: BTreeMap::new(),
            allocator: allocator,
        }
    }

    pub fn instantiate(&mut self, value: &T) -> T {
        match self.values.get(value) {
            Some(new) => new.clone(),
            None => {
                let new = self.allocator.allocate();
                self.values.insert(value.clone(), new.clone());
                new
            }
        }
    }

    pub fn allocate(&mut self) -> T {
        self.allocator.allocate()
    }

    pub fn reset(&mut self) {
        self.values.clear();
    }
}

impl<I, T: Instantiable<Item = I>> Instantiable for Vec<T> {
    type Item = I;
    fn instantiate<A: Allocator<Item = I>>(&self, instantiator: &mut Instantiator<I, A>) -> Self {
        let mut result = Vec::new();
        for i in self {
            result.push(i.instantiate(instantiator));
        }
        result
    }
}

impl<I, T: Instantiable<Item = I>> Instantiable for Option<T> {
    type Item = I;
    fn instantiate<A: Allocator<Item = I>>(&self, instantiator: &mut Instantiator<I, A>) -> Self {
        match self {
            Some(v) => Some(v.instantiate(instantiator)),
            None => None,
        }
    }
}

impl<I, K: Instantiable<Item = I> + Ord, V: Instantiable<Item = I>> Instantiable
    for BTreeMap<K, V>
{
    type Item = I;
    fn instantiate<A: Allocator<Item = I>>(&self, instantiator: &mut Instantiator<I, A>) -> Self {
        let mut result = BTreeMap::new();
        for (key, value) in self {
            let key = key.instantiate(instantiator);
            let value = value.instantiate(instantiator);
            result.insert(key, value);
        }
        result
    }
}
