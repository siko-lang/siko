#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct String_String_0<> {
}

#[derive(Clone)]
struct Main_Address_0<> {
    city: String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Address_1<'l0> {
    city: &'l0 String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Person_0<'l0> {
    name: String_String_0,
    address: Main_Address_1<'l0>,
}

#[derive(Clone)]
struct Main_Address_2<'l0> {
    city: String_String_0,
    street: &'l0 String_String_0,
}

#[derive(Clone)]
struct Main_Person_1<'l0> {
    name: String_String_0,
    address: Main_Address_2<'l0>,
}

fn Main_main_0() -> () {
    let i_0_0 : String_String_0 = String_String_0{};
    let i_0_1 : String_String_0 = String_String_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let tmp_7 = i_0_2;
    let i_0_4 : &Main_Address_0 = &tmp_7;
    let i_0_5 : String_String_0 = Main_id_0(i_0_4);
    let i_0_6 : Main_Address_0 = tmp_7;
    let i_0_7 : () = ();
    i_0_7
}

fn Main_id_0<'l0: >(arg_0: &'l0 Main_Address_0) -> String_String_0 {
    let i_0_0 : &Main_Address_0 = &arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : bool = true;
    let i_0_3 : String_String_0 = if i_0_2 {
        let i_1_0 : &String_String_0 = &tmp_1.city;
        let i_1_1 : String_String_0 = String_String_0{};
        let i_1_2 : Main_Address_1 = Main_Address_1{city: i_1_0, street: i_1_1};
        let tmp_2 = i_1_2;
        let i_1_4 : String_String_0 = String_String_0{};
        let i_1_5 : Main_Address_1 = tmp_2;
        let i_1_6 : Main_Person_0 = Main_Person_0{name: i_1_4, address: i_1_5};
        let tmp_3 = i_1_6;
        let i_1_8 : String_String_0 = tmp_3.address.city.clone();
        i_1_8
    } else {
        let i_2_0 : String_String_0 = String_String_0{};
        let i_2_1 : &String_String_0 = &tmp_1.street;
        let i_2_2 : Main_Address_2 = Main_Address_2{city: i_2_0, street: i_2_1};
        let tmp_4 = i_2_2;
        let i_2_4 : String_String_0 = String_String_0{};
        let i_2_5 : Main_Address_2 = tmp_4;
        let i_2_6 : Main_Person_1 = Main_Person_1{name: i_2_4, address: i_2_5};
        let tmp_5 = i_2_6;
        let i_2_8 : String_String_0 = tmp_5.address.city;
        i_2_8
    };
    i_0_3
}

fn main() {
    Main_main_0();
}


