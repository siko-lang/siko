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

fn Main_main_0() -> () {
    let i_0_0 : Main_Name_0 = Main_Name_0{};
    let i_0_1 : Main_Name_0 = Main_Name_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let i_0_3 : Main_Person_0 = Main_Person_0{address: i_0_2};
    let i_0_4 : Main_Person_0 = Main_process_0(i_0_3);
    let i_0_5 : () = ();
    i_0_5
}

fn Main_process_0(arg_0: Main_Person_0) -> Main_Person_0 {
    let i_0_0 : Main_Person_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : Main_Name_0 = Main_Name_0{};
    let i_0_3 : Main_Name_0 = tmp_1.address.street;
    let i_0_4 : Main_Address_0 = Main_Address_0{city: i_0_2, street: i_0_3};
    let i_0_5 : Main_Person_0 = Main_Person_0{address: i_0_4};
    i_0_5
}

fn main() {
    Main_main_0();
}


