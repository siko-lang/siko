#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Foo {
}

#[derive(Clone)]
struct Main_Other2 {
    f: Main_Foo,
}

#[derive(Clone)]
struct Main_Other {
    f: Main_Foo,
}

#[derive(Clone)]
struct Main_Object {
    o: Main_Other,
}

fn Main_foo(arg_0: Main_Object) -> () {
    let i_0_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 = ();
    i_0_2
}

fn Main_main() -> () {
    let i_0_0 = Main_Foo{};
    let i_0_1 = Main_Other{f: i_0_0};
    let tmp_2 = i_0_1;
    let i_0_3 = Main_Foo{};
    let i_0_4 = Main_Other2{f: i_0_3};
    let tmp_3 = i_0_4;
    let i_0_6 = &tmp_2;
    let i_0_7 = Main_Object{o: i_0_6};
    let tmp_4 = i_0_7;
    let i_0_9 = &tmp_4;
    let i_0_10 = Main_foo(i_0_9);
    let i_0_11 = tmp_4;
    let i_0_12 = Main_foo(i_0_11);
    let i_0_13 = tmp_2;
    let i_0_14 = ();
    i_0_14
}

fn main() {
    Main_main();
}


