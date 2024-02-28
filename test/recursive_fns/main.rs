#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Foo_0<> {
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Foo_0 = Main_Foo_0{};
    let tmp_2 = i_0_0;
    let i_0_2 : Main_Foo_0 = tmp_2;
    let i_0_3 : Main_Foo_0 = Main_foo_0(i_0_2);
    let i_0_4 : () = ();
    i_0_4
}

fn Main_foo_0(arg_0: Main_Foo_0) -> Main_Foo_0 {
    let i_0_0 : Main_Foo_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : bool = true;
    let i_0_3 : Main_Foo_0 = if i_0_2 {
        let i_1_0 : Main_Foo_0 = tmp_1;
        let i_1_1 : Main_Foo_0 = Main_foo_0(i_1_0);
        i_1_1
    } else {
        let i_2_0 : Main_Foo_0 = Main_Foo_0{};
        i_2_0
    };
    i_0_3
}

fn main() {
    Main_main_0();
}


