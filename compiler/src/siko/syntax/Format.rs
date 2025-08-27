#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Chunk(String),
    StartOfBlock,
    StartOfItem,
    EndOfItem,
    EndOfBlock,
    NewLine,
    Indent,
    UnIndent,
    PushOffset,
    Break,
    PopOffset,
}

#[derive(Debug)]
struct PrinterState {
    output: String,
    index: usize,
    indent: i32,
    saved_offsets: Vec<usize>,
    line_start: usize,
    total_len: usize,
}

impl PrinterState {
    fn new() -> Self {
        PrinterState {
            output: String::new(),
            index: 0,
            indent: 0,
            saved_offsets: Vec::new(),
            line_start: 0,
            total_len: 0,
        }
    }

    fn inc(&mut self) {
        self.indent += 1;
    }

    fn dec(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn step(&mut self) {
        self.index += 1;
    }

    fn start_line(&mut self) {
        self.output.push('\n');
        self.total_len += 1;
        self.line_start = self.total_len;
    }

    fn add_string(&mut self, s: &str) {
        self.output.push_str(s);
        self.total_len += s.chars().count();
    }

    fn push_offset(&mut self) {
        self.saved_offsets.push(self.total_len - self.line_start);
    }

    fn pop_offset(&mut self) {
        self.saved_offsets.pop();
    }
}

fn get_spaces(indent: usize) -> String {
    " ".repeat(indent)
}

pub fn format_tokens(tokens: &[Token]) -> String {
    let mut state = PrinterState::new();

    while state.index < tokens.len() {
        match &tokens[state.index] {
            Token::Chunk(s) => state.add_string(s),
            Token::StartOfBlock => state.inc(),
            Token::StartOfItem => {
                let spaces = get_spaces((state.indent * 4) as usize);
                state.start_line();
                state.add_string(&spaces);
            }
            Token::EndOfItem => {}
            Token::Indent => state.inc(),
            Token::UnIndent => state.dec(),
            Token::EndOfBlock => state.dec(),
            Token::NewLine => state.start_line(),
            Token::PushOffset => state.push_offset(),
            Token::Break => {
                if let Some(&offset) = state.saved_offsets.last() {
                    let spaces = get_spaces(offset);
                    state.start_line();
                    state.add_string(&spaces);
                }
            }
            Token::PopOffset => state.pop_offset(),
        }
        state.step();
    }

    state.output
}

pub trait Format {
    fn format(&self) -> Vec<Token>;
}

pub fn format_list<T: Format>(items: &[T], separator: Token) -> Vec<Token> {
    let mut output = vec![Token::PushOffset];

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            let item_output = item.format();
            if !item_output.is_empty() {
                output.push(separator.clone());
                output.extend(item_output);
            }
        } else {
            output.extend(item.format());
        }
    }

    output.push(Token::PopOffset);
    output
}

pub fn format_list2<T: Format>(items: &[T], separator: &[Token]) -> Vec<Token> {
    let mut output = Vec::new();

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            let item_output = item.format();
            if !item_output.is_empty() {
                output.extend_from_slice(separator);
                output.extend(item_output);
            }
        } else {
            output.extend(item.format());
        }
    }

    output
}

pub fn format_block<T: Format>(items: &[T]) -> Vec<Token> {
    let mut output = Vec::new();
    output.extend(format_block_header());

    for item in items {
        let item_output = item.format();
        if !item_output.is_empty() {
            output.push(Token::StartOfItem);
            output.extend(item_output);
            output.push(Token::EndOfItem);
        }
    }

    output.extend(format_block_footer());
    output
}

pub fn format_block_2_items<T1: Format, T2: Format>(items1: &[T1], items2: &[T2]) -> Vec<Token> {
    let mut output = Vec::new();
    output.extend(format_block_header());

    for item in items1 {
        let item_output = item.format();
        if !item_output.is_empty() {
            output.push(Token::StartOfItem);
            output.extend(item_output);
            output.push(Token::EndOfItem);
        }
    }
    for item in items2 {
        let item_output = item.format();
        if !item_output.is_empty() {
            output.push(Token::StartOfItem);
            output.extend(item_output);
            output.push(Token::EndOfItem);
        }
    }
    output.extend(format_block_footer());
    output
}

pub fn format_block_header() -> Vec<Token> {
    vec![Token::Chunk(" {".to_string()), Token::StartOfBlock]
}

pub fn format_block_footer() -> Vec<Token> {
    vec![
        Token::EndOfBlock,
        Token::StartOfItem,
        Token::Chunk("}".to_string()),
        Token::EndOfItem,
    ]
}

pub fn format_block_inner<T: Format>(items: &[T]) -> Vec<Token> {
    let mut output = Vec::new();
    for item in items {
        let item_output = item.format();
        if !item_output.is_empty() {
            output.push(Token::StartOfItem);
            output.extend(item_output);
            output.push(Token::EndOfItem);
        }
    }

    output
}

impl Format for String {
    fn format(&self) -> Vec<Token> {
        vec![Token::Chunk(self.clone())]
    }
}

impl Format for &str {
    fn format(&self) -> Vec<Token> {
        vec![Token::Chunk(self.to_string())]
    }
}

impl<T: Format> Format for Vec<T> {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();
        for item in self {
            result.extend(item.format());
        }
        result
    }
}

impl<T: Format> Format for Option<T> {
    fn format(&self) -> Vec<Token> {
        match self {
            Some(item) => item.format(),
            None => Vec::new(),
        }
    }
}

pub fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result
}

pub fn escape_char(c: char) -> String {
    match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c => c.to_string(),
    }
}

pub fn format_any<T: Format>(item: &T) {
    let tokens = item.format();
    println!("{}", format_tokens(&tokens));
}
