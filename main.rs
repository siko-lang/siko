#![allow(non_camel_case_types)]
#![allow(unused_variables)]


struct String_String {
}

struct Bool_Bool {
}

struct Unit_Unit {
}

struct Main_Address {
    city: String_String,
    street: String_String,
}

struct Main_Person {
    name: String_String,
    address: Main_Address,
}

fn Main_check(arg_0: Main_Address) -> Unit_Unit {
    let i_0_0 = arg_0;
    let tmp_1 = i_0_0;
    let i_0_2 = Unit_Unit{};
    i_0_2
}

fn Main_foo() -> Unit_Unit {
    let i_0_0 = Unit_Unit{};
    i_0_0
}

fn Main_bar() -> Unit_Unit {
    let i_0_0 = Unit_Unit{};
    i_0_0
}

fn Main_loop_init_fn() -> Unit_Unit {
    let i_0_0 = Unit_Unit{};
    i_0_0
}

fn Main_id(arg_0: Main_Address) -> String_String {
    let i_0_0 = arg_0;
    let tmp_2 = i_0_0;
    let i_0_2 = Unit_Unit{};
    let i_0_3 = Unit_Unit{};
    let tmp_3 = i_0_3;
    let i_0_5 = Unit_Unit{};
    let tmp_4 = i_0_5;
    let i_0_7 = tmp_4;
    let i_0_8 = /*convert*/i_0_7;
    let i_0_9 = Main_eat(i_0_8);
    let i_0_10 = Unit_Unit{};
    let tmp_5 = i_0_10;
    let _block_1 = {
        let i_1_0 = Unit_Unit{};
        let tmp_6 = i_1_0;
        let _block_2 = {
            let i_2_0 = true;
            let _block_3 = {
                let i_3_0 = tmp_2.city;
                let i_3_1 = /*convert*/i_3_0;
                let i_3_2 = String_String{};
                let i_3_3 = Main_Address{city: i_3_1, street: i_3_2};
                let tmp_7 = i_3_3;
                let i_3_5 = String_String{};
                let i_3_6 = tmp_7;
                let i_3_7 = /*convert*/i_3_6;
                let i_3_8 = Main_Person{name: i_3_5, address: i_3_7};
                let tmp_8 = i_3_8;
                let i_3_10 = tmp_8.address.city;
                let i_3_11 = /*convert*/i_3_10;
                i_3_11
            };
            let _block_4 = {
                let i_4_0 = String_String{};
                let i_4_1 = tmp_2.street;
                let i_4_2 = /*convert*/i_4_1;
                let i_4_3 = Main_Address{city: i_4_0, street: i_4_2};
                let tmp_9 = i_4_3;
                let i_4_5 = String_String{};
                let i_4_6 = tmp_9;
                let i_4_7 = /*convert*/i_4_6;
                let i_4_8 = Main_Person{name: i_4_5, address: i_4_7};
                let tmp_10 = i_4_8;
                let i_4_10 = tmp_10.address.city;
                let i_4_11 = /*convert*/i_4_10;
                i_4_11
            };
            let i_2_1 = if i_2_0 { _block_3 } else { _block_4 };
            i_2_1
        };
        let i_1_2 = _block_2;
        i_1_2
    };
    let i_0_12 = _block_1;
    i_0_12
}

fn Main_eat(arg_0: Unit_Unit) -> Unit_Unit {
    let i_0_0 = arg_0;
    let tmp_11 = i_0_0;
    let i_0_2 = Unit_Unit{};
    i_0_2
}

fn Main_main() -> Unit_Unit {
    let i_0_0 = String_String{};
    let i_0_1 = String_String{};
    let i_0_2 = Main_Address{city: i_0_0, street: i_0_1};
    let tmp_12 = i_0_2;
    let i_0_4 = String_String{};
    let i_0_5 = tmp_12;
    let i_0_6 = /*convert*/i_0_5;
    let i_0_7 = Main_Person{name: i_0_4, address: i_0_6};
    let tmp_13 = i_0_7;
    let i_0_9 = tmp_13.address.city;
    let i_0_10 = /*convert*/i_0_9;
    let i_0_11 = tmp_13.address;
    let i_0_12 = /*convert*/i_0_11;
    let i_0_13 = Unit_Unit{};
    i_0_13
}

fn main() {
    Main_main();
}


