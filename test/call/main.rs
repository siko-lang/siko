#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Object {
}

fn Main_foo(arg_0: Main_Object) -> () {
    let i_0_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 = ();
    i_0_2
}

fn Main_main() -> () {
    let i_0_0 = Main_Object{};
    let tmp_2 = i_0_0;
    let i_0_2 = &tmp_2;
    let i_0_3 = Main_foo(i_0_2);
    let i_0_4 = tmp_2;
    let i_0_5 = Main_foo(i_0_4);
    let i_0_6 = ();
    i_0_6
}

fn main() {
    Main_main();
}


