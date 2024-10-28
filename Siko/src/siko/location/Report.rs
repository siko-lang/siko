use super::Location::Location;
use std::{
    fmt::Debug,
    io::{stdout, IsTerminal},
};

pub struct ReportContext {}

impl Debug for ReportContext {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ReportContext {
    pub fn new() -> ReportContext {
        ReportContext {}
    }

    pub fn yellow(&self, s: &str) -> String {
        if stdout().is_terminal() {
            format!("\x1b[33m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    pub fn red(&self, s: &str) -> String {
        if stdout().is_terminal() {
            format!("\x1b[31m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    pub fn blue(&self, s: &str) -> String {
        if stdout().is_terminal() {
            format!("\x1b[34m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }
}

pub struct Entry {
    msg: Option<String>,
    location: Location,
}

impl Entry {
    pub fn new(msg: Option<String>, location: Location) -> Entry {
        Entry {
            msg: msg,
            location: location,
        }
    }
}

pub struct Report<'a> {
    ctx: &'a ReportContext,
    slogan: String,
    entries: Vec<Entry>,
}

impl<'a> Report<'a> {
    pub fn new(ctx: &'a ReportContext, slogan: String, location: Option<Location>) -> Report<'a> {
        let mut entries = Vec::new();
        if let Some(loc) = location {
            entries.push(Entry::new(None, loc));
        }
        Report::build(ctx, slogan, entries)
    }

    pub fn build(ctx: &'a ReportContext, slogan: String, entries: Vec<Entry>) -> Report<'a> {
        Report {
            ctx: ctx,
            slogan: slogan,
            entries: entries,
        }
    }

    pub fn print(&self) {
        println!("{}: {}", self.ctx.red("ERROR"), self.slogan);
        for entry in &self.entries {
            if let Some(msg) = &entry.msg {
                println!("   {}", self.ctx.blue(msg));
            }
            println!(
                "{} {}:{}:{}",
                self.ctx.red(" --->"),
                entry.location.fileId.getFileName(),
                entry.location.span.start.line + 1,
                entry.location.span.start.offset + 1
            );
            let lines = entry.location.fileId.getLines();
            let startLine = entry.location.span.start.line;
            let endLine = entry.location.span.end.line;
            let mut separatorPrinted = false;
            for (lineNumber, line) in lines.iter().enumerate() {
                let lineNumber = lineNumber as i64;
                let distance = std::cmp::min((lineNumber - startLine).abs(), (endLine - lineNumber).abs());
                if lineNumber >= startLine && lineNumber <= endLine {
                    if distance < 3 {
                        let highlighted_line = if lineNumber == startLine && lineNumber == endLine {
                            let start = entry.location.span.start.offset as usize;
                            let end = entry.location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&self.ctx.yellow(&line[start..end]));
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else if lineNumber == startLine {
                            let start = entry.location.span.start.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&self.ctx.yellow(&line[start..]));
                            modifiedLine
                        } else if lineNumber == endLine {
                            let end = entry.location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&self.ctx.yellow(&line[..end]));
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else {
                            self.ctx.yellow(line)
                        };
                        println!(
                            " {} {} {}",
                            self.ctx.red("|"),
                            self.ctx.blue(&format!("{}", lineNumber + 1)),
                            highlighted_line
                        );
                    } else {
                        if !separatorPrinted {
                            separatorPrinted = true;
                            println!(" {}", "...");
                        }
                    }
                }
            }
        }
    }
}
