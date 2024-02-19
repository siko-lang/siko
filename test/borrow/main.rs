#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Object {
}

#[derive(Clone)]
struct Main_Name {
}

#[derive(Clone)]
struct Main_Address {
    street: Main_Name,
    city: Main_Name,
}

#[derive(Clone)]
struct Main_Person {
    address: Main_Address,
}

fn Main_main() -> () {
    let i_0_0 = Main_test1();
    let i_0_1 = Main_test2();
    let i_0_2 = ();
    i_0_2
}

fn Main_test1() -> () {
    let _block_1 = {
        let i_1_0 = Main_Object{};
        let tmp_1 = i_1_0;
        let i_1_2 = &tmp_1;
        let tmp_2 = i_1_2;
        let i_1_4 = tmp_1;
        let i_1_5 = tmp_2;
        i_1_5
    };
    let i_0_0 = _block_1;
    let tmp_3 = i_0_0;
    let i_0_2 = ();
    i_0_2
}

fn Main_test2() -> () {
    let i_0_0 = Main_Name{};
    let tmp_4 = i_0_0;
    let i_0_2 = Main_Name{};
    let tmp_5 = i_0_2;
    let i_0_4 = &tmp_4;
    let i_0_5 = tmp_5;
    let i_0_6 = Main_Address{street: i_0_4, city: i_0_5};
    let tmp_6 = i_0_6;
    let i_0_8 = tmp_6;
    let i_0_9 = Main_Person{address: i_0_8};
    let tmp_7 = i_0_9;
    let i_0_11 = tmp_4;
    let i_0_12 = ();
    i_0_12
}

fn main() {
    Main_main();
}


