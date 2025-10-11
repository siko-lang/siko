use crate::siko::location::{
    Location::Location,
    Report::{Entry, Report, ReportContext},
};

pub enum BorrowCheckerError {
    UseAfterMove(String, Location, Location),
    UseAfterDrop(String, Location, Location),
}

impl BorrowCheckerError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            BorrowCheckerError::UseAfterMove(path, moveLocation, borrowLocation) => {
                let slogan = format!(
                    "Trying to move {} but there is a live borrow for {} and the borrow will be used later",
                    ctx.yellow(&path.to_string()),
                    ctx.yellow(&path.to_string())
                );
                let mut entries = Vec::new();
                entries.push(Entry::new(
                    Some("NOTE: Value moved here".to_string()),
                    moveLocation.clone(),
                ));
                entries.push(Entry::new(
                    Some("NOTE: Value borrowed here".to_string()),
                    borrowLocation.clone(),
                ));
                let r = Report::build(ctx, slogan, entries);
                r.print();
            }
            BorrowCheckerError::UseAfterDrop(path, createLocation, borrowLocation) => {
                let slogan = format!(
                    "Trying to drop {} but there is a live borrow for {} and the borrow will be used later",
                    ctx.yellow(&path.to_string()),
                    ctx.yellow(&path.to_string())
                );
                let mut entries = Vec::new();
                entries.push(Entry::new(
                    Some("NOTE: Value created here".to_string()),
                    createLocation.clone(),
                ));
                entries.push(Entry::new(
                    Some("NOTE: Value borrowed here".to_string()),
                    borrowLocation.clone(),
                ));
                let r = Report::build(ctx, slogan, entries);
                r.print();
            }
        }
        std::process::exit(1);
    }
}
