use crate::CommandType;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub struct CodeWriter {
    output_file: Option<File>,
    file_name: String,
    continue_count: usize,
}

impl CodeWriter {
    pub fn new(output_file: PathBuf) -> Self {
        let output_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)
            .expect("Should be able to open output file.");

        let code_writer = Self {
            output_file: Some(output_file),
            file_name: String::new(),
            continue_count: 0,
        };

        code_writer
    }

    pub fn set_file_name(&mut self, file_name: PathBuf) {
        self.file_name = file_name.file_stem().unwrap().to_str().unwrap().to_owned();
    }

    pub fn bootstrap(&mut self) {
        self.write(
            "@256
            D=A
            @SP
            M=D"
            .into(),
        );

        if self.file_name == "Sys" {
            self.write_call("Sys.init", 0);
        }
    }

    pub fn write_arithmetic(&mut self, command: &str) {
        let continue_count = &mut self.continue_count;

        let replace_top = match command {
            "add" => String::from("M=D+M"),
            "sub" => String::from("M=M-D"),
            "and" => String::from("M=D&M"),
            "or" => String::from("M=D|M"),
            "neg" => {
                return self.write(String::from(
                    "@SP 
                    A=M-1
                    M=-M",
                ));
            }
            "not" => {
                return self.write(String::from(
                    "@SP 
                    A=M-1
                    M=!M",
                ));
            }
            cmp => {
                *continue_count += 1;

                let jump = match cmp {
                    "lt" => "D;JLT",
                    "eq" => "D;JEQ",
                    "gt" => "D;JGT",
                    _ => {
                        *continue_count -= 1;
                        return;
                    }
                };

                format!(
                    "D=M-D
                    @REPLACE_TOP_WITH_TRUE{continue_count}
                    {jump}
                    @REPLACE_TOP_WITH_FALSE{continue_count}
                    0;JMP
                    (CONTINUE{continue_count})",
                )
            }
        };

        let translated_command = format!(
            "@SP 
            AM=M-1
            D=M
            @SP 
            A=M-1
            {replace_top}",
        );

        self.write(translated_command);
    }

    pub fn write_push_pop(&mut self, command: CommandType, segment: &str, mut index: i32) {
        let is_pop = command == CommandType::POP;

        let segment = match segment {
            "local" => "LCL",
            "argument" => "ARG",
            "this" => "THIS",
            "that" => "THAT",
            "static" => "static",
            "constant" => "constant",
            "temp" => {
                index += 5;
                "n/a"
            }
            "pointer" => {
                index += 3;
                "n/a"
            }
            _ => panic!("invalid segment"),
        };

        let translated_command = format!(
            "{}\n{}\n{}\n{}",
            if segment == "static" {
                format!("@{}.{index}", self.file_name)
            } else {
                format!("@{index}")
            },
            if is_pop || !["static", "n/a"].contains(&segment) {
                "D=A"
            } else {
                "D=M"
            },
            &if ["n/a", "static", "constant"].contains(&segment) {
                String::new()
            } else if is_pop {
                format!(
                    "@{segment}
                    D=D+M"
                )
            } else {
                format!(
                    "@{segment}
                    A=D+M
                    D=M"
                )
            },
            if is_pop {
                "@13
                M=D
                @SP
                AM=M-1
                D=M
                @13
                A=M
                M=D"
            } else {
                "@SP
                A=M
                M=D
                @SP
                M=M+1"
            }
        );

        self.write(translated_command);
    }

    pub fn close(&mut self) {
        let end = (1..=self.continue_count).fold(
            String::from(
                "@END 
                0;JMP 
                (END) 
                @END 
                0;JMP",
            ),
            |acc, continue_count: usize| {
                acc + &format!(
                    "\n(REPLACE_TOP_WITH_TRUE{continue_count})
                    @SP 
                    A=M-1
                    M=-1
                    @CONTINUE{continue_count}
                    0;JMP
                    (REPLACE_TOP_WITH_FALSE{continue_count})
                    @SP 
                    A=M-1
                    M=0
                    @CONTINUE{continue_count}
                    0;JMP"
                )
            },
        );

        self.write(end);
        let _ = self.output_file.take();
    }

    pub fn write_label(&mut self, label: &str) {
        let translated_command = format!("(__{}__)\n", label);

        self.write(translated_command);
    }

    pub fn write_goto(&mut self, label: &str) {
        let translated_command = format!(
            "@__{}__\n\
            0;JMP\n",
            label
        );

        self.write(translated_command);
    }

    pub fn write_if(&mut self, label: &str) {
        let translated_command = format!(
            "@SP
            AM=M-1
            D=M
            @__{}__
            D;JNE\n",
            label
        );

        self.write(translated_command);
    }

    pub fn write_function(&mut self, function_name: &str, n_vars: i32) {
        let translated_command = format!(
            "(__{}__)
            @0
            D=A
            {}",
            function_name,
            "@SP
            A=M
            M=D
            @SP
            M=M+1\n"
                .repeat(n_vars as usize),
        );

        self.write(translated_command);
    }

    pub fn write_call(&mut self, function_name: &str, n_args: i32) {
        self.continue_count += 1;

        let translated_command = format!(
            "@CONTINUE{}
            D=A 
            @SP
            A=M
            M=D
            @SP
            M=M+1

            @LCL
            D=M 
            @SP
            A=M
            M=D
            @SP
            M=M+1

            @ARG
            D=M 
            @SP
            A=M
            M=D
            @SP
            M=M+1

            @THIS
            D=M 
            @SP
            A=M
            M=D
            @SP
            M=M+1

            @THAT
            D=M 
            @SP
            A=M
            M=D
            @SP
            M=M+1

            D=M
            @LCL
            M=D

            @5
            D=D-A
            @{n_args}
            D=D-A
            @ARG 
            M=D
            
            @__{function_name}__
            0;JMP
            (CONTINUE{})\n",
            self.continue_count, self.continue_count
        );

        self.write(translated_command);
    }

    pub fn write_return(&mut self) {
        let translated_command = format!(
            "@LCL
            D=M
            @15
            M=D 

            @5
            A=D-A 
            D=M
            @14
            M=D

            @SP 
            AM=M-1
            D=M
            @ARG
            A=M
            M=D

            @ARG
            D=M
            @SP
            M=D+1

            @15
            AM=M-1
            D=M
            @THAT
            M=D

            @15
            AM=M-1
            D=M
            @THIS
            M=D

            @15
            AM=M-1
            D=M
            @ARG
            M=D

            @15
            AM=M-1
            D=M
            @LCL
            M=D

            @14
            A=M
            0;JMP"
        );

        self.write(translated_command);
    }

    fn write(&mut self, s: String) {
        self.output_file
            .as_ref()
            .expect("Output file should be opened")
            .write_all(
                s.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .fold(String::new(), |acc, line| acc + line + "\n")
                    .as_bytes(),
            )
            .expect("Should be able to write to the output file.");
    }
}
