#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Bar_0<> {
}

#[derive(Clone)]
struct Main_Foo_0<> {
    inner: Main_Bar_0,
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Bar_0 = Main_Bar_0{};
    let i_0_1 : Main_Foo_0 = Main_Foo_0{inner: i_0_0};
    let tmp_3 = i_0_1;
    let i_0_3 : &Main_Foo_0 = &tmp_3;
    let i_0_4 : &Main_Bar_0 = Main_other_0(i_0_3);
    let tmp_4 = i_0_4;
    let i_0_6 : Main_Foo_0 = tmp_3;
    let i_0_7 : () = ();
    i_0_7
}

fn Main_other_0<'l0: 'l2, 'l2: >(arg_0: &'l0 Main_Foo_0) -> &'l2 Main_Bar_0 {
    let i_0_0 : &Main_Foo_0 = &arg_0;
    let tmp_2 = i_0_0;
    let i_0_2 : &Main_Foo_0 = &tmp_2;
    let i_0_3 : &Main_Bar_0 = Main_id_0(i_0_2);
    i_0_3
}

fn Main_id_0<'l0: 'l2, 'l2: >(arg_0: &'l0 Main_Foo_0) -> &'l2 Main_Bar_0 {
    let i_0_0 : &Main_Foo_0 = &arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : &Main_Bar_0 = &tmp_1.inner;
    i_0_2
}

fn main() {
    Main_main_0();
}


