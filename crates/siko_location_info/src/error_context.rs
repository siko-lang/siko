use crate::file_manager::FileManager;
use crate::location_id::LocationId;
use crate::location_info::LocationInfo;

pub struct ErrorContext {
    pub file_manager: FileManager,
    pub location_info: LocationInfo,
}

impl ErrorContext {
    pub fn report_error(&self, msg: String, _location: LocationId) {
        println!("ERROR: {}", msg); // TODO
    }
}
