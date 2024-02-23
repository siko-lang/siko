#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct String_String_0 {
}

#[derive(Clone)]
struct Main_Address_0 {
    city: String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Person_0 {
    name: String_String_0,
    address: Main_Address_0,
}

#[derive(Clone)]
struct Main_Unit_0 {
}

#[derive(Clone)]
struct Main_Address_1 {
    city: String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Person_1 {
    name: String_String_0,
    address: Main_Address_2,
}

#[derive(Clone)]
struct Main_Address_3 {
    city: String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Person_2 {
    name: String_String_0,
    address: Main_Address_4,
}

#[derive(Clone)]
struct Main_Address_2 {
    city: String_String_0,
    street: String_String_0,
}

#[derive(Clone)]
struct Main_Address_4 {
    city: String_String_0,
    street: String_String_0,
}

fn Main_main_0() -> () {
    let i_0_0 : String_String_0 = String_String_0{};
    let i_0_1 : String_String_0 = String_String_0{};
    let i_0_2 : Main_Address_0 = Main_Address_0{city: i_0_0, street: i_0_1};
    let tmp_11 = i_0_2;
    let i_0_4 : &Main_Address_0 = &tmp_11;
    let i_0_5 : String_String_0 = Main_id_0(i_0_4);
    let i_0_6 : String_String_0 = String_String_0{};
    let i_0_7 : Main_Address_0 = tmp_11;
    let i_0_8 : Main_Person_0 = Main_Person_0{name: i_0_6, address: i_0_7};
    let tmp_12 = i_0_8;
    let i_0_10 : &String_String_0 = &tmp_12.address.city;
    let i_0_11 : Main_Address_0 = tmp_12.address;
    let i_0_12 : () = ();
    i_0_12
}

fn Main_id_0(arg_0: &Main_Address_0) -> String_String_0 {
    let i_0_0 : &Main_Address_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 : Main_Unit_0 = Main_Unit_0{};
    let i_0_3 : Main_Unit_0 = Main_Unit_0{};
    let tmp_2 = i_0_3;
    let i_0_5 : Main_Unit_0 = Main_Unit_0{};
    let tmp_3 = i_0_5;
    let i_0_7 : Main_Unit_0 = tmp_3;
    let i_0_8 : () = Main_eat_0(i_0_7);
    let i_0_9 : Main_Unit_0 = Main_Unit_0{};
    let tmp_4 = i_0_9;
    let _block_1 = {
        let i_1_0 : Main_Unit_0 = Main_Unit_0{};
        let tmp_5 = i_1_0;
        let _block_2 = {
            let i_2_0 : bool = true;
            let i_2_1 : String_String_0 = if i_2_0 {
                let i_3_0 : &String_String_0 = tmp_1.city;
                let i_3_1 : String_String_0 = String_String_0{};
                let i_3_2 : Main_Address_1 = Main_Address_1{city: i_3_0, street: i_3_1};
                let tmp_6 = i_3_2;
                let i_3_4 : String_String_0 = String_String_0{};
                let i_3_5 : Main_Address_1 = tmp_6;
                let i_3_6 : Main_Person_1 = Main_Person_1{name: i_3_4, address: i_3_5};
                let tmp_7 = i_3_6;
                let i_3_8 : String_String_0 = tmp_7.address.city;
                i_3_8
            } else {
                let i_4_0 : String_String_0 = String_String_0{};
                let i_4_1 : &String_String_0 = tmp_1.street;
                let i_4_2 : Main_Address_3 = Main_Address_3{city: i_4_0, street: i_4_1};
                let tmp_8 = i_4_2;
                let i_4_4 : String_String_0 = String_String_0{};
                let i_4_5 : Main_Address_3 = tmp_8;
                let i_4_6 : Main_Person_2 = Main_Person_2{name: i_4_4, address: i_4_5};
                let tmp_9 = i_4_6;
                let i_4_8 : String_String_0 = tmp_9.address.city;
                i_4_8
            };
            i_2_1
        };
        let i_1_2 : String_String_0 = _block_2;
        i_1_2
    };
    let i_0_11 : String_String_0 = _block_1;
    i_0_11
}

fn Main_eat_0(arg_0: Main_Unit_0) -> () {
    let i_0_0 : Main_Unit_0 = arg_0;
    let tmp_10 = i_0_0;
    let i_0_2 : () = ();
    i_0_2
}

fn main() {
    Main_main_0();
}


