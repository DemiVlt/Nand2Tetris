mod code_writer;
mod parser;

use code_writer::CodeWriter;
use parser::{CommandType, Parser};
use std::{
    fs::{self},
    path::PathBuf,
};

struct VMTranslator {
    parser: Parser,
    code_writer: CodeWriter,
    files: Vec<PathBuf>,
}

impl VMTranslator {
    fn new(input_file: PathBuf) -> Self {
        let mut output_file = input_file.clone();

        let mut files = if input_file.is_dir() {
            output_file.push(
                PathBuf::from(input_file.file_name().unwrap().to_str().unwrap())
                    .with_extension("asm"),
            );

            fs::read_dir(&input_file)
                .expect("Should be able to read directory contents")
                .filter_map(|entry| {
                    let entry = entry.unwrap().path();

                    (entry.with_extension("vm") == entry).then_some(entry)
                })
                .collect()
        } else {
            output_file.set_extension("asm");

            vec![input_file]
        };

        let file = files
            .iter()
            .position(|i| *i == i.with_file_name("Sys.vm"))
            .map(|index| files.remove(index))
            .unwrap_or_else(|| files.pop().expect("Expected a .vm file."));

        let mut code_writer = CodeWriter::new(output_file);
        code_writer.set_file_name(file.clone());

        Self {
            parser: Parser::new(file),
            code_writer,
            files,
        }
    }

    fn translate(&mut self) {
        let Self {
            parser,
            code_writer,
            files,
        } = self;
        code_writer.bootstrap();

        loop {
            while parser.has_more_lines() {
                parser.advance();

                match parser.command_type() {
                    CommandType::ARITHMETIC => code_writer.write_arithmetic(parser.arg1()),
                    CommandType::LABEL => code_writer.write_label(parser.arg1()),
                    CommandType::GOTO => code_writer.write_goto(parser.arg1()),
                    CommandType::IF => code_writer.write_if(parser.arg1()),
                    CommandType::FUNCTION => {
                        code_writer.write_function(parser.arg1(), parser.arg2())
                    }
                    CommandType::RETURN => code_writer.write_return(),
                    CommandType::CALL => code_writer.write_call(parser.arg1(), parser.arg2()),
                    pop_push => code_writer.write_push_pop(pop_push, parser.arg1(), parser.arg2()),
                }
            }

            let Some(file_name) = files.pop() else {
                break;
            };

            code_writer.set_file_name(file_name.clone());
            *parser = Parser::new(file_name);
        }

        code_writer.close()
    }
}

fn main() {
    let input = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("An input file should be specified."),
    );

    let mut vm_translator = VMTranslator::new(input);

    vm_translator.translate();
}
