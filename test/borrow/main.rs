#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Object_0<> {
}

#[derive(Clone)]
struct Main_Name_0<> {
}

#[derive(Clone)]
struct Main_Address_0<'l0> {
    street: &'l0 Main_Name_0,
    city: Main_Name_0,
}

#[derive(Clone)]
struct Main_Person_0<'l0> {
    address: Main_Address_0<'l0>,
}

fn Main_main_0() -> () {
    let i_0_0 : () = Main_test1_0();
    let i_0_1 : () = Main_test2_0();
    let i_0_2 : () = ();
    i_0_2
}

fn Main_test1_0() -> () {
    let _block_1 = {
        let i_1_0 : Main_Object_0 = Main_Object_0{};
        let tmp_1 = i_1_0;
        let i_1_2 : Main_Object_0 = tmp_1.clone();
        let tmp_2 = i_1_2;
        let i_1_4 : Main_Object_0 = tmp_1;
        let i_1_5 : Main_Object_0 = tmp_2;
        i_1_5
    };
    let i_0_0 : Main_Object_0 = _block_1;
    let tmp_3 = i_0_0;
    let i_0_2 : () = ();
    i_0_2
}

fn Main_test2_0() -> () {
    let i_0_0 : Main_Name_0 = Main_Name_0{};
    let tmp_4 = i_0_0;
    let i_0_2 : Main_Name_0 = Main_Name_0{};
    let tmp_5 = i_0_2;
    let i_0_4 : &Main_Name_0 = &tmp_4;
    let i_0_5 : Main_Name_0 = tmp_5;
    let i_0_6 : Main_Address_0 = Main_Address_0{street: i_0_4, city: i_0_5};
    let tmp_6 = i_0_6;
    let i_0_8 : Main_Address_0 = tmp_6;
    let i_0_9 : Main_Person_0 = Main_Person_0{address: i_0_8};
    let tmp_7 = i_0_9;
    let i_0_11 : Main_Name_0 = tmp_4;
    let i_0_12 : () = ();
    i_0_12
}

fn main() {
    Main_main_0();
}


