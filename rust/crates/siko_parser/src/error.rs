use siko_location_info::filepath::FilePath;
use siko_location_info::location::Location;

#[derive(Debug)]
pub struct LocationInfo {
    pub file_path: FilePath,
    pub location: Location,
}

#[derive(Debug)]
pub enum LexerError {
    UnsupportedCharacter(char, LocationInfo),
    General(String, FilePath, Location),
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub file_path: FilePath,
    pub location: Location,
}

impl ParseError {
    pub fn new(msg: String, file_path: FilePath, location: Location) -> ParseError {
        ParseError {
            msg: msg,
            file_path: file_path,
            location: location,
        }
    }
}
