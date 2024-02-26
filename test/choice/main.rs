#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Address_0<> {
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Address_0 = Main_Address_0{};
    let tmp_3 = i_0_0;
    let i_0_2 : Main_Address_0 = Main_Address_0{};
    let tmp_4 = i_0_2;
    let i_0_4 : &Main_Address_0 = &tmp_3;
    let i_0_5 : &Main_Address_0 = &tmp_4;
    let i_0_6 : &Main_Address_0 = Main_choice_0(i_0_4, i_0_5);
    let i_0_7 : Main_Address_0 = tmp_3;
    let i_0_8 : Main_Address_0 = tmp_4;
    let i_0_9 : () = ();
    i_0_9
}

fn Main_choice_0(arg_0: &'l0 Main_Address_0, arg_1: &'l1 Main_Address_0) -> &'l4 Main_Address_0 {
    let i_0_0 : &Main_Address_0 = &arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : &Main_Address_0 = &arg_1;
    let tmp_2 = i_0_2;
    let i_0_4 : bool = true;
    let i_0_5 : &Main_Address_0 = if i_0_4 {
        let i_1_0 : &Main_Address_0 = &tmp_1;
        i_1_0
    } else {
        let i_2_0 : &Main_Address_0 = &tmp_2;
        i_2_0
    };
    i_0_5
}

fn main() {
    Main_main_0();
}


