use std::{fs, path::PathBuf};

#[derive(PartialEq, Debug)]
pub enum CommandType {
    ARITHMETIC,
    PUSH,
    POP,
    LABEL,
    GOTO,
    IF,
    FUNCTION,
    RETURN,
    CALL,
}

pub struct Parser {
    vm_code: String,
    current_command: String,
}

impl Parser {
    pub fn new(input_file: PathBuf) -> Self {
        Self {
            vm_code: fs::read_to_string(input_file)
                .expect("Should be able to read input file")
                .lines()
                .map(|line| {
                    line.split_once("//")
                        .map(|(line, _)| line)
                        .unwrap_or(line)
                        .trim()
                })
                .filter(|line| !line.is_empty())
                .fold(String::new(), |acc, line| acc + line + "\n")
                .trim()
                .to_owned(),
            current_command: String::new(),
        }
    }

    pub fn has_more_lines(&self) -> bool {
        !self.vm_code.is_empty()
    }

    pub fn advance(&mut self) {
        let mut lines = self.vm_code.lines();

        self.current_command = lines
            .next()
            .expect("This method should be called only if hasMoreLines is true.")
            .to_owned();

        self.vm_code = lines
            .fold(String::new(), |acc, line| acc + line + "\n")
            .trim()
            .to_owned();
    }

    pub fn command_type(&self) -> CommandType {
        match self
            .current_command
            .split_whitespace()
            .next()
            .expect("Command expected.")
        {
            "pop" => CommandType::POP,
            "push" => CommandType::PUSH,
            "add" | "sub" | "and" | "or" | "neg" | "not" | "lt" | "eq" | "gt" => {
                CommandType::ARITHMETIC
            }
            "label" => CommandType::LABEL,
            "goto" => CommandType::GOTO,
            "if-goto" => CommandType::IF,
            "function" => CommandType::FUNCTION,
            "return" => CommandType::RETURN,
            "call" => CommandType::CALL,
            _ => panic!("Unknown command!"),
        }
    }

    pub fn arg1(&self) -> &str {
        let mut args = self.current_command.split_whitespace();

        match self.command_type() {
            CommandType::ARITHMETIC => args.nth(0).expect("Arithmetic command expected."),
            CommandType::RETURN => {
                panic!("Should not be called if the current command is C_RETURN.")
            }
            _ => args.nth(1).expect("Should have an argument."),
        }
    }

    pub fn arg2(&self) -> i32 {
        let mut args = self.current_command.split_whitespace();

        match self.command_type() {
            CommandType::POP |
            CommandType::PUSH |
            CommandType::FUNCTION |
            CommandType::CALL => args.nth(2).expect("Should have a second argument.").parse().expect("Second argument should be a number."),
            _ => panic!("Should only be called if the current command is C_PUSH, C_POP, C_FUNCTION, or C_CALL)"),
        }
    }
}
