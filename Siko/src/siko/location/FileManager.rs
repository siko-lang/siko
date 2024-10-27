use super::Location::FileId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

struct File {
    id: FileId,
    content: String,
}

#[derive(Clone)]
pub struct FileManager {
    names: Rc<RefCell<BTreeMap<String, FileId>>>,
    files: Rc<RefCell<BTreeMap<FileId, String>>>,
}

impl FileManager {
    pub fn new() -> FileManager {
        FileManager {
            names: Rc::new(RefCell::new(BTreeMap::new())),
            files: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn add(&self, fileName: String) -> FileId {
        let mut names = self.names.borrow_mut();
        if let Some(id) = names.get(&fileName) {
            return id.clone();
        }
        let id = FileId::new((self.files.borrow().len() + 1) as i64, self.clone());
        names.insert(fileName.clone(), id.clone());
        let mut files = self.files.borrow_mut();
        files.insert(id.clone(), fileName);
        id
    }

    pub fn get(&self, id: &FileId) -> String {
        self.files.borrow().get(id).expect("No file found").clone()
    }
}
