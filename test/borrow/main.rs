#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]


#[derive(Clone)]
struct Main_Object {
}

fn Main_main() -> () {
    let _block_1 = {
        let i_1_0 = Main_Object{};
        let tmp_1 = i_1_0;
        let i_1_2 = &tmp_1;
        let i_1_3 = /* move */i_1_2;
        let tmp_2 = i_1_3;
        let i_1_5 = tmp_1;
        let i_1_6 = /* move */i_1_5;
        let i_1_7 = tmp_2;
        let i_1_8 = /* move */i_1_7;
        i_1_8
    };
    let i_0_0 = _block_1;
    let tmp_3 = i_0_0;
    let i_0_2 = ();
    i_0_2
}

fn main() {
    Main_main();
}


