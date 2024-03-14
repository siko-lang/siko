use super::Location::FileId;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct FileManager {
    files: Rc<RefCell<Vec<String>>>,
}

impl FileManager {
    pub fn new() -> FileManager {
        FileManager {
            files: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add(&self, fileName: String) -> FileId {
        let id = FileId::new(self.files.borrow().len() as i64, self.clone());
        let mut files = self.files.borrow_mut();
        files.push(fileName);
        id
    }

    pub fn get(&self, id: i64) -> String {
        self.files
            .borrow()
            .get(id as usize)
            .expect("No file found")
            .clone()
    }
}
