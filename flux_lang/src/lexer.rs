use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i32),
    Ident(String),
    String(String),

    // Keywords
    Let,
    In,
    If,
    Then,
    Else,
    Match,
    With,
    Data,
    Type,
    Unsafe,
    True,
    False,
    Unit,

    // Effect keywords
    Pure,
    IO,
    State,
    Debug,
    Cpu,
    Gpu,
    Alloc,
    None_,
    Arena,
    Heap,
    Concurrent,
    Single,

    // Parallel primitives
    ParFor,
    ParMap,
    ParMapInplace,
    Async,
    Await,

    // Actor primitives
    Actor,
    Send,

    // Array primitives
    NewArray,
    ArrayGet,
    ArraySet,
    ArrayLen,

    // GPU primitives
    GpuKernel,
    CpuToGpu,
    GpuToCpu,

    // Debug primitives
    Log,
    Assert,

    // Unsafe primitives
    RawThreadSpawn,
    AtomicLoad,
    AtomicStore,

    // Symbols
    Arrow,        // ->
    FatArrow,     // =>
    Colon,        // :
    DoubleColon,  // ::
    Comma,        // ,
    Dot,          // .
    Pipe,         // |
    Eq,           // =
    EqEq,         // ==
    Ne,           // !=
    Lt,           // <
    Le,           // <=
    Gt,           // >
    Ge,           // >=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    And,          // &&
    Or,           // ||
    Bang,         // !

    // Delimiters
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.current() == Some('-') && self.peek(1) == Some('-') {
            while let Some(c) = self.current() {
                self.advance();
                if c == '\n' {
                    break;
                }
            }
        }
    }

    fn read_number(&mut self) -> i32 {
        let mut num = String::new();

        if self.current() == Some('-') {
            num.push('-');
            self.advance();
        }

        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                num.push(c);
                self.advance();
            } else {
                break;
            }
        }

        num.parse().unwrap_or(0)
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();

        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' || c == '\'' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        ident
    }

    fn read_string(&mut self) -> String {
        let mut s = String::new();
        self.advance(); // skip opening quote

        while let Some(c) = self.current() {
            if c == '"' {
                self.advance(); // skip closing quote
                break;
            } else if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current() {
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => s.push(escaped),
                    }
                    self.advance();
                }
            } else {
                s.push(c);
                self.advance();
            }
        }

        s
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            if self.current() == Some('-') && self.peek(1) == Some('-') {
                self.skip_comment();
                continue;
            }

            break;
        }

        match self.current() {
            None => Token::Eof,
            Some(c) => {
                if c.is_ascii_digit() || (c == '-' && self.peek(1).map_or(false, |p| p.is_ascii_digit())) {
                    Token::Int(self.read_number())
                } else if c.is_alphabetic() || c == '_' {
                    let ident = self.read_ident();
                    match ident.as_str() {
                        "let" => Token::Let,
                        "in" => Token::In,
                        "if" => Token::If,
                        "then" => Token::Then,
                        "else" => Token::Else,
                        "match" => Token::Match,
                        "with" => Token::With,
                        "data" => Token::Data,
                        "type" => Token::Type,
                        "unsafe" => Token::Unsafe,
                        "true" => Token::True,
                        "false" => Token::False,
                        "unit" => Token::Unit,
                        "pure" => Token::Pure,
                        "io" => Token::IO,
                        "state" => Token::State,
                        "debug" => Token::Debug,
                        "cpu" => Token::Cpu,
                        "gpu" => Token::Gpu,
                        "alloc" => Token::Alloc,
                        "none" => Token::None_,
                        "arena" => Token::Arena,
                        "heap" => Token::Heap,
                        "concurrent" => Token::Concurrent,
                        "single" => Token::Single,
                        "par_for" => Token::ParFor,
                        "par_map" => Token::ParMap,
                        "par_map_inplace" => Token::ParMapInplace,
                        "async" => Token::Async,
                        "await" => Token::Await,
                        "actor" => Token::Actor,
                        "send" => Token::Send,
                        "new_array" => Token::NewArray,
                        "array_get" => Token::ArrayGet,
                        "array_set" => Token::ArraySet,
                        "array_len" => Token::ArrayLen,
                        "gpu_kernel" => Token::GpuKernel,
                        "cpu_to_gpu" => Token::CpuToGpu,
                        "gpu_to_cpu" => Token::GpuToCpu,
                        "log" => Token::Log,
                        "assert" => Token::Assert,
                        "raw_thread_spawn" => Token::RawThreadSpawn,
                        "atomic_load" => Token::AtomicLoad,
                        "atomic_store" => Token::AtomicStore,
                        _ => Token::Ident(ident),
                    }
                } else if c == '"' {
                    Token::String(self.read_string())
                } else {
                    self.advance();
                    match c {
                        '(' => Token::LParen,
                        ')' => Token::RParen,
                        '{' => Token::LBrace,
                        '}' => Token::RBrace,
                        '[' => Token::LBracket,
                        ']' => Token::RBracket,
                        ',' => Token::Comma,
                        '.' => Token::Dot,
                        '|' => Token::Pipe,
                        '+' => Token::Plus,
                        '*' => Token::Star,
                        '/' => Token::Slash,
                        '%' => Token::Percent,
                        ':' => {
                            if self.current() == Some(':') {
                                self.advance();
                                Token::DoubleColon
                            } else {
                                Token::Colon
                            }
                        },
                        '-' => {
                            if self.current() == Some('>') {
                                self.advance();
                                Token::Arrow
                            } else {
                                Token::Minus
                            }
                        },
                        '=' => {
                            if self.current() == Some('=') {
                                self.advance();
                                Token::EqEq
                            } else if self.current() == Some('>') {
                                self.advance();
                                Token::FatArrow
                            } else {
                                Token::Eq
                            }
                        },
                        '!' => {
                            if self.current() == Some('=') {
                                self.advance();
                                Token::Ne
                            } else {
                                Token::Bang
                            }
                        },
                        '<' => {
                            if self.current() == Some('=') {
                                self.advance();
                                Token::Le
                            } else {
                                Token::Lt
                            }
                        },
                        '>' => {
                            if self.current() == Some('=') {
                                self.advance();
                                Token::Ge
                            } else {
                                Token::Gt
                            }
                        },
                        '&' => {
                            if self.current() == Some('&') {
                                self.advance();
                                Token::And
                            } else {
                                Token::Ident(format!("&"))
                            }
                        },
                        '|' if self.current() == Some('|') => {
                            self.advance();
                            Token::Or
                        },
                        _ => Token::Ident(c.to_string()),
                    }
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 42 in x + 1");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Ident("x".to_string()));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::Int(42));
    }

    #[test]
    fn test_effects() {
        let mut lexer = Lexer::new("!{pure, cpu, alloc none}");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::Bang);
        assert_eq!(tokens[1], Token::LBrace);
        assert_eq!(tokens[2], Token::Pure);
    }
}
