use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::mir::*;

#[derive(Clone)]
pub struct Converter {
    from: TypeVariable,
    to: TypeVariable,
}

impl std::fmt::Debug for Converter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?} -> {:?})", self.from, self.to)
    }
}

#[derive(Clone)]
pub struct FunctionInfo {
    args: Vec<TypeVariableInfo>,
    result: TypeVariableInfo,
    members: Vec<MemberInfo>,
    variants: Vec<VariantInfo>,
    converters: Vec<Converter>,
}

impl FunctionInfo {
    fn new(
        args: Vec<TypeVariableInfo>,
        result: TypeVariableInfo,
        members: Vec<MemberInfo>,
        variants: Vec<VariantInfo>,
        converters: Vec<Converter>,
    ) -> FunctionInfo {
        FunctionInfo {
            args: args,
            result: result,
            members: members,
            variants: variants,
            converters: converters,
        }
    }
}

pub struct FunctionInfoStore {
    functions: BTreeMap<String, FunctionInfo>,
}

impl FunctionInfoStore {
    fn new() -> FunctionInfoStore {
        FunctionInfoStore {
            functions: BTreeMap::new(),
        }
    }

    fn add_function(&mut self, name: String, function_info: FunctionInfo) {
        self.functions.insert(name, function_info);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeVariable {
    pub v: i64,
}

impl std::fmt::Debug for TypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.v)
    }
}

impl TypeVariable {
    pub fn new(v: i64) -> TypeVariable {
        TypeVariable { v: v }
    }

    pub fn apply_constraint(&mut self, c: &Constraint) {
        match c {
            Constraint::Equal(c1, c2) => {
                let min_c = std::cmp::min(c1, c2);
                let max_c = std::cmp::max(c1, c2);
                if self == min_c {
                    *self = *max_c;
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberInfo {
    name: String,
    root: TypeVariable,
    info: TypeVariableInfo,
}

impl MemberInfo {
    fn apply_constraint(&mut self, c: &Constraint) {
        self.root.apply_constraint(c);
        self.info.apply_constraint(c);
    }
}

impl std::fmt::Debug for MemberInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {:?} => {:?})", self.name, self.root, self.info)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariantInfo {
    index: i64,
    root: TypeVariable,
    info: TypeVariableInfo,
}

impl VariantInfo {
    fn apply_constraint(&mut self, c: &Constraint) {
        self.root.apply_constraint(c);
        self.info.apply_constraint(c);
    }
}

impl std::fmt::Debug for VariantInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {:?} => {:?})", self.index, self.root, self.info)
    }
}

pub struct InferenceInfo {
    expr_type_variables: BTreeMap<i64, TypeVariableInfo>,
    var_type_variables: BTreeMap<String, TypeVariableInfo>,
    args: Vec<TypeVariableInfo>,
    result_info: TypeVariableInfo,
    members: Vec<MemberInfo>,
    variants: Vec<VariantInfo>,
    var_allocator: TypeVariableAllocator,
    converters: Vec<Converter>,
}

impl InferenceInfo {
    pub fn new(arg_count: usize) -> InferenceInfo {
        let mut allocator = TypeVariableAllocator::new();
        let result_info = allocator.allocate_info();

        let mut inference_info = InferenceInfo {
            expr_type_variables: BTreeMap::new(),
            var_type_variables: BTreeMap::new(),
            args: Vec::new(),
            result_info: result_info,
            members: Vec::new(),
            variants: Vec::new(),
            var_allocator: allocator,
            converters: Vec::new(),
        };

        for index in 0..arg_count {
            let v = format!("arg{}", index);
            inference_info.add_var(v.clone());
            inference_info.args.push(inference_info.var_info(&v));
        }
        inference_info
    }

    fn to_function_info(&self) -> FunctionInfo {
        let mut input_vars = BTreeSet::new();
        for arg in &self.args {
            input_vars.insert(arg.ownership_var);
            input_vars.insert(arg.arg_group_var);
        }
        loop {
            let mut added = false;
            for member in &self.members {
                if input_vars.contains(&member.root) {
                    if input_vars.insert(member.info.ownership_var) {
                        added = true;
                    }
                    if input_vars.insert(member.info.arg_group_var) {
                        added = true;
                    }
                }
            }
            for variant in &self.variants {
                if input_vars.contains(&variant.root) {
                    if input_vars.insert(variant.info.ownership_var) {
                        added = true;
                    }
                    if input_vars.insert(variant.info.arg_group_var) {
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        let mut output_vars = BTreeSet::new();
        output_vars.insert(self.result_info.ownership_var);
        output_vars.insert(self.result_info.arg_group_var);
        loop {
            let mut added = false;
            for member in &self.members {
                if output_vars.contains(&member.root) {
                    if output_vars.insert(member.info.ownership_var) {
                        added = true;
                    }
                    if output_vars.insert(member.info.arg_group_var) {
                        added = true;
                    }
                }
            }
            for variant in &self.variants {
                if output_vars.contains(&variant.root) {
                    if output_vars.insert(variant.info.ownership_var) {
                        added = true;
                    }
                    if output_vars.insert(variant.info.arg_group_var) {
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        //println!("All inputs {:?}", input_vars);
        //println!("All outputs: {:?}", output_vars);
        let same_vars: Vec<_> = input_vars.intersection(&output_vars).cloned().collect();
        let members = self
            .members
            .iter()
            .filter(|member| {
                !same_vars.contains(&member.info.ownership_var)
                    || !same_vars.contains(&member.info.arg_group_var)
            })
            .cloned()
            .collect();
        let variants = self
            .variants
            .iter()
            .filter(|variant| {
                !same_vars.contains(&variant.info.ownership_var)
                    || !same_vars.contains(&variant.info.arg_group_var)
            })
            .cloned()
            .collect();
        let mut converter_map = BTreeMap::new();
        for c in &self.converters {
            let tos = converter_map.entry(c.from).or_insert_with(|| Vec::new());
            tos.push(c.to);
        }
        let mut converters = Vec::new();
        for input in &input_vars {
            let mut conversions = BTreeSet::new();
            conversions.insert(input);
            let mut new_ones = vec![input];
            loop {
                let mut currents = Vec::new();
                std::mem::swap(&mut new_ones, &mut currents);
                for new_one in currents {
                    match converter_map.get(new_one) {
                        Some(tos) => {
                            for to in tos {
                                if conversions.insert(to) {
                                    new_ones.push(to);
                                }
                            }
                        }
                        None => {}
                    }
                }
                if new_ones.is_empty() {
                    break;
                }
            }
            for output in &output_vars {
                if conversions.contains(output) {
                    //println!("{:?} converts into {:?}", input, output);
                    converters.push(Converter {
                        from:* input,
                        to: *output,
                    });
                }
            }
        }
        FunctionInfo::new(
            self.args.clone(),
            self.result_info.clone(),
            members,
            variants,
            converters,
        )
    }

    fn create_record_ctor_function_info(
        &mut self,
        name: &String,
        mir_program: &Program,
    ) -> FunctionInfo {
        let data = mir_program.data.get(name).unwrap();
        match data {
            Data::Record(record) => {
                let mut args = Vec::new();
                let mut members = Vec::new();
                let result = self.var_allocator.allocate_info();
                for field in &record.fields {
                    let arg_info = self.var_allocator.allocate_info();
                    let member = MemberInfo {
                        name: field.name.clone(),
                        root: result.arg_group_var,
                        info: arg_info,
                    };
                    members.push(member);
                    args.push(arg_info);
                }
                FunctionInfo::new(args, result, members, Vec::new(), Vec::new())
            }
            _ => panic!("Not a record!"),
        }
    }

    fn create_variant_ctor_function_info(
        &mut self,
        name: &String,
        index: i64,
        mir_program: &Program,
    ) -> FunctionInfo {
        let data = mir_program.data.get(name).unwrap();
        match data {
            Data::Adt(adt) => {
                let variant = &adt.variants[index as usize];
                let record = mir_program.data.get(&variant.ty.ty).unwrap();
                match record {
                    Data::Record(record) => {
                        let mut args = Vec::new();
                        let mut members = Vec::new();
                        let adt_result = self.var_allocator.allocate_info();
                        let record_result = self.var_allocator.allocate_info();
                        for field in &record.fields {
                            let arg_info = self.var_allocator.allocate_info();
                            let member = MemberInfo {
                                name: field.name.clone(),
                                root: record_result.arg_group_var,
                                info: arg_info,
                            };
                            members.push(member);
                            args.push(arg_info);
                        }
                        let variants = vec![VariantInfo {
                            index: index,
                            root: adt_result.arg_group_var,
                            info: record_result,
                        }];
                        FunctionInfo::new(args, adt_result, members, variants, Vec::new())
                    }
                    _ => panic!("Not a record!"),
                }
            }
            _ => panic!("Not an adt!"),
        }
    }

    fn create_dummy_function_info(&mut self, arg_count: usize) -> FunctionInfo {
        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.var_allocator.allocate_info());
        }
        FunctionInfo::new(
            args,
            self.var_allocator.allocate_info(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn add_member(&mut self, name: String, root: TypeVariable, info: TypeVariableInfo) {
        self.members.push(MemberInfo { name, root, info });
    }

    fn add_variant(&mut self, index: i64, root: TypeVariable, info: TypeVariableInfo) {
        self.variants.push(VariantInfo { index, root, info });
    }

    fn add_expr(&mut self, id: i64) {
        let info = self.var_allocator.allocate_info();
        self.expr_type_variables.insert(id, info);
    }

    fn add_var(&mut self, id: String) {
        let info = self.var_allocator.allocate_info();
        self.var_type_variables.insert(id, info);
    }

    fn expr_info(&self, id: i64) -> TypeVariableInfo {
        self.expr_type_variables.get(&id).unwrap().clone()
    }

    fn var_info(&self, id: &String) -> TypeVariableInfo {
        self.var_type_variables.get(id).unwrap().clone()
    }

    fn apply_constraint(&mut self, constraint: &Constraint) {
        //println!("C: {:?}", constraint);
        for info in self.expr_type_variables.values_mut() {
            info.apply_constraint(constraint);
        }
        for info in self.var_type_variables.values_mut() {
            info.apply_constraint(constraint);
        }
        for info in &mut self.args {
            info.apply_constraint(constraint);
        }
        for info in &mut self.members {
            info.apply_constraint(constraint);
        }
        for info in &mut self.variants {
            info.apply_constraint(constraint);
        }
        self.result_info.apply_constraint(constraint);
        //println!("result {:?}", self.result_info);
    }

    fn merge_members_and_variants(&mut self) {
        // println!(
        //     "merge_members_and_variants started {} {} {} {} {}",
        //     self.expr_type_variables.len(),
        //     self.var_type_variables.len(),
        //     self.members.len(),
        //     self.variants.len(),
        //     self.converters.len()
        // );
        loop {
            let mut root_map = BTreeMap::new();
            let mut constraints = ConstraintStore::new();
            for member in &self.members {
                let infos = root_map
                    .entry((&member.name, member.root))
                    .or_insert_with(|| Vec::new());
                infos.push(member.info);
            }
            for (_, infos) in root_map {
                if infos.len() > 1 {
                    let first = infos[0];
                    for info in infos.iter().skip(1) {
                        constraints.add_equal_info(first, *info);
                    }
                }
            }
            if constraints.constraints.is_empty() {
                break;
            } else {
                //println!("Applying {:?}", constraints.constraints);
                self.apply_all(&mut constraints);
                self.members.sort();
                self.members.dedup();
            }
        }
        loop {
            let mut root_map = BTreeMap::new();
            let mut constraints = ConstraintStore::new();
            for variant in &self.variants {
                let infos = root_map
                    .entry((variant.index, variant.root))
                    .or_insert_with(|| Vec::new());
                infos.push(variant.info);
            }
            for (_, infos) in root_map {
                if infos.len() > 1 {
                    let first = infos[0];
                    for info in infos.iter().skip(1) {
                        constraints.add_equal_info(first, *info);
                    }
                }
            }
            if constraints.constraints.is_empty() {
                break;
            } else {
                //println!("Applying {:?}", constraints.constraints);
                self.apply_all(&mut constraints);
                self.variants.sort();
                self.variants.dedup();
            }
        }
        //println!("merge_members_and_variants ended");
    }

    fn apply_all(&mut self, constraints: &mut ConstraintStore) {
        let c_count = constraints.constraints.len();
        for index in 0..c_count {
            let c = constraints.constraints[index].clone();
            self.apply_constraint(&c);
            constraints.apply(c);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeVariableInfo {
    pub ownership_var: TypeVariable,
    pub arg_group_var: TypeVariable,
}

impl TypeVariableInfo {
    fn new(ov: TypeVariable, gv: TypeVariable) -> TypeVariableInfo {
        TypeVariableInfo {
            ownership_var: ov,
            arg_group_var: gv,
        }
    }

    fn apply_constraint(&mut self, constraint: &Constraint) {
        self.ownership_var.apply_constraint(constraint);
        self.arg_group_var.apply_constraint(constraint);
    }
}

impl std::fmt::Debug for TypeVariableInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.ownership_var, self.arg_group_var)
    }
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Equal(TypeVariable, TypeVariable),
}

struct TypeVariableAllocator {
    next: i64,
}

impl TypeVariableAllocator {
    pub fn new() -> TypeVariableAllocator {
        TypeVariableAllocator { next: 1 }
    }

    pub fn allocate(&mut self) -> TypeVariable {
        let v = self.next;
        self.next += 1;
        TypeVariable { v: v }
    }

    pub fn allocate_info(&mut self) -> TypeVariableInfo {
        TypeVariableInfo::new(self.allocate(), self.allocate())
    }
}

pub struct ConstraintStore {
    constraints: Vec<Constraint>,
}

impl ConstraintStore {
    pub fn new() -> ConstraintStore {
        ConstraintStore {
            constraints: Vec::new(),
        }
    }

    pub fn add_equal_exprs(&mut self, id1: i64, id2: i64, inference_info: &InferenceInfo) {
        let info1 = inference_info.expr_info(id1);
        let info2 = inference_info.expr_info(id2);
        self.add_equal_info(info1, info2);
    }

    pub fn add_equal_expr_var(&mut self, id: i64, var: &String, inference_info: &InferenceInfo) {
        let info1 = inference_info.expr_info(id);
        let info2 = inference_info.var_info(var);
        self.add_equal_info(info1, info2);
    }

    pub fn add_equal_info(&mut self, info1: TypeVariableInfo, info2: TypeVariableInfo) {
        self.constraints
            .push(Constraint::Equal(info1.ownership_var, info2.ownership_var));
        self.constraints
            .push(Constraint::Equal(info1.arg_group_var, info2.arg_group_var));
    }

    pub fn apply(&mut self, applied: Constraint) {
        match applied {
            Constraint::Equal(from, to) => {
                for c in &mut self.constraints {
                    match c {
                        Constraint::Equal(id1, id2) => {
                            if *id1 == from {
                                *id1 = to;
                            }
                            if *id2 == from {
                                *id2 = to;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

struct LoopCollector {
    loops: Vec<i64>,
    breaks: BTreeMap<i64, Vec<i64>>,
    continues: BTreeMap<i64, Vec<i64>>,
}

impl LoopCollector {
    fn new() -> LoopCollector {
        LoopCollector {
            loops: Vec::new(),
            breaks: BTreeMap::new(),
            continues: BTreeMap::new(),
        }
    }
}

impl Visitor for LoopCollector {
    fn visit(&mut self, expr: &Expr) {
        match expr.kind {
            ExprKind::Loop(_, _, _) => {
                self.loops.push(expr.id);
            }
            ExprKind::Break(_) => {
                let loop_id = self.loops.last().unwrap();
                let breaks = self.breaks.entry(*loop_id).or_insert_with(|| Vec::new());
                breaks.push(expr.id);
            }
            ExprKind::Continue(_) => {
                let loop_id = self.loops.last().unwrap();
                let continues = self.continues.entry(*loop_id).or_insert_with(|| Vec::new());
                continues.push(expr.id);
            }
            _ => {}
        }
    }
}

struct Inferer {
    inference_info: InferenceInfo,
}

impl Inferer {
    fn new(inference_info: InferenceInfo) -> Inferer {
        Inferer {
            inference_info: inference_info,
        }
    }
}

impl Visitor for Inferer {
    fn visit_after(&mut self, expr: &Expr) {
        let info = self.inference_info.expr_info(expr.id);
        //println!("Inferring {} {:?} {:?}", expr.id, expr.kind, info);
    }
}

fn process_function(
    f: &String,
    mir_program: &Program,
    function_info_store: &mut FunctionInfoStore,
) {
    // initialization
    let f = mir_program.functions.get(f).unwrap();
    let mut inference_info = InferenceInfo::new(f.args.len());
    //println!("Processing {}", f.name);
    let mut loop_collector = LoopCollector::new();
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            walk(exprs, &0, &mut loop_collector);
            for e in exprs.iter() {
                inference_info.add_expr(e.id);
                match &e.kind {
                    ExprKind::VarDecl(name, _) => {
                        inference_info.add_var(name.clone());
                    }
                    ExprKind::Loop(name, _, _) => {
                        inference_info.add_var(name.clone());
                    }
                    ExprKind::CaseOf(_, cases) => {
                        for case in cases {
                            match &case.checker {
                                Checker::Variant(_, name, _) => {
                                    inference_info.add_var(name.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    // constraint generation
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            let mut constraints = ConstraintStore::new();
            for e in exprs.iter() {
                match &e.kind {
                    ExprKind::Do(items) => {
                        assert!(!items.is_empty());
                        let last = items.last().unwrap();
                        constraints.add_equal_exprs(*last, e.id, &inference_info);
                    }
                    ExprKind::StaticFunctionCall(name, args) => {
                        let function_info = match function_info_store.functions.get(name) {
                            Some(function_info) => function_info.clone(),
                            None => {
                                let called_f = mir_program.functions.get(name).unwrap();
                                match &called_f.kind {
                                    FunctionKind::Normal(_) => {
                                        inference_info.create_dummy_function_info(args.len())
                                    }
                                    FunctionKind::RecordCtor => inference_info
                                        .create_record_ctor_function_info(
                                            &called_f.result.ty,
                                            mir_program,
                                        ),
                                    FunctionKind::VariantCtor(index) => inference_info
                                        .create_variant_ctor_function_info(
                                            &called_f.result.ty,
                                            *index,
                                            mir_program,
                                        ),
                                    FunctionKind::External => {
                                        //panic!("No external function info found! {}", name);
                                        inference_info.create_dummy_function_info(args.len())
                                    }
                                }
                            }
                        };
                        inference_info.members.extend(function_info.members);
                        inference_info.variants.extend(function_info.variants);
                        inference_info.converters.extend(function_info.converters);
                        for index in 0..args.len() {
                            let arg_expr = &args[index];
                            let arg_expr_info = inference_info.expr_info(*arg_expr);
                            let arg_info = function_info.args[index];
                            constraints.add_equal_info(arg_expr_info, arg_info);
                        }
                        let info = inference_info.expr_info(e.id);
                        //println!("F {:?} {:?}", function_info.result, info);
                        constraints.add_equal_info(function_info.result, info);
                    }
                    ExprKind::VarDecl(name, rhs) => {
                        constraints.add_equal_expr_var(*rhs, name, &inference_info);
                    }
                    ExprKind::VarRef(name) => {
                        constraints.add_equal_expr_var(e.id, name, &inference_info);
                    }
                    ExprKind::If(_, true_branch, false_branch) => {
                        constraints.add_equal_exprs(*true_branch, e.id, &inference_info);
                        constraints.add_equal_exprs(*true_branch, *false_branch, &inference_info);
                    }
                    ExprKind::List(items) => match items.first() {
                        Some(first) => {
                            for (index, item) in items.iter().enumerate() {
                                if index != 0 {
                                    constraints.add_equal_exprs(*first, *item, &inference_info);
                                }
                            }
                        }
                        None => {}
                    },
                    ExprKind::FieldAccess(name, receiver) => {
                        let receiver_info = inference_info.expr_info(*receiver);
                        let info = inference_info.expr_info(e.id);
                        inference_info.add_member(name.clone(), receiver_info.arg_group_var, info);
                    }
                    ExprKind::Return(arg) => {
                        let result_info = inference_info.result_info;
                        let arg_info = inference_info.expr_info(*arg);
                        constraints.add_equal_info(result_info, arg_info);
                    }
                    ExprKind::CaseOf(body, cases) => {
                        let first = cases.first().unwrap().body;
                        for (index, case) in cases.iter().enumerate() {
                            match &case.checker {
                                Checker::Variant(index, var, ty) => {
                                    match mir_program.data.get(ty).unwrap() {
                                        Data::Adt(adt) => {
                                            let expr_info = inference_info.expr_info(*body);
                                            let var_info = inference_info.var_info(&var);
                                            inference_info.add_variant(
                                                *index,
                                                expr_info.arg_group_var,
                                                var_info,
                                            );
                                        }
                                        Data::Record(_) => {
                                            let expr_info = inference_info.expr_info(*body);
                                            let var_info = inference_info.var_info(&var);
                                            constraints.add_equal_info(expr_info, var_info);
                                        }
                                    }
                                }
                                _ => {}
                            }
                            if index != 0 {
                                constraints.add_equal_exprs(first, case.body, &inference_info);
                            }
                        }
                    }
                    ExprKind::Loop(var, initializer, body) => {
                        constraints.add_equal_expr_var(*initializer, var, &inference_info);
                        constraints.add_equal_expr_var(*body, var, &inference_info);
                        if let Some(continues) = loop_collector.continues.get(&e.id) {
                            for c in continues {
                                constraints.add_equal_exprs(*c, *body, &inference_info);
                            }
                        }
                        if let Some(breaks) = loop_collector.breaks.get(&e.id) {
                            for b in breaks {
                                constraints.add_equal_exprs(*b, e.id, &inference_info);
                            }
                        }
                    }
                    ExprKind::Converter(arg) => {
                        let info1 = inference_info.expr_info(e.id);
                        let info2 = inference_info.expr_info(*arg);
                        constraints
                            .constraints
                            .push(Constraint::Equal(info1.arg_group_var, info2.arg_group_var));
                        inference_info.converters.push(Converter {
                            from: info1.ownership_var,
                            to: info2.ownership_var,
                        });
                    }
                    _ => {}
                }
            }
            let body_info = inference_info.expr_info(0);
            constraints.add_equal_info(inference_info.result_info, body_info);
            //println!("{} constraints", constraints.constraints.len());
            inference_info.apply_all(&mut constraints);
            inference_info.merge_members_and_variants();
            //println!("members: {:?}", inference_info.members);
            //println!("variants: {:?}", inference_info.variants);
        }
        _ => {}
    }

    // ownership inference
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            //println!("Inferring {}", f.name);
            let mut inferer = Inferer::new(inference_info);
            walk(exprs, &0, &mut inferer);
            /*
            println!("Args {:?}", inferer.inference_info.args);
            println!("Result {:?}", inferer.inference_info.result_info);
            println!("members: {:?}", inferer.inference_info.members);
            println!("variants: {:?}", inferer.inference_info.variants);
            println!("Converters: {:?}", inferer.inference_info.converters);
            */
            function_info_store
                .add_function(f.name.clone(), inferer.inference_info.to_function_info());
            /*
                if f.name == "Bool.opEq_0" {
                panic!("Done with {}!", f.name);
            }*/
        }
        _ => {}
    }
}

fn process_function_group(
    group: &Vec<String>,
    mir_program: &Program,
    function_info_store: &mut FunctionInfoStore,
) {
    //println!("Processing f group {:?}", group);
    for f in group {
        process_function(&f, mir_program, function_info_store);
    }
}

pub fn inference(function_groups: Vec<Vec<String>>, mir_program: &Program) {
    println!("Inference started");
    let mut function_info_store = FunctionInfoStore::new();
    {
        let name = "Int.eqInt_0".to_string();
        let info = TypeVariableInfo::new(TypeVariable::new(30), TypeVariable::new(31));
        let args = vec![info, info];
        let result = info;
        let function_info = FunctionInfo::new(args, result, Vec::new(), Vec::new(), Vec::new());
        function_info_store.add_function(name, function_info);
    }
    {
        let name = "Std.Ops.getDiscriminant_12".to_string();
        let info1 = TypeVariableInfo::new(TypeVariable::new(1030), TypeVariable::new(1031));
        let info2 = TypeVariableInfo::new(TypeVariable::new(1032), TypeVariable::new(1033));
        let args = vec![info1];
        let result = info2;
        let function_info = FunctionInfo::new(args, result, Vec::new(), Vec::new(), Vec::new());
        function_info_store.add_function(name, function_info);
    }
    for group in &function_groups {
        process_function_group(group, mir_program, &mut function_info_store);
    }
}
