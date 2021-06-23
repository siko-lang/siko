use crate::mir::*;
use std::collections::BTreeMap;

fn process_data_group(
    mir_program: &Program,
    data_group: &Vec<String>,
    data_arg_counts: &mut BTreeMap<String, i64>,
) {
    let mut total_count = 0;
    for data in data_group {
        match mir_program.adts.get(data) {
            Some(adt) => {
                for v in &adt.variants {
                    if !data_group.contains(&v.ty) {
                        if let Some(member) = mir_program.records.get(&v.ty) {
                            if let Some(externals) = &member.externals {
                                for e in externals {
                                    if !data_group.contains(e) {
                                    let count = data_arg_counts
                                        .get(e)
                                        .expect("dependent type not found in data arg counts");
                                    total_count += count;
                                    }
                                }
                                continue;
                            }
                        }
                        let count = data_arg_counts
                            .get(&v.ty)
                            .expect("dependent type not found in data arg counts");
                        total_count += count;
                    }
                }
            }
            None => match mir_program.records.get(data) {
                Some(record) => {
                    total_count += record.fields.len() as i64;
                    for f in &record.fields {
                        if !data_group.contains(&f.ty) {
                            if let Some(member) = mir_program.records.get(&f.ty) {
                                if let Some(externals) = &member.externals {
                                    for e in externals {
                                        if !data_group.contains(e) {
                                        let count = data_arg_counts
                                            .get(e)
                                            .expect("dependent type not found in data arg counts");
                                        total_count += count;
                                        }
                                    }
                                    continue;
                                }
                            }
                            let count = data_arg_counts
                                .get(&f.ty)
                                .expect("dependent type not found in data arg counts");
                            total_count += count;
                        }
                    }
                }
                None => {}
            },
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
    let max = data_arg_counts.iter().map(|(_, v)| v).max();
    data_arg_counts
}
