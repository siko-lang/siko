#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Foo_0<> {
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Foo_0 = Main_Foo_0{};
    let tmp_4 = i_0_0;
    let i_0_2 : &Main_Foo_0 = &tmp_4;
    let i_0_3 : &Main_Foo_0 = Main_one_0(i_0_2);
    let i_0_4 : Main_Foo_0 = tmp_4;
    let i_0_5 : Main_Foo_0 = Main_foo_0(i_0_4);
    let i_0_6 : () = ();
    i_0_6
}

fn Main_one_0<'l0: 'l5, 'l5: >(arg_0: &'l0 Main_Foo_0) -> &'l5 Main_Foo_0 {
    let i_0_0 : &Main_Foo_0 = &arg_0;
    let tmp_2 = i_0_0;
    let i_0_2 : bool = true;
    let i_0_3 : &Main_Foo_0 = if i_0_2 {
        let i_1_0 : &Main_Foo_0 = &tmp_2;
        let i_1_1 : &Main_Foo_0 = Main_two_0(i_1_0);
        i_1_1
    } else {
        let i_2_0 : &Main_Foo_0 = &tmp_2;
        let i_2_1 : &Main_Foo_0 = Main_two_0(i_2_0);
        i_2_1
    };
    i_0_3
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

fn Main_two_0<'l0: 'l4, 'l4: >(arg_0: &'l0 Main_Foo_0) -> &'l4 Main_Foo_0 {
    let i_0_0 : &Main_Foo_0 = &arg_0;
    let tmp_3 = i_0_0;
    let i_0_2 : bool = false;
    let i_0_3 : &Main_Foo_0 = if i_0_2 {
        let i_1_0 : &Main_Foo_0 = &tmp_3;
        i_1_0
    } else {
        let i_2_0 : &Main_Foo_0 = &tmp_3;
        let i_2_1 : &Main_Foo_0 = Main_two_0(i_2_0);
        i_2_1
    };
    i_0_3
}

fn main() {
    Main_main_0();
}


