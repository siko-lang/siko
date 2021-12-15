use crate::error::TypecheckError;
use siko_ir::class::ClassId;
use siko_ir::instance_resolver::InstanceInfo;
use siko_ir::program::Program;
use siko_ir::types::BaseType;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use siko_location_info::location_id::LocationId;

pub fn check_dependencies_for_single_instance(
    ty: &Type,
    class_id: ClassId,
    location: LocationId,
    program: &mut Program,
    errors: &mut Vec<TypecheckError>,
) {
    let class = program.classes.get(&class_id);
    for dep in &class.constraints {
        let dep_class = program.classes.get(dep);
        let mut unifiers = Vec::new();
        if !program.instance_resolver.check_instance(
            *dep,
            program.get_class_name(*dep),
            &ty,
            location,
            &mut unifiers,
        ) {
            let err = TypecheckError::MissingInstance(dep_class.name.clone(), location);
            errors.push(err);
        }
    }
}

pub fn check_instance_dependencies(program: &mut Program, errors: &mut Vec<TypecheckError>) {
    for (class_id, class_instances) in program.instance_resolver.instance_map.clone() {
        for (_, instances) in class_instances {
            for instance in instances {
                match instance {
                    InstanceInfo::AutoDerived(index) => {
                        let instance =
                            program.instance_resolver.auto_derived_instances[index].clone();
                        check_dependencies_for_single_instance(
                            &instance.ty,
                            class_id,
                            instance.location,
                            program,
                            errors,
                        );
                    }
                    InstanceInfo::UserDefined(ty, _, location) => {
                        check_dependencies_for_single_instance(
                            &ty, class_id, location, program, errors,
                        );
                    }
                }
            }
        }
    }
}

pub fn check_conflicts(errors: &mut Vec<TypecheckError>, program: &Program) {
    let instance_resolver = &program.instance_resolver;
    for (class_id, class_instances) in &instance_resolver.instance_map {
        let class = program.classes.get(&class_id);
        let mut first_generic_instance_location = None;
        if let Some(generic_instances) = class_instances.get(&BaseType::Generic) {
            first_generic_instance_location =
                Some(generic_instances[0].get_location(instance_resolver));
        }
        for (_, instances) in class_instances {
            if let Some(generic_location) = first_generic_instance_location {
                for instance in instances {
                    let other_instance_location = instance.get_location(instance_resolver);
                    if other_instance_location == generic_location {
                        continue;
                    }
                    let err = TypecheckError::ConflictingInstances(
                        class.name.clone(),
                        generic_location,
                        other_instance_location,
                    );
                    errors.push(err);
                }
            } else {
                for (first_index, first_instance) in instances.iter().enumerate() {
                    for (second_index, second_instance) in instances.iter().enumerate() {
                        if first_index < second_index {
                            let first = first_instance.get_type(instance_resolver);
                            let second = second_instance.get_type(instance_resolver);
                            let mut unifier =
                                Unifier::new(instance_resolver.type_var_generator.clone());
                            if unifier.unify(first, second).is_ok() {
                                let err = TypecheckError::ConflictingInstances(
                                    class.name.clone(),
                                    first_instance.get_location(instance_resolver),
                                    second_instance.get_location(instance_resolver),
                                );
                                errors.push(err);
                            }
                        }
                    }
                }
            }
        }
    }
}
