use super::Location::Location;

pub trait Painter {
    fn yellow(&self) -> String;
    fn red(&self) -> String;
    fn blue(&self) -> String;
}

impl Painter for str {
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }

    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }

    fn blue(&self) -> String {
        format!("\x1b[34m{}\x1b[0m", self)
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

pub struct Report {
    slogan: String,
    entries: Vec<Entry>,
}

impl Report {
    pub fn new(slogan: String, location: Option<Location>) -> Report {
        let mut entries = Vec::new();
        if let Some(loc) = location {
            entries.push(Entry::new(None, loc));
        }
        Report::build(slogan, entries)
    }

    pub fn build(slogan: String, entries: Vec<Entry>) -> Report {
        Report {
            slogan: slogan,
            entries: entries,
        }
    }

    pub fn print(&self) {
        println!("{}: {}", "ERROR".red(), self.slogan);
        for entry in &self.entries {
            if let Some(msg) = &entry.msg {
                println!("   {}", msg.blue());
            }
            println!(
                "{} {}:{}:{}",
                " --->".red(),
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
                let distance =
                    std::cmp::min((lineNumber - startLine).abs(), (endLine - lineNumber).abs());
                if lineNumber >= startLine && lineNumber <= endLine {
                    if distance < 3 {
                        let highlighted_line = if lineNumber == startLine && lineNumber == endLine {
                            let start = entry.location.span.start.offset as usize;
                            let end = entry.location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&line[start..end].yellow());
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else if lineNumber == startLine {
                            let start = entry.location.span.start.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&line[start..].yellow());
                            modifiedLine
                        } else if lineNumber == endLine {
                            let end = entry.location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..end].yellow());
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else {
                            line.yellow()
                        };
                        println!(
                            " {} {} {}",
                            "|".red(),
                            format!("{}", lineNumber + 1).blue(),
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
