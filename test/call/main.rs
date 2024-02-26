#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Foo_0<> {
}

#[derive(Clone)]
struct Main_Other_0<> {
    f: Main_Foo_0,
}

#[derive(Clone)]
struct Main_Object_0<> {
    o: Main_Other_0,
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Foo_0 = Main_Foo_0{};
    let i_0_1 : Main_Other_0 = Main_Other_0{f: i_0_0};
    let tmp_2 = i_0_1;
    let i_0_3 : Main_Other_0 = tmp_2;
    let i_0_4 : Main_Object_0 = Main_Object_0{o: i_0_3};
    let tmp_3 = i_0_4;
    let i_0_6 : &Main_Object_0 = &tmp_3;
    let i_0_7 : () = Main_foo_0(i_0_6);
    let i_0_8 : Main_Object_0 = tmp_3;
    let i_0_9 : () = Main_foo_1(i_0_8);
    let i_0_10 : () = ();
    i_0_10
}

fn Main_foo_0<'l0>(arg_0: &'l0 Main_Object_0) -> () {
    let i_0_0 : &Main_Object_0 = &arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : () = ();
    i_0_2
}

fn Main_foo_1(arg_0: Main_Object_0) -> () {
    let i_0_0 : Main_Object_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : () = ();
    i_0_2
}

fn main() {
    Main_main_0();
}


