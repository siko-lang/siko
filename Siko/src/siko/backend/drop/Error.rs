use crate::siko::{
    backend::drop::Path::Path,
    location::Report::{Entry, Report, ReportContext},
};

pub enum Error {
    AlreadyMoved { path: Path, prevMove: Path },
}

pub fn reportErrors(ctx: &ReportContext, errors: Vec<Error>) {
    for error in &errors {
        match error {
            Error::AlreadyMoved { path, prevMove } => {
                let mut entries = Vec::new();
                entries.push(Entry::new(None, path.location.clone()));
                entries.push(Entry::new(
                    Some(format!("NOTE: previous move was here")),
                    prevMove.location.clone(),
                ));
                Report::build(
                    ctx,
                    format!("Value {} already moved", ctx.yellow(&path.userPath())),
                    entries,
                )
                .print();
            }
        }
    }
    if errors.len() > 0 {
        std::process::exit(1);
    }
}
