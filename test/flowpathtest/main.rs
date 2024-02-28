#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Name_0<> {
}

#[derive(Clone)]
struct Main_Address_0<> {
    city: Main_Name_0,
    street: Main_Name_0,
}

#[derive(Clone)]
struct Main_Person_0<> {
    address: Main_Address_0,
}

#[derive(Clone)]
struct Main_Person_1<'l0> {
    address: Main_Address_1<'l0>,
}

#[derive(Clone)]
struct Main_Address_1<'l0> {
    city: Main_Name_0,
    street: &'l0 Main_Name_0,
}

fn Main_main_0() -> () {
    let i_0_0 : Main_Name_0 = Main_Name_0{};
    let tmp_2 = i_0_0;
    let i_0_2 : Main_Name_0 = Main_Name_0{};
    let tmp_3 = i_0_2;
    let i_0_4 : Main_Name_0 = tmp_2;
    let i_0_5 : Main_Name_0 = tmp_3;
    let i_0_6 : Main_Address_0 = Main_Address_0{city: i_0_4, street: i_0_5};
    let tmp_4 = i_0_6;
    let i_0_8 : Main_Address_0 = tmp_4;
    let i_0_9 : Main_Person_0 = Main_Person_0{address: i_0_8};
    let tmp_5 = i_0_9;
    let i_0_11 : &Main_Person_0 = &tmp_5;
    let i_0_12 : Main_Person_1 = Main_process_0(i_0_11);
    let tmp_6 = i_0_12;
    let i_0_14 : Main_Person_0 = tmp_5;
    let i_0_15 : () = ();
    i_0_15
}

fn Main_process_0<'l0: >(arg_0: &'l0 Main_Person_0) -> Main_Person_0 {
    let i_0_0 : &Main_Person_0 = &arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : Main_Name_0 = Main_Name_0{};
    let i_0_3 : Main_Name_0 = tmp_1.address.street.clone();
    let i_0_4 : Main_Address_0 = Main_Address_0{city: i_0_2, street: i_0_3};
    let i_0_5 : Main_Person_0 = Main_Person_0{address: i_0_4};
    i_0_5
}

fn main() {
    Main_main_0();
}


