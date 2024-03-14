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
    location: Location,
}

impl Report {
    pub fn new(slogan: String, location: Location) -> Report {
        Report {
            slogan: slogan,
            location: location,
        }
    }

    pub fn print(&self) {
        println!("{}: {}", "ERROR".red(), self.slogan);
        println!(
            "{} {}:{}:{}",
            " --->".red(),
            self.location.fileId.getFileName(),
            self.location.span.start.line + 1,
            self.location.span.start.offset + 1
        );
        let lines = self.location.fileId.getLines();
        let startLine = self.location.span.start.line;
        let endLine = self.location.span.end.line;
        let mut separatorPrinted = false;
        for (lineNumber, line) in lines.iter().enumerate() {
            let lineNumber = lineNumber as i64;
            let distance =
                std::cmp::min((lineNumber - startLine).abs(), (endLine - lineNumber).abs());
            if lineNumber >= startLine && lineNumber <= endLine {
                if distance < 3 {
                    let highlighted_line = if lineNumber == startLine && lineNumber == endLine {
                        let start = self.location.span.start.offset as usize;
                        let end = self.location.span.end.offset as usize;
                        let mut modifiedLine = String::new();
                        modifiedLine.push_str(&line[..start]);
                        modifiedLine.push_str(&line[start..end].yellow());
                        modifiedLine.push_str(&line[end..]);
                        modifiedLine
                    } else if lineNumber == startLine {
                        let start = self.location.span.start.offset as usize;
                        let mut modifiedLine = String::new();
                        modifiedLine.push_str(&line[..start]);
                        modifiedLine.push_str(&line[start..].yellow());
                        modifiedLine
                    } else if lineNumber == endLine {
                        let end = self.location.span.end.offset as usize;
                        let mut modifiedLine = String::new();
                        modifiedLine.push_str(&line[..end].yellow());
                        modifiedLine.push_str(&line[end..]);
                        modifiedLine
                    } else {
                        line.yellow()
                    };
                    println!(" {} {} {}", "|".red(), lineNumber, highlighted_line);
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
