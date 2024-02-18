#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct String_String {
}

#[derive(Clone)]
struct Main_Address {
    city: String_String,
    street: String_String,
}

#[derive(Clone)]
struct Main_Person {
    name: String_String,
    address: Main_Address,
}

#[derive(Clone)]
struct Main_Unit {
}

fn Main_main() -> () {
    let i_0_0 = String_String{};
    let i_0_1 = String_String{};
    let i_0_2 = Main_Address{city: i_0_0, street: i_0_1};
    let tmp_1 = i_0_2;
    let i_0_4 = String_String{};
    let i_0_5 = tmp_1;
    let i_0_6 = /* move */i_0_5;
    let i_0_7 = Main_Person{name: i_0_4, address: i_0_6};
    let tmp_2 = i_0_7;
    let i_0_9 = &tmp_2.address.city;
    let i_0_10 = /* convert */i_0_9;
    let i_0_11 = tmp_2.address;
    let i_0_12 = /* move */i_0_11;
    let i_0_13 = ();
    i_0_13
}

fn main() {
    Main_main();
}


