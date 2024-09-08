use super::Location::Location;

pub trait Painter {
    fn yellow(&self) -> String;
    fn red(&self) -> String;
}

impl Painter for str {
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }

    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }
}

pub struct Report {
    slogan: String,
    location: Option<Location>,
}

impl Report {
    pub fn new(slogan: String, location: Option<Location>) -> Report {
        Report {
            slogan: slogan,
            location: location,
        }
    }

    pub fn print(&self) {
        println!("{}: {}", "ERROR".red(), self.slogan);
        if let Some(location) = &self.location {
            println!(
                "{} {}:{}:{}",
                " --->".red(),
                location.fileId.getFileName(),
                location.span.start.line + 1,
                location.span.start.offset + 1
            );
            let lines = location.fileId.getLines();
            let startLine = location.span.start.line;
            let endLine = location.span.end.line;
            let mut separatorPrinted = false;
            for (lineNumber, line) in lines.iter().enumerate() {
                let lineNumber = lineNumber as i64;
                let distance =
                    std::cmp::min((lineNumber - startLine).abs(), (endLine - lineNumber).abs());
                if lineNumber >= startLine && lineNumber <= endLine {
                    if distance < 3 {
                        let highlighted_line = if lineNumber == startLine && lineNumber == endLine {
                            let start = location.span.start.offset as usize;
                            let end = location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&line[start..end].yellow());
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else if lineNumber == startLine {
                            let start = location.span.start.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..start]);
                            modifiedLine.push_str(&line[start..].yellow());
                            modifiedLine
                        } else if lineNumber == endLine {
                            let end = location.span.end.offset as usize;
                            let mut modifiedLine = String::new();
                            modifiedLine.push_str(&line[..end].yellow());
                            modifiedLine.push_str(&line[end..]);
                            modifiedLine
                        } else {
                            line.yellow()
                        };
                        println!(" {} {} {}", "|".red(), lineNumber + 1, highlighted_line);
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
