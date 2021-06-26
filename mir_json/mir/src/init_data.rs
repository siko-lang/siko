use crate::mir::*;
use std::collections::BTreeMap;

fn get_external<'a, 'b>(ty: &'b str, mir_program: &'a Program) -> Option<&'a Vec<String>> {
    if let Some(Data::Record(record)) = mir_program.data.get(ty) {
        return record.externals.as_ref();
    }
    None
}

fn process_item(
    total_count: &mut i64,
    item: &String,
    mir_program: &Program,
    data_group: &Vec<String>,
    data_arg_counts: &mut BTreeMap<String, i64>,
    item_arg: i64,
) {
    if !data_group.contains(item) {
        if let Some(externals) = get_external(item, mir_program) {
            for e in externals {
                if !data_group.contains(e) {
                    let count = data_arg_counts
                        .get(e)
                        .expect("dependent type not found in data arg counts");
                    *total_count += count + 1;
                }
            }
        } else {
            let count = data_arg_counts
                .get(item)
                .expect("dependent type not found in data arg counts");
            *total_count += count + item_arg;
        }
    }
}

fn process_data_group(
    mir_program: &Program,
    data_group: &Vec<String>,
    data_arg_counts: &mut BTreeMap<String, i64>,
) {
    let mut total_count = 0;
    for data in data_group {
        match mir_program.data.get(data).unwrap() {
            Data::Adt(adt) => {
                for v in &adt.variants {
                    process_item(
                        &mut total_count,
                        &v.ty,
                        mir_program,
                        data_group,
                        data_arg_counts,
                        0,
                    );
                }
            }
            Data::Record(record) => {
                total_count += record.fields.len() as i64;
                for f in &record.fields {
                    process_item(
                        &mut total_count,
                        &f.ty,
                        mir_program,
                        data_group,
                        data_arg_counts,
                        1,
                    );
                }
            }
        }
    }
    for data in data_group {
        data_arg_counts.insert(data.clone(), total_count);
    }
    //println!("Processed data group {:?}, {}", data_group, total_count);
}

pub fn init_data(mir_program: &Program, data_groups: &Vec<Vec<String>>) -> BTreeMap<String, i64> {
    let mut data_arg_counts = BTreeMap::new();
    for data_group in data_groups {
        process_data_group(&mir_program, data_group, &mut data_arg_counts);
    }
    data_arg_counts
}
