#[macro_use]
pub mod siko_macros {
    macro_rules! partial_cmp_body {
        ($arg0:ident, $arg1:ident, $option_crate:tt :: $option_source:tt :: $option_module:tt :: $option_name:tt,
                                   $ordering_crate:tt :: $ordering_source:tt :: $ordering_module:tt :: $ordering_name:tt) => {
            match $arg0.value.partial_cmp(&$arg1.value) {
                Some(std::cmp::Ordering::Less) => {
                    $option_crate :: $option_source :: $option_module :: $option_name ::Some($ordering_crate:: $ordering_source:: $ordering_module :: $ordering_name::Less)
                }
                Some(std::cmp::Ordering::Equal) => $option_crate :: $option_source :: $option_module :: $option_name ::Some(
                    $ordering_crate:: $ordering_source:: $ordering_module :: $ordering_name::Equal
                ),
                Some(std::cmp::Ordering::Greater) => $option_crate :: $option_source :: $option_module :: $option_name ::Some(
                    $ordering_crate:: $ordering_source:: $ordering_module :: $ordering_name::Greater
                ),
                None => $option_crate :: $option_source :: $option_module :: $option_name ::None,
            }
        };
    }

    macro_rules! cmp_body {
        ($arg0:ident, $arg1:ident, $ordering_crate:tt :: $ordering_source:tt :: $ordering_module:tt :: $ordering_name:tt) => {
            match $arg0.value.cmp(&$arg1.value) {
                std::cmp::Ordering::Less => {
                    $ordering_crate::$ordering_source::$ordering_module::$ordering_name::Less
                }
                std::cmp::Ordering::Equal => {
                    $ordering_crate::$ordering_source::$ordering_module::$ordering_name::Equal
                }
                std::cmp::Ordering::Greater => {
                    $ordering_crate::$ordering_source::$ordering_module::$ordering_name::Greater
                }
            }
        };
    }

    macro_rules! map_insert {
        ($arg0:ident, $arg1:ident, $arg2:ident, $option_crate:tt :: $option_source:tt :: $option_module:tt :: $option_name:tt,
                                                $tuple_crate:tt :: $tuple_source:tt :: $tuple_module:tt :: $tuple_name:tt) => {{
            let mut arg0 = $arg0;
            let value = match arg0.value.insert($arg1, $arg2) {
                Some(v) => $option_crate::$option_source::$option_module::$option_name::Some(v),
                None => $option_crate::$option_source::$option_module::$option_name::None,
            };
            $tuple_crate::$tuple_source::$tuple_module::$tuple_name {
                field_0: arg0,
                field_1: value,
            }
        }};
    }

    macro_rules! map_remove {
        ($arg0:ident, $arg1:ident, $option_crate:tt :: $option_source:tt :: $option_module:tt :: $option_name:tt,
                                   $tuple_crate:tt :: $tuple_source:tt :: $tuple_module:tt :: $tuple_name:tt) => {{
            let mut arg0 = $arg0;
            let value = match arg0.value.remove(&$arg1) {
                Some(v) => $option_crate::$option_source::$option_module::$option_name::Some(v),
                None => $option_crate::$option_source::$option_module::$option_name::None,
            };
            $tuple_crate::$tuple_source::$tuple_module::$tuple_name {
                field_0: arg0,
                field_1: value,
            }
        }};
    }

    macro_rules! map_empty {
        ($map_crate:tt :: $map_source:tt :: $map_module:tt :: $map_name:tt) => {{
            let value = std::collections::BTreeMap::new();
            $map_crate::$map_source::$map_module::$map_name { value: value }
        }};
    }

    macro_rules! map_get {
        ($arg0:ident, $arg1:ident, $option_crate:tt :: $option_source:tt :: $option_module:tt :: $option_name:tt) => {{
            match $arg0.value.get(&$arg1) {
                Some(v) => {
                    $option_crate::$option_source::$option_module::$option_name::Some(v.clone())
                }
                None => $option_crate::$option_source::$option_module::$option_name::None,
            }
        }};
    }
}

mod source;

use crate::source::Main;

fn main() {
    crate::Main::main_0();
}