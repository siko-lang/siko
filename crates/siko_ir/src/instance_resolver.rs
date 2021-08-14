use crate::class::ClassId;
use crate::class::InstanceId;
use crate::type_var_generator::TypeVarGenerator;
use crate::types::BaseType;
use crate::types::Type;
use crate::unifier::Unifier;
use siko_location_info::location_id::LocationId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum ResolutionResult {
    UserDefined(InstanceId),
    AutoDerived,
}

#[derive(Clone)]
pub struct AutoDerivedInstance {
    pub ty: Type,
    pub location: LocationId,
}

#[derive(Clone)]
pub enum InstanceInfo {
    UserDefined(Type, InstanceId, LocationId),
    AutoDerived(usize),
}

impl InstanceInfo {
    pub fn get_type<'a, 'b: 'a>(&'b self, instance_resolver: &'a InstanceResolver) -> &'a Type {
        match self {
            InstanceInfo::UserDefined(ty, _, _) => &ty,
            InstanceInfo::AutoDerived(index) => {
                &instance_resolver.get_auto_derived_instance(*index).ty
            }
        }
    }

    pub fn get_location(&self, instance_resolver: &InstanceResolver) -> LocationId {
        match self {
            InstanceInfo::UserDefined(_, _, id) => *id,
            InstanceInfo::AutoDerived(index) => {
                instance_resolver.get_auto_derived_instance(*index).location
            }
        }
    }
}

pub struct InstanceResolver {
    pub instance_map: BTreeMap<ClassId, BTreeMap<BaseType, Vec<InstanceInfo>>>,
    pub auto_derived_instances: Vec<AutoDerivedInstance>,
    cache: Rc<RefCell<BTreeMap<(ClassId, Type), ResolutionResult>>>,
    pub type_var_generator: TypeVarGenerator,
}

impl InstanceResolver {
    pub fn new(type_var_generator: TypeVarGenerator) -> InstanceResolver {
        InstanceResolver {
            instance_map: BTreeMap::new(),
            auto_derived_instances: Vec::new(),
            cache: Rc::new(RefCell::new(BTreeMap::new())),
            type_var_generator: type_var_generator,
        }
    }

    pub fn get_auto_derived_instance(&self, index: usize) -> &AutoDerivedInstance {
        &self.auto_derived_instances[index]
    }

    pub fn update_auto_derived_instance(&mut self, index: usize, instance: AutoDerivedInstance) {
        self.auto_derived_instances[index] = instance;
    }

    pub fn add_user_defined(
        &mut self,
        class_id: ClassId,
        instance_ty: Type,
        instance_id: InstanceId,
        location_id: LocationId,
    ) {
        let class_instances = self
            .instance_map
            .entry(class_id)
            .or_insert_with(|| BTreeMap::new());
        let instances = class_instances
            .entry(instance_ty.get_base_type())
            .or_insert_with(|| Vec::new());
        instances.push(InstanceInfo::UserDefined(
            instance_ty,
            instance_id,
            location_id,
        ));
    }

    pub fn add_auto_derived(
        &mut self,
        class_id: ClassId,
        instance_ty: Type,
        location_id: LocationId,
    ) -> usize {
        let class_instances = self
            .instance_map
            .entry(class_id)
            .or_insert_with(|| BTreeMap::new());
        let instances = class_instances
            .entry(instance_ty.get_base_type())
            .or_insert_with(|| Vec::new());
        let instance = AutoDerivedInstance {
            ty: instance_ty,
            location: location_id,
        };
        let index = self.auto_derived_instances.len();
        self.auto_derived_instances.push(instance);
        instances.push(InstanceInfo::AutoDerived(index));
        index
    }

    fn has_instance(&self, ty: &Type, class_id: ClassId) -> Option<Unifier> {
        let base_type = ty.get_base_type();
        if let Some(class_instances) = self.instance_map.get(&class_id) {
            if let Some(instances) = class_instances.get(&base_type) {
                for instance in instances {
                    let mut unifier = Unifier::new(self.type_var_generator.clone());
                    match instance {
                        InstanceInfo::AutoDerived(index) => {
                            let instance = self.get_auto_derived_instance(*index);
                            if unifier.unify(ty, &instance.ty).is_ok() {
                                if ty.is_concrete_type() {
                                    let result = ResolutionResult::AutoDerived;
                                    let mut cache = self.cache.borrow_mut();
                                    cache.insert((class_id, ty.clone()), result);
                                }
                                return Some(unifier);
                            }
                        }
                        InstanceInfo::UserDefined(instance_ty, instance_id, _) => {
                            if unifier.unify(ty, instance_ty).is_ok() {
                                if ty.is_concrete_type() {
                                    let result = ResolutionResult::UserDefined(*instance_id);
                                    let mut cache = self.cache.borrow_mut();
                                    cache.insert((class_id, ty.clone()), result);
                                }
                                return Some(unifier);
                            }
                        }
                    }
                }
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn check_instance(
        &mut self,
        class_id: ClassId,
        _name: String,
        ty: &Type,
        location_id: LocationId,
        unifiers: &mut Vec<(Unifier, LocationId)>,
    ) -> bool {
        //println!("Checking instance {} {}", name, ty);
        if let Type::Var(_, constraints) = ty {
            if constraints.contains(&class_id) {
                return true;
            } else {
                let mut new_constraints = constraints.clone();
                new_constraints.push(class_id);
                let new_type = Type::Var(self.type_var_generator.get_new_index(), new_constraints);
                let mut unifier = Unifier::new(self.type_var_generator.clone());
                let r = unifier.unify(&new_type, ty);
                assert!(r.is_ok());
                unifiers.push((unifier, location_id));
                return true;
            }
        }
        if let Some(unifier) = self.has_instance(&ty, class_id) {
            let constraints = unifier.get_substitution().get_constraints();
            for constraint in constraints {
                if constraint.ty.get_base_type() == BaseType::Generic {
                    unimplemented!();
                } else {
                    if !self.check_instance(
                        constraint.class_id,
                        "<Other>".to_string(),
                        &constraint.ty,
                        location_id,
                        unifiers,
                    ) {
                        println!("Failed2");
                        return false;
                    }
                }
            }
            return true;
        } else {
            println!("Failed!");
            return false;
        }
    }

    pub fn get(&self, class_id: ClassId, ty: Type) -> ResolutionResult {
        {
            let cache = self.cache.borrow();
            if let Some(r) = cache.get(&(class_id, ty.clone())) {
                return *r;
            }
        }
        let r = self.has_instance(&ty, class_id);
        assert!(r.is_some());
        return self.get(class_id, ty);
    }
}
