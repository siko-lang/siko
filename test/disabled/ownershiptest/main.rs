#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_String_0<> {
}

#[derive(Clone)]
struct Main_Address_0<> {
    city: Main_String_0,
    street: Main_String_0,
}

#[derive(Clone)]
struct Main_Person_0<> {
    name: Main_String_0,
    address: Main_Address_0,
}

fn Main_main_0() -> () {
    let i_0_0 : Main_String_0 = Main_String_0{};
    let i_0_1 : Main_String_0 = Main_String_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let tmp_1 = i_0_2;
    let i_0_4 : Main_String_0 = Main_String_0{};
    let i_0_5 : Main_Address_0 = tmp_1;
    let i_0_6 : Main_Person_0 = Main_Person_0{name: i_0_4, address: i_0_5};
    let tmp_2 = i_0_6;
    let i_0_8 : &Main_String_0 = &tmp_2.address.city;
    let i_0_9 : Main_Address_0 = tmp_2.address;
    let i_0_10 : () = Main_other_0();
    let i_0_11 : () = Main_other2_0();
    let i_0_12 : () = ();
    i_0_12
}

fn Main_other_0() -> () {
    let i_0_0 : Main_String_0 = Main_String_0{};
    let i_0_1 : Main_String_0 = Main_String_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let tmp_3 = i_0_2;
    let i_0_4 : Main_String_0 = Main_String_0{};
    let i_0_5 : Main_Address_0 = tmp_3;
    let i_0_6 : Main_Person_0 = Main_Person_0{name: i_0_4, address: i_0_5};
    let tmp_4 = i_0_6;
    let i_0_8 : bool = true;
    let i_0_9 : () = if i_0_8 {
        let i_1_0 : Main_String_0 = tmp_4.address.city;
        let i_1_1 : () = ();
        i_1_1
    } else {
        let i_2_0 : Main_Address_0 = tmp_4.address;
        let i_2_1 : () = ();
        i_2_1
    };
    i_0_9
}

fn Main_other2_0() -> () {
    let i_0_0 : Main_String_0 = Main_String_0{};
    let i_0_1 : Main_String_0 = Main_String_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let tmp_5 = i_0_2;
    let i_0_4 : Main_String_0 = Main_String_0{};
    let i_0_5 : Main_Address_0 = tmp_5;
    let i_0_6 : Main_Person_0 = Main_Person_0{name: i_0_4, address: i_0_5};
    let tmp_6 = i_0_6;
    let i_0_8 : bool = true;
    let i_0_9 : () = if i_0_8 {
        let i_1_0 : &Main_String_0 = &tmp_6.address.city;
        let i_1_1 : () = ();
        i_1_1
    } else {
        let i_2_0 : &Main_Address_0 = &tmp_6.address;
        let i_2_1 : () = ();
        i_2_1
    };
    let i_0_10 : Main_Person_0 = tmp_6;
    let i_0_11 : () = ();
    i_0_11
}

fn main() {
    Main_main_0();
}


