use crate::siko::qualifiedname::QualifiedName;
use base64::engine::general_purpose;
use base64::Engine;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct NameManager {
    usedNames: Rc<RefCell<BTreeMap<String, QualifiedName>>>,
}

impl NameManager {
    pub fn new() -> NameManager {
        NameManager {
            usedNames: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn processName(&self, name: &QualifiedName) -> String {
        let mut allowedLength = 3;
        loop {
            let converted = hashedName(name, allowedLength);
            {
                if let Some(existing) = self.usedNames.borrow().get(&converted) {
                    if existing != name {
                        // name clash
                        allowedLength += 1;
                        continue;
                    } else {
                        //println!("Reusing name: {} for {}", converted, name);
                        return converted;
                    }
                }
            }
            self.usedNames.borrow_mut().insert(converted.clone(), name.clone());
            //println!("New name: {} for {}", converted, name);
            return converted;
        }
    }
}

fn base64(s: &str) -> String {
    let encoded = general_purpose::STANDARD.encode(s);
    encoded
}

pub fn hashedName(name: &QualifiedName, length: usize) -> String {
    let (base, context) = name.split();
    let c = base64(&context.to_string()).replace('=', "").replace("+", "");
    if c.is_empty() {
        cleanupName(&base)
    } else {
        let c = if c.len() > length { &c[0..length] } else { &c };
        cleanupName(&base) + "_" + &c
    }
}

pub fn cleanupName<T: ToString>(name: &T) -> String {
    format!(
        "{}",
        name.to_string()
            .replace(".", "_")
            .replace("(", "_t_")
            .replace(")", "_t_")
            .replace("{", "")
            .replace("}", "")
            .replace(",", "_")
            .replace(" ", "_")
            .replace("*", "s")
            .replace("#", "_")
            .replace("/", "_")
            .replace("[", "_")
            .replace("]", "_")
            .replace("&", "_r_")
            .replace(">", "_l_")
            .replace("-", "_minus_")
            .replace(":", "_colon_")
    )
}
