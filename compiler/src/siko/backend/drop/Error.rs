use crate::siko::{
    backend::path::Path::Path,
    location::Report::{Entry, Report, ReportContext},
};

pub enum Error {
    AlreadyMoved { path: Path, prevMove: Path },
}

pub fn reportErrors(ctx: &ReportContext, errors: Vec<Error>) {
    for error in &errors {
        match error {
            Error::AlreadyMoved { path, prevMove } => {
                if prevMove.location == path.location {
                    Report::build(
                        ctx,
                        format!(
                            "Value {} moved in previous iteration of loop",
                            ctx.yellow(&path.userPath())
                        ),
                        vec![Entry::new(None, path.location.clone())],
                    )
                    .print();
                } else {
                    if path.isRootOnly() {
                        let mut entries = Vec::new();
                        entries.push(Entry::new(None, path.location.clone()));
                        entries.push(Entry::new(
                            Some(format!("NOTE: previously moved here")),
                            prevMove.location.clone(),
                        ));
                        Report::build(
                            ctx,
                            format!("Value {} already moved", ctx.yellow(&path.userPath())),
                            entries,
                        )
                        .print();
                    } else {
                        let mut entries = Vec::new();
                        entries.push(Entry::new(None, path.location.clone()));
                        entries.push(Entry::new(
                            Some(format!("NOTE: previously moved here")),
                            prevMove.location.clone(),
                        ));
                        Report::build(
                            ctx,
                            format!("Value {} already moved", ctx.yellow(&prevMove.userPath())),
                            entries,
                        )
                        .print();
                    }
                }
            }
        }
    }
    if errors.len() > 0 {
        std::process::exit(1);
    }
}
