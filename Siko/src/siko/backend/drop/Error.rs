use crate::siko::backend::drop::Path::Path;

pub enum Error {
    AlreadyMoved { path: Path, prevMove: Path },
}
