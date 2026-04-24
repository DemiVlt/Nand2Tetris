use std::{fs, iter::once, path::PathBuf};

#[derive(PartialEq)]
pub enum Token {
    Keyword(String),
    Symbol(char),
    Identifier(String),
    IntConst(u32),
    StringConst(String),
    None,
}

impl Token {
    pub fn process(t: Token) -> String {
        return match t {
            Token::Keyword(kw) => format!("<keyword> {kw} </keyword>"),
            Token::Symbol(c) => format!("<symbol> {c} </symbol>"),
            Token::Identifier(name) => format!("<identifier> {name} </identifier>"),
            Token::IntConst(x) => format!("<intConst> {x} </intConst>"),
            Token::StringConst(s) => format!("<stringConst> {s} </stringConst>"),
            Token::None => panic!("Shouldn't be called when there's no current token."),
        } + "\n";
    }
}

const SYMBOLS: [char; 19] = [
    '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=', '~',
];

const KEYWORDS: [&str; 21] = [
    "class",
    "constructor",
    "function",
    "method",
    "field",
    "static",
    "var",
    "int",
    "char",
    "boolean",
    "void",
    "true",
    "false",
    "null",
    "this",
    "let",
    "do",
    "if",
    "else",
    "while",
    "return",
];

pub trait TakeWhileRef {
    fn take_while_ref<P>(&mut self, predicate: P) -> std::vec::IntoIter<Token>
    where
        P: FnMut(&Token) -> bool;
}

impl<T: Iterator<Item = Token>> TakeWhileRef for T {
    fn take_while_ref<P>(&mut self, mut predicate: P) -> std::vec::IntoIter<Token>
    where
        P: FnMut(&Token) -> bool,
    {
        let mut ret = Vec::new();

        while let Some(next) = self.next() {
            if predicate(&next) {
                ret.push(next);
            } else {
                once(next).chain(self);
                break;
            }
        }

        ret.into_iter()
    }
}


/// Provides routines that skip comments and white space, get the next token, and advance the input
/// exactly beyond it. Other routines return the type of the current token, and its value.

pub struct JackTokenizer {
    jack_code: String,
}

impl Iterator for JackTokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_more_tokens() {
            return None;
        }

        let first_char = self.jack_code.remove(0);

        if SYMBOLS.contains(&first_char) {
            return Some(Token::Symbol(first_char));
        }

        if let Some(first_digit) = first_char.to_digit(10) {
            let (s, rest) = self
                .jack_code
                .split_once(|c: char| !c.is_digit(10))
                .unwrap();

            let ret = Token::IntConst(
                first_digit * 10u32.pow(s.len() as u32) + s.parse::<u32>().unwrap(),
            );
            self.jack_code = rest.into();

            return Some(ret);
        }

        if first_char == '"' {
            let (s, rest) = self.jack_code.split_once('"').unwrap();

            let ret = Token::StringConst(s.into());
            self.jack_code = rest.into();

            return Some(ret);
        }

        return self
            .jack_code
            .chars()
            .try_fold(first_char.to_string(), |acc, c| {
                if KEYWORDS.contains(&acc.as_str()) {
                    Err(Token::Keyword(acc))
                } else if SYMBOLS.contains(&c) || c.to_digit(10).is_some() || c == '"' {
                    Err(Token::Identifier(acc))
                } else {
                    Ok(acc + c.to_string().as_str())
                }
            })
            .err();
    }
}

impl JackTokenizer {
    /// Opens the input .jack file and gets ready to tokenize it.
    pub fn new(input_file: PathBuf) -> Self {
        Self {
            jack_code: fs::read_to_string(input_file)
                .expect("Should be able to read input file")
                .lines()
                .map(|line| {
                    line.split_once("//")
                        .map(|(line, _)| line)
                        .unwrap_or(line)
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>()
                })
                .collect::<String>(),
        }
    }

    /// Are there more tokens in the input?
    pub fn has_more_tokens(&self) -> bool {
        !self.jack_code.is_empty()
    }
}
