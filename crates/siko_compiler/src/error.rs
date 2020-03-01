use colored::*;
use siko_constants::MAIN_FUNCTION;
use siko_constants::MAIN_MODULE_NAME;
use siko_location_info::error_context::ErrorContext;
use siko_location_info::file_manager::FileManager;
use siko_location_info::filepath::FilePath;
use siko_location_info::location::Location;
use siko_location_info::location_id::LocationId;
use siko_location_info::location_set::LocationSet;
use siko_name_resolver::error::Error as ResolverErrorContainer;
use siko_name_resolver::error::ResolverError;
use siko_parser::error::LexerError;
use siko_parser::error::ParseError;
use siko_type_checker::error::Error as TypecheckErrorContainer;
use siko_type_checker::error::TypecheckError;
use siko_util::format_list;
use std::cmp;
use std::convert::From;
use std::io::Error as IoError;

fn s_from_range(chars: &[char], start: usize, end: usize) -> String {
    let start = cmp::min(start, end);
    let subs = &chars[start..end];
    let s: String = subs.iter().collect();
    s
}

fn print_location_set(file_manager: &FileManager, location_set: &LocationSet) {
    let input = file_manager.content(&location_set.file_path);
    let lines: Vec<_> = input.lines().collect();
    let mut first = true;
    let pipe = "|";
    let mut last_line = 0;
    for (line_index, ranges) in &location_set.lines {
        last_line = *line_index;
        if first {
            first = false;
            eprintln!(
                "{}{}:{}",
                "-- ".blue(),
                location_set.file_path.path.green(),
                format!("{}", line_index + 1).green()
            );
            if *line_index != 0 {
                let line = &lines[*line_index - 1];
                eprintln!("{} {}", pipe.blue(), line);
            }
        }
        let line = &lines[*line_index];
        let chars: Vec<_> = line.chars().collect();
        let first = s_from_range(&chars[..], 0, ranges[0].start);
        eprint!("{} {}", pipe.blue(), first);
        for (index, range) in ranges.iter().enumerate() {
            let s = s_from_range(&chars[..], range.start, range.end);
            eprint!("{}", s.yellow());
            if index < ranges.len() - 1 {
                let s = s_from_range(&chars[..], range.end, ranges[index + 1].start);
                eprint!("{}", s);
            }
        }
        let last = s_from_range(&chars[..], ranges[ranges.len() - 1].end, chars.len());
        eprintln!("{}", last);
    }
    if last_line + 1 < lines.len() {
        let line = &lines[last_line + 1];
        eprintln!("{} {}", pipe.blue(), line);
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    LexerError(Vec<LexerError>),
    ParseError(ParseError),
    ResolverError(ResolverErrorContainer),
    TypecheckError(TypecheckErrorContainer),
    RuntimeError(String, LocationId),
}

impl Error {
    fn report_location(file_manager: &FileManager, file_path: &FilePath, location: &Location) {
        let input = file_manager.content(file_path);
        let lines: Vec<_> = input.lines().collect();
        eprintln!(
            "--{}:{}",
            file_path.path.green(),
            format!("{}", location.line + 1).green()
        );
        let line = &lines[location.line];
        let chars: Vec<_> = line.chars().collect();
        let first = s_from_range(&chars[..], 0, location.span.start);
        eprint!("{}", first);
        let s = s_from_range(&chars[..], location.span.start, location.span.end);
        eprint!("{}", s.red());
        let last = s_from_range(&chars[..], location.span.end, chars.len());
        eprintln!("{}", last);
    }

    fn report_error_base(
        msg: &str,
        file_manager: &FileManager,
        file_path: &FilePath,
        location: &Location,
    ) {
        let error = "ERROR:";
        eprintln!("{} {}", error.red(), msg);
        Error::report_location(file_manager, file_path, location);
    }

    pub fn report_error(&self, context: &ErrorContext) {
        let file_manager = &context.file_manager;
        let location_info = &context.location_info;
        let error = "ERROR:";
        match self {
            Error::LexerError(errors) => {
                for err in errors {
                    match err {
                        LexerError::General(msg, file_path, location) => {
                            Error::report_error_base(msg, file_manager, file_path, location);
                        }
                        LexerError::UnsupportedCharacter(c, location) => {
                            Error::report_error_base(
                                &format!(
                                    "{} unsupported character {}",
                                    error.red(),
                                    format!("{}", c).yellow()
                                ),
                                file_manager,
                                &location.file_path,
                                &location.location,
                            );
                        }
                    }
                }
            }
            Error::ParseError(err) => {
                Error::report_error_base(&err.msg, file_manager, &err.file_path, &err.location);
            }
            Error::ResolverError(errs) => {
                for err in &errs.errors {
                    match err {
                        ResolverError::ModuleConflict(errors) => {
                            for (name, ids) in errors.iter() {
                                eprintln!(
                                    "{} module name {} defined more than once",
                                    error.red(),
                                    name.yellow()
                                );
                                for id in ids.iter() {
                                    let location_set = location_info.get_item_location(id);
                                    print_location_set(file_manager, location_set);
                                }
                            }
                        }
                        ResolverError::ImportedModuleNotFound(name, id) => {
                            eprintln!(
                                "{} imported module {} does not exist",
                                error.red(),
                                name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnknownTypeName(var_name, id) => {
                            eprintln!("{} unknown type name {}", error.red(), var_name.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnknownTypeArg(var_name, id) => {
                            eprintln!(
                                "{} unknown type argument {}",
                                error.red(),
                                var_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::TypeArgumentConflict(args, id) => {
                            eprintln!(
                                "{} type argument(s) are not unique: {}",
                                error.red(),
                                format_list(args).yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ArgumentConflict(args, id) => {
                            eprintln!(
                                "{} argument(s) are not unique: {}",
                                error.red(),
                                format_list(args).yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::LambdaArgumentConflict(args, id) => {
                            eprintln!(
                                "{} lambda argument(s) {} are not unique",
                                error.red(),
                                format_list(args).yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnknownFunction(var_name, id) => {
                            eprintln!("{} unknown function {}", error.red(), var_name.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::AmbiguousName(var_name, id) => {
                            eprintln!("{} ambiguous name {}", error.red(), var_name.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnusedTypeArgument(arg, id) => {
                            eprintln!("{} unused type argument: {}", error.red(), arg.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::InternalModuleConflicts(module_name, name, locations) => {
                            eprintln!(
                                "{} conflicting items named {} in module {}",
                                error.red(),
                                name.yellow(),
                                module_name.yellow()
                            );
                            for id in locations {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::RecordFieldNotUnique(record_name, item_name, id) => {
                            eprintln!(
                                "{} field name {} is not unique in record {}",
                                error.red(),
                                item_name.yellow(),
                                record_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::VariantNotUnique(adt_name, variant_name, id) => {
                            eprintln!(
                                "{} variant name {} is not unique in type {}",
                                error.red(),
                                variant_name.yellow(),
                                adt_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ExportNoMatch(module_name, entity_name, id) => {
                            eprintln!(
                                "{} item {} does not export anything in module {}",
                                error.red(),
                                entity_name.yellow(),
                                module_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ImportNoMatch(module_name, entity_name, id) => {
                            eprintln!(
                                "{} item {} does not import anything from module {}",
                                error.red(),
                                entity_name.yellow(),
                                module_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::IncorrectTypeArgumentCount(
                            type_name,
                            expected,
                            found,
                            id,
                        ) => {
                            eprintln!(
                                "{} incorrect type argument count for type {}",
                                error.red(),
                                type_name.yellow(),
                            );
                            let expected = format!("{}", expected);
                            let found = format!("{}", found);
                            eprintln!("Expected: {}", expected.yellow());
                            eprintln!("Found:    {}", found.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NameNotType(name, id) => {
                            eprintln!("{} name is not a type {}", error.red(), name.yellow(),);
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnusedHiddenItem(hidden_item, module_name, id) => {
                            eprintln!(
                                "{} hidden item {} does not hide anything from module {}",
                                error.red(),
                                hidden_item.yellow(),
                                module_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::UnknownFieldName(field_name, id) => {
                            eprintln!("{} unknown field name {}", error.red(), field_name.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NotIrrefutablePattern(id) => {
                            eprintln!("{} not irrefutable pattern", error.red(),);
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NotRecordType(name, id) => {
                            eprintln!("{} {} is not a record type", error.red(), name.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NoSuchField(record, field_name, id) => {
                            eprintln!(
                                "{} there is no field named {} in {}",
                                error.red(),
                                field_name.yellow(),
                                record.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::MissingFields(missing_fields, id) => {
                            eprintln!(
                                "{} missing initialization of the following field(s): {}",
                                error.red(),
                                format_list(missing_fields).yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::FieldsInitializedMultipleTimes(
                            fields_initialized_twice,
                            id,
                        ) => {
                            eprintln!(
                                "{} the following field(s) are initialized multiple times: {}",
                                error.red(),
                                format_list(fields_initialized_twice).yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NoRecordFoundWithFields(fields, id) => {
                            eprintln!(
                                "{} no record found that has all the following field(s): {}",
                                error.red(),
                                format_list(fields).yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NotAClassName(name, id) => {
                            eprintln!("{} {} is not a class", error.red(), name.yellow(),);
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::InvalidArgumentInTypeClassConstraint(arg, id) => {
                            eprintln!(
                                "{} class constraint argument {} is unknown type argument",
                                error.red(),
                                arg.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NotAClassMember(member_name, class_name, id) => {
                            eprintln!(
                                "{} {} is not a member of class {}",
                                error.red(),
                                member_name.yellow(),
                                class_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::MissingClassMemberInInstance(
                            member_name,
                            class_name,
                            id,
                        ) => {
                            eprintln!(
                                "{} class member {} of class {} is missing in instance",
                                error.red(),
                                member_name.yellow(),
                                class_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ClassMemberTypeArgMissing(member_name, class_arg, id) => {
                            eprintln!(
                                "{} type arguments of class member {} does not contain the type argument of class: {}",
                                error.red(),
                                member_name.yellow(),
                                class_arg.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ExtraConstraintInClassMember(member_name, id) => {
                            eprintln!(
                                "{} extra type constraint in class member {}",
                                error.red(),
                                member_name.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ConflictingDefaultClassMember(class, name, locations) => {
                            eprintln!(
                                "{} conflicting default implementations for class member {} in class {}",
                                error.red(),
                                name.yellow(),
                                class.yellow()
                            );
                            for id in locations {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::DefaultClassMemberWithoutType(class, name, id) => {
                            eprintln!(
                                "{} class member {} in class {} has no type signature",
                                error.red(),
                                name.yellow(),
                                class.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ConflictingFunctionTypesInModule(
                            module,
                            name,
                            locations,
                        ) => {
                            eprintln!(
                                "{} conflicting function types named {} in module {}",
                                error.red(),
                                name.yellow(),
                                module.yellow()
                            );
                            for id in locations {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::InstanceMemberWithoutImplementation(name, id) => {
                            eprintln!(
                                "{} instance member {} has no implementation",
                                error.red(),
                                name.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ConflictingInstanceMemberFunction(name, locations) => {
                            eprintln!(
                                "{} conflicting instance member function named {}",
                                error.red(),
                                name.yellow(),
                            );
                            for id in locations {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::ConflictingFunctionTypesInInstance(name, locations) => {
                            eprintln!(
                                "{} conflicting function types named {} in instance",
                                error.red(),
                                name.yellow(),
                            );
                            for id in locations {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::FunctionTypeWithoutImplementationInModule(
                            module,
                            name,
                            id,
                        ) => {
                            eprintln!(
                                "{} function type {} has no implementation in module {}",
                                error.red(),
                                name.yellow(),
                                module.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::InvalidClassArgument(id) => {
                            eprintln!(
                                "{} invalid class argument, must be a single type argument",
                                error.red(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::InvalidTypeArgInInstanceConstraint(arg, id) => {
                            eprintln!(
                                "{} instance constraint argument {} is unknown type argument",
                                error.red(),
                                arg.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::NamedInstancedNotUnique(module, instance, id) => {
                            eprintln!(
                                "{} named instance {} is not unique in module {}",
                                error.red(),
                                instance.yellow(),
                                module.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::PatternBindConflict(name, ids) => {
                            eprintln!(
                                "{} multiple variable named {} found",
                                error.red(),
                                name.yellow(),
                            );
                            for id in ids {
                                let location_set = location_info.get_item_location(id);
                                print_location_set(file_manager, location_set);
                            }
                        }
                        ResolverError::PatternBindNotPresent(name, id) => {
                            eprintln!(
                                "{} variable {} not present in all patterns in or pattern",
                                error.red(),
                                name.yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::ContinueOutsideLoop(id) => {
                            eprintln!("{} continue outside of a loop", error.red());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        ResolverError::BreakOutsideLoop(id) => {
                            eprintln!("{} break outside of a loop", error.red());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                    }
                }
            }
            Error::RuntimeError(err, id) => {
                eprintln!("{} {}", error.red(), err);
                let location_set = location_info.get_item_location(id);
                print_location_set(file_manager, location_set);
            }
            Error::TypecheckError(errs) => {
                for err in &errs.errors {
                    match err {
                        TypecheckError::ConflictingInstances(name, id1, id2) => {
                            eprintln!(
                                "{} conflicting class instances for class {}",
                                error.red(),
                                name.yellow()
                            );
                            let location_set = location_info.get_item_location(id1);
                            print_location_set(file_manager, location_set);
                            let location_set = location_info.get_item_location(id2);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::DeriveFailureNoInstanceFound(type_name, class_name, id) => {
                            eprintln!(
                                "{} auto derive failure, no instance found for class {} for a member of {}",
                                error.red(),
                                class_name.yellow(),
                                type_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::DeriveFailureInstanceNotGeneric(
                            type_name,
                            class_name,
                            id,
                        ) => {
                            eprintln!(
                                "{} auto derive failure, instance not generic for class {} for a member of {}",
                                error.red(),
                                class_name.yellow(),
                                type_name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::UntypedExternFunction(name, id) => {
                            eprintln!(
                                "{} extern function {} does not have a type signature",
                                error.red(),
                                name.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::FunctionArgAndSignatureMismatch(
                            name,
                            arg_count,
                            signature_arg_count,
                            id,
                            is_member,
                        ) => {
                            if *is_member {
                                eprintln!(
                                "{} member function type signature of {} does not match its argument count",
                                error.red(),
                                name.yellow()
                            );
                            } else {
                                eprintln!(
                                "{} function type signature of {} does not match its argument count",
                                error.red(),
                                name.yellow()
                            );
                            }
                            eprintln!(
                                "Arguments:                      {}",
                                format!("{}", arg_count).yellow()
                            );
                            eprintln!(
                                "Arguments in type signature:    {}",
                                format!("{}", signature_arg_count).yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::MainNotFound => {
                            eprintln!(
                                "{} {} function in module {} not found",
                                error.red(),
                                "main".yellow(),
                                "Main".yellow()
                            );
                        }
                        TypecheckError::TypeMismatch(id, expected, found) => {
                            eprintln!("{} type mismatch in expression", error.red());
                            eprintln!("Expected: {}", expected.yellow());
                            eprintln!("Found:    {}", found.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::FunctionArgumentMismatch(id, args, func) => {
                            eprintln!("{} invalid argument(s)", error.red());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                            eprintln!("Argument(s):      {}", args.yellow());
                            eprintln!("Function type:    {}", func.yellow());
                        }
                        TypecheckError::InvalidVariantPattern(id, name, expected, found) => {
                            eprintln!(
                                "{} invalid {} variant pattern, argument count mismatch",
                                error.red(),
                                name.yellow()
                            );
                            eprintln!("Expected:      {}", format!("{}", expected).yellow());
                            eprintln!("Found:         {}", format!("{}", found).yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::InvalidRecordPattern(id, name, expected, found) => {
                            eprintln!(
                                "{} invalid {} record pattern, argument count mismatch",
                                error.red(),
                                name.yellow()
                            );
                            eprintln!("Expected:      {}", format!("{}", expected).yellow());
                            eprintln!("Found:         {}", format!("{}", found).yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::TypeAnnotationNeeded(id) => {
                            eprintln!("{} Type annotation needed", error.red());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::InvalidFormatString(id) => {
                            eprintln!("{} invalid format string", error.red());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::CyclicClassDependencies(id, path) => {
                            eprintln!(
                                "{} cyclic class dependencies: {}",
                                error.red(),
                                path.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::MissingInstance(class, id) => {
                            eprintln!("{} missing instance of {}", error.red(), class.yellow());
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::ClassNotAutoDerivable(class, id) => {
                            eprintln!(
                                "{} class {} is not auto derivable",
                                error.red(),
                                class.yellow()
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::IncorrectTypeForMain(ty, id) => {
                            eprintln!(
                                "{} {} in module {} has type {} instead of {}",
                                error.red(),
                                MAIN_FUNCTION.yellow(),
                                MAIN_MODULE_NAME.yellow(),
                                ty.yellow(),
                                "()".yellow(),
                            );
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::UnreachablePattern(id) => {
                            eprintln!("{} unreachable pattern", error.red(),);
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                        TypecheckError::NonExhaustivePattern(id) => {
                            eprintln!("{} non exhaustive pattern", error.red(),);
                            let location_set = location_info.get_item_location(id);
                            print_location_set(file_manager, location_set);
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Error {
        Error::ParseError(e)
    }
}

impl From<ResolverErrorContainer> for Error {
    fn from(e: ResolverErrorContainer) -> Error {
        Error::ResolverError(e)
    }
}

impl From<TypecheckErrorContainer> for Error {
    fn from(e: TypecheckErrorContainer) -> Error {
        Error::TypecheckError(e)
    }
}
