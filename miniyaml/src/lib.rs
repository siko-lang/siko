#![allow(non_snake_case)]

#[derive(Debug)]
pub enum YamlValue {
    String(String),
    List(Vec<YamlValue>),
    Map(Vec<(String, YamlValue)>),
}

impl YamlValue {
    pub fn prettyPrint(&self, indent: usize, emptyLine: bool) {
        let indentStr = " ".repeat(indent);
        match self {
            YamlValue::String(s) => {
                if emptyLine {
                    println!("{}{}", indentStr, s);
                } else {
                    println!("{}", s);
                }
            }
            YamlValue::List(l) => {
                for item in l {
                    print!("{} - ", indentStr);
                    item.prettyPrint(indent + 3, false);
                }
            }
            YamlValue::Map(m) => {
                for (index, (k, v)) in m.iter().enumerate() {
                    if index == 0 && !emptyLine {
                        print!("{}:", k);
                    } else {
                        print!("{}{}:", indentStr, k);
                    }
                    match v {
                        YamlValue::String(s) => {
                            println!(" {}", s);
                        }
                        _ => {
                            println!();
                            v.prettyPrint(indent + 2, true);
                        }
                    }
                }
            }
        }
    }
}

fn getIdentSize(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}

fn getLineKind(line: &str) -> LineKind {
    for (index, c) in line.chars().enumerate() {
        if c == ':' {
            return LineKind::Value(index);
        }
        if c == '-' {
            return LineKind::ListItem(index);
        }
        if c == '"' {
            return LineKind::String;
        }
    }
    LineKind::String
}

#[derive(Debug)]
pub enum Error {
    InconsistentValue,
}

enum LineKind {
    Value(usize),
    ListItem(usize),
    String,
}

pub struct Parser {
    lines: Vec<String>,
    currentLine: usize,
    errors: Vec<Error>,
}

enum LineParseResult {
    ValueName(String),
    Value(String, YamlValue),
    ListItem,
    String(YamlValue),
}

impl Parser {
    pub fn new(input: &String) -> Parser {
        let mut lines = Vec::new();
        for line in input.lines() {
            if line.is_empty() {
                continue;
            }
            if let LineKind::ListItem(index) = getLineKind(line) {
                let indent = getIdentSize(line);
                let indentStr = " ".repeat(indent + 1);
                lines.push(line[0..index + 1].to_string());
                lines.push(indentStr + &line[index + 1..].to_string());
            } else {
                lines.push(line.to_string());
            }
        }
        // for line in &lines {
        //     println!("Line: '{}'", line);
        // }
        Parser {
            lines,
            currentLine: 0,
            errors: Vec::new(),
        }
    }

    fn getCurrentLine(&self) -> Option<String> {
        if self.currentLine < self.lines.len() {
            Some(self.lines[self.currentLine].clone())
        } else {
            None
        }
    }

    fn stepLine(&mut self) {
        self.currentLine += 1;
    }

    pub fn parseValue(&mut self, indent: usize) -> Result<YamlValue, Error> {
        let mut map = Vec::new();
        let mut list = Vec::new();
        while let Some(line) = self.getCurrentLine() {
            let currentIdent = getIdentSize(&line);
            if currentIdent < indent {
                break;
            }
            let line = line.trim();
            match self.parseLine(&line) {
                LineParseResult::ValueName(name) => {
                    self.stepLine();
                    let value = self.parseValue(currentIdent + 1)?;
                    map.push((name, value));
                }
                LineParseResult::Value(name, value) => {
                    map.push((name, value));
                    self.stepLine();
                }
                LineParseResult::ListItem => {
                    self.stepLine();
                    let value = self.parseValue(currentIdent + 1)?;
                    list.push(value);
                }
                LineParseResult::String(value) => {
                    self.stepLine();
                    return Ok(value);
                }
            }
        }
        if !list.is_empty() && !map.is_empty() {
            self.errors.push(Error::InconsistentValue);
            return Err(Error::InconsistentValue);
        }
        if !list.is_empty() {
            Ok(YamlValue::List(list))
        } else {
            Ok(YamlValue::Map(map))
        }
    }

    pub fn parseString(&mut self, line: &str) -> YamlValue {
        if line.starts_with('"') && line.ends_with('"') {
            let value = &line[1..line.len() - 1];
            return YamlValue::String(value.to_string());
        }
        YamlValue::String(line.to_string())
    }

    fn parseLine(&mut self, line: &str) -> LineParseResult {
        //println!("Line: {}", line);
        let line = line.trim();
        match getLineKind(&line) {
            LineKind::Value(index) => {
                let name = line[0..index].trim();
                //println!("Value name: {}", name);
                let rest = &line[index + 1..];
                if rest.trim().is_empty() {
                    //println!("Value: <empty>");
                    return LineParseResult::ValueName(name.to_string());
                } else {
                    let value = self.parseString(rest.trim());
                    // println!("Value: {}", rest.trim());
                    return LineParseResult::Value(name.to_string(), value);
                }
            }
            LineKind::ListItem(_) => {
                assert_eq!(&line[0..1], "-");
                assert_eq!(line.len(), 1);
                return LineParseResult::ListItem;
            }
            LineKind::String => {
                return LineParseResult::String(self.parseString(line));
            }
        }
    }
}

#[test]
fn test1() {
    let input = "
users:
  - name: Alice
    age: 30
    email: alice@example.com

  - name: Bob
    age: 25
    email: bob@example.com

  -name: Charlie
   age:
        28
   email: charlie@example.com";
    let mut parser = Parser::new(&input.to_string());
    let value = parser.parseValue(0).expect("Failed to parse YAML");
    value.prettyPrint(0, true);
}

#[test]
fn test2() {
    //     let input = "
    // package:
    //     name: example
    //     version: 1.0.0
    //     authors:
    //         - Alice <alice@example.com>
    //         - Bob <bob@example.com>
    //         - Charlie <charlie@example.com>";
    let input = "
    package:
        name: example
        version: 1.0.0
        authors:
            - Alice <alice@example.com>
            - Bob <bob@example.com>
            - Charlie <charlie@example.com>
        dependencies:
            - serde: ^1.0
            - regex: ^1.5
            - tokio: ^1.0";
    let mut parser = Parser::new(&input.to_string());
    let value = parser.parseValue(0).expect("Failed to parse YAML");
    value.prettyPrint(0, true);
}
