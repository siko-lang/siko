use crate::mir::*;
use std::collections::BTreeMap;

fn process_item(
    total_count: &mut i64,
    ty: &String,
    data_group: &Vec<String>,
    data_arg_counts: &mut BTreeMap<String, i64>,
) {
    *total_count += 1;
    if !data_group.contains(ty) {
        let count = data_arg_counts
            .get(ty)
            .expect("dependent type not found in data arg counts");
        *total_count += count;
    }
}

struct ArgAllocator {
    unallocated_args: Vec<i64>,
    group_args: Vec<i64>,
}

impl ArgAllocator {
    fn new(count: i64) -> ArgAllocator {
        let group_args: Vec<i64> = (0..count).collect();
        let unallocated_args = group_args.clone();
        ArgAllocator {
            unallocated_args: unallocated_args,
            group_args: group_args,
        }
    }

    fn allocate(&mut self, count: i64) -> Vec<i64> {
        let mut rest = self.unallocated_args.split_off(count as usize);
        std::mem::swap(&mut rest, &mut self.unallocated_args);
        return rest;
    }

    fn get_group_args(&self) -> Vec<i64> {
        self.group_args.clone()
    }

    fn assert_done(&self) {
        if !self.unallocated_args.is_empty() {
            println!(
                "ArgAllocator, not all args were allocated! {} from {}",
                self.unallocated_args.len(),
                self.group_args.len()
            );
        }
    }
}

fn allocate_item_args(
    ty: &mut ExtendedType,
    allocator: &mut ArgAllocator,
    data_group: &Vec<String>,
    data_arg_counts: &BTreeMap<String, i64>,
) {
    if data_group.contains(&ty.ty) {
        let mut args = allocator.allocate(1);
        args.extend(allocator.get_group_args());
        std::mem::swap(&mut ty.args, &mut args);
    } else {
        let arg_count = data_arg_counts.get(&ty.ty).unwrap();
        let mut args = allocator.allocate(arg_count + 1);
        std::mem::swap(&mut ty.args, &mut args);
    }
}

fn process_data_group(
    mir_program: &mut Program,
    data_group: &Vec<String>,
    data_arg_counts: &mut BTreeMap<String, i64>,
) {
    let mut total_count = 0;
    for data in data_group {
        match mir_program.data.get(data).unwrap() {
            Data::Adt(adt) => {
                for v in &adt.variants {
                    process_item(&mut total_count, &v.ty.ty, data_group, data_arg_counts);
                }
            }
            Data::Record(record) => {
                if let Some(externals) = &record.externals {
                    for e in externals {
                        process_item(&mut total_count, &e.ty.ty, data_group, data_arg_counts);
                    }
                }
                for f in &record.fields {
                    process_item(&mut total_count, &f.ty.ty, data_group, data_arg_counts);
                }
            }
        }
    }
    let mut allocator = ArgAllocator::new(total_count);
    for data in data_group {
        data_arg_counts.insert(data.clone(), total_count);
        match mir_program.data.get_mut(data).unwrap() {
            Data::Adt(adt) => {
                adt.args = allocator.get_group_args();
                for v in &mut adt.variants {
                    allocate_item_args(&mut v.ty, &mut allocator, data_group, &data_arg_counts);
                }
            }
            Data::Record(record) => {
                record.args = allocator.get_group_args();
                if let Some(externals) = &mut record.externals {
                    for e in externals {
                        allocate_item_args(&mut e.ty, &mut allocator, data_group, &data_arg_counts);
                    }
                }
                for f in &mut record.fields {
                    allocate_item_args(&mut f.ty, &mut allocator, data_group, &data_arg_counts);
                }
            }
        }
    }
    allocator.assert_done();
    //println!("Processed data group {:?}, {}", data_group, total_count);
}

pub fn init_data(
    mir_program: &mut Program,
    data_groups: &Vec<Vec<String>>,
) -> BTreeMap<String, i64> {
    let mut data_arg_counts = BTreeMap::new();
    for data_group in data_groups {
        process_data_group(mir_program, data_group, &mut data_arg_counts);
    }
    data_arg_counts
}
