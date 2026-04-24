use std::{fs, path::PathBuf, str::SplitWhitespace};

fn main() {
    let input = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("please specify an input file"),
    );

    let mut output_file = input.clone();

    let mut continue_count = 0;

    let translated = if input.is_dir() {
        output_file.push(
            PathBuf::from(input.file_name().unwrap().to_str().unwrap()).with_extension("asm"),
        );

        fs::read_dir(input)
            .expect("failed to read directory contents")
            .filter_map(|entry| {
                let entry = entry.unwrap().path();

                (entry == entry.with_extension("vm"))
                    .then(|| translate_file(entry, &mut continue_count))
            })
            .collect()
    } else {
        output_file.set_extension("asm");

        translate_file(input, &mut continue_count)
    };

    let bootstrap = format!(
        "@256
        D=A 
        @SP 
        M=D
        {}",
        translated
            .contains("__Sys.init__")
            .then_some(translate_line(
                "call Sys.init 0".split_whitespace(),
                &mut continue_count,
                "Sys",
            ))
            .unwrap_or_default(),
    );

    let end = format!(
        "@END 
        0;JMP 
        (END) 
        @END 
        0;JMP
        {}",
        (1..=continue_count).fold(String::new(), |acc, continue_count: usize| {
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
        })
    );

    let asm = format!("{bootstrap}\n{translated}\n{end}")
        .lines()
        .filter_map(|mut line| {
            line = line.trim();
            (!line.is_empty()).then_some(line.to_owned() + "\n")
        })
        .collect::<String>();

    fs::write(output_file, asm).expect("failed to write to output file");
}

fn translate_line(
    mut args: SplitWhitespace,
    continue_count: &mut usize,
    file_name: &str,
) -> String {
    match args.next().unwrap_or("") {
        command if ["pop", "push"].contains(&command) => {
            let is_pop = command == "pop";
            let segment = args.next().expect("invalid segment");
            let mut index = args
                .next()
                .map_or(0, |arg| arg.parse::<i32>().expect("invalid segment offset"));

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

            format!(
                "{}\n{}\n{}\n{}",
                if segment == "static" {
                    format!("@{}.{index}", file_name)
                } else {
                    format!("@{index}")
                },
                if is_pop || !["static", "n/a"].contains(&segment) {
                    "D=A"
                } else {
                    "D=M"
                },
                (!["n/a", "static", "constant"].contains(&segment))
                    .then_some(if is_pop {
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
                    })
                    .unwrap_or_default(),
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
            )
        }

        "neg" => String::from(
            "@SP 
            A=M-1
            M=-M",
        ),

        "not" => String::from(
            "@SP 
            A=M-1
            M=!M",
        ),

        "label" => format!("(__{}__)", args.next().unwrap()),

        "goto" => format!(
            "@__{}__
            0;JMP",
            args.next().unwrap()
        ),

        "if-goto" => format!(
            "@SP
            AM=M-1
            D=M
            @__{}__
            D;JNE",
            args.next().unwrap()
        ),

        "function" => format!(
            "(__{}__)
            @0
            D=A
            {}",
            args.next().unwrap(),
            "@SP
            A=M
            M=D
            @SP
            M=M+1\n"
                .repeat(args.next().and_then(|x| x.parse().ok()).unwrap()),
        ),

        "return" => format!(
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
        ),

        "call" => {
            let fn_name = args.next().unwrap();
            let n_args = args.next().unwrap();

            *continue_count += 1;

            format!(
                "@CONTINUE{continue_count}
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
                
                @__{fn_name}__
                0;JMP
                (CONTINUE{continue_count})"
            )
        }

        command => {
            let replace_top = match command {
                "add" => String::from("M=D+M"),
                "sub" => String::from("M=M-D"),
                "and" => String::from("M=D&M"),
                "or" => String::from("M=D|M"),
                cmp => {
                    *continue_count += 1;

                    let jump = match cmp {
                        "lt" => "D;JLT",
                        "eq" => "D;JEQ",
                        "gt" => "D;JGT",
                        _ => {
                            *continue_count -= 1;
                            return String::new();
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

            format!(
                "@SP 
                AM=M-1
                D=M
                @SP 
                A=M-1
                {replace_top}",
            )
        }
    }
}

fn translate_file(path: PathBuf, continue_count: &mut usize) -> String {
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

    fs::read_to_string(path)
        .expect("failed to read input file")
        .lines()
        .fold(String::new(), |acc, line| {
            acc + &translate_line(
                line.split_once("//")
                    .map(|(line, _)| line)
                    .unwrap_or(line)
                    .split_whitespace(),
                continue_count,
                &file_name,
            ) + "\n"
        })
}
