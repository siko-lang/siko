#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Foo_0 {
}

#[derive(Clone)]
struct Main_Other_0 {
    f: Main_Foo_0,
}

#[derive(Clone)]
struct Main_Other2_0 {
    f: Main_Foo_0,
}

#[derive(Clone)]
struct Main_Object_0 {
    o: Main_Other_0,
}

fn Main_main() -> () {
    let i_0_0 : Main_Foo_0 = Main_Foo_0{};
    let i_0_1 : Main_Other_0 = Main_Other_0{f: i_0_0};
    let tmp_2 = i_0_1;
    let i_0_3 : Main_Foo_0 = Main_Foo_0{};
    let i_0_4 : Main_Other2_0 = Main_Other2_0{f: i_0_3};
    let tmp_3 = i_0_4;
    let i_0_6 : &Main_Other_0 = &tmp_2;
    let i_0_7 : Main_Object_0 = Main_Object_0{o: i_0_6};
    let tmp_4 = i_0_7;
    let i_0_9 : &Main_Object_0 = &tmp_4;
    let i_0_10 : () = Main_foo(i_0_9);
    let i_0_11 : Main_Object_0 = tmp_4;
    let i_0_12 : () = Main_foo(i_0_11);
    let i_0_13 : Main_Other_0 = tmp_2;
    let i_0_14 : Main_Foo_0 = Main_Foo_0{};
    let i_0_15 : Main_Other_0 = Main_Other_0{f: i_0_14};
    let tmp_5 = i_0_15;
    let i_0_17 : Main_Foo_0 = Main_Foo_0{};
    let i_0_18 : Main_Other2_0 = Main_Other2_0{f: i_0_17};
    let tmp_6 = i_0_18;
    let i_0_20 : &Main_Other_0 = &tmp_5;
    let i_0_21 : Main_Object_0 = Main_Object_0{o: i_0_20};
    let tmp_7 = i_0_21;
    let i_0_23 : &Main_Object_0 = &tmp_7;
    let i_0_24 : () = Main_foo(i_0_23);
    let i_0_25 : Main_Object_0 = tmp_7;
    let i_0_26 : () = Main_foo(i_0_25);
    let i_0_27 : Main_Other_0 = tmp_5;
    let i_0_28 : () = ();
    i_0_28
}

fn Main_foo(arg_0: Main_Object_0) -> () {
    let i_0_0 : Main_Object_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : () = ();
    i_0_2
}

fn main() {
    Main_main();
}


