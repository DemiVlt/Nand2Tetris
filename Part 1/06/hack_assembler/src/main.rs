const PREDEFINED: [(&str, u16); 23] = [
    ("R0", 0),
    ("R1", 1),
    ("R2", 2),
    ("R3", 3),
    ("R4", 4),
    ("R5", 5),
    ("R6", 6),
    ("R7", 7),
    ("R8", 8),
    ("R9", 9),
    ("R10", 10),
    ("R11", 11),
    ("R12", 12),
    ("R13", 13),
    ("R14", 14),
    ("R15", 15),
    ("SCREEN", 16384),
    ("KBD", 24576),
    ("SP", 0),
    ("LCL", 1),
    ("ARG", 2),
    ("THIS", 3),
    ("THAT", 4),
];

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .expect("please specify an input file");
    let output_file = format!(
        "{}.hack",
        input_file
            .strip_suffix(".asm")
            .expect("input is not a .asm file")
    );

    let assembly = std::fs::read_to_string(input_file).expect("failed to find input file");

    let mut comment_started = false;
    let mut symbols = Vec::new();
    let mut label_count = 0;

    let lines: Vec<(usize, String)> = assembly
        .lines()
        .map(|line| {
            remove_block_comments(
                line.split_once("//").map(|(line, _)| line).unwrap_or(line),
                &mut comment_started,
            )
            .trim()
            .to_owned()
        })
        .enumerate()
        .filter(|(i, line)| {
            line.strip_prefix('(')
                .and_then(|line| line.strip_suffix(')'))
                .map(|label| {
                    symbols.push((label.to_owned(), *i as u16 - label_count));
                    label_count += 1;
                })
                .is_none()
                && line.is_empty().then(|| label_count += 1).is_none()
        })
        .collect();

    let mut new_var = 15;

    let binary = lines.iter().fold(String::new(), |binary, (_, line)| {
        binary + &format!("{:016b}\n", binary_instr(line, &mut symbols, &mut new_var))
    });
    std::fs::write(output_file, binary).expect("failed to write to output file");
}

fn binary_instr(line: &str, symbols: &mut Vec<(String, u16)>, new_var: &mut u16) -> u16 {
    // A or C instr.
    if let Some(line) = line.strip_prefix('@') {
        return line.parse::<u16>().unwrap_or_else(|_| {
            let value = PREDEFINED
                .into_iter()
                .chain(symbols.iter().map(|(from, to)| (from.as_str(), *to)))
                .find(|(from, _)| line == *from)
                .map(|(_, to)| to);

            value.unwrap_or_else(|| {
                *new_var += 1;
                symbols.push((line.to_owned(), *new_var));
                *new_var
            })
        });
    }
    let mut binary_instr: u16 = 7 << 13;

    // dest
    let after_dest = line
        .split_once('=')
        .map(|(x, y)| {
            assert!(!x.is_empty(), "'=' without destination");
            for dest in x.chars() {
                binary_instr |= match dest {
                    'A' => 1 << 5,
                    'D' => 1 << 4,
                    'M' => 1 << 3,
                    _ => panic!("invalid destination"),
                }
            }
            y
        })
        .unwrap_or(line);

    // jump
    let eval = after_dest
        .split_once(";J")
        .map(|(x, y)| {
            binary_instr |= match y {
                "GT" => 1,
                "EQ" => 2,
                "GE" => 3,
                "LT" => 4,
                "NE" => 5,
                "LE" => 6,
                "MP" => 7,
                _ => panic!("invalid jump"),
            };
            x
        })
        .unwrap_or(after_dest);

    // "a"
    if eval.contains("M") {
        binary_instr |= 1 << 12;
    }
    // comp
    binary_instr |= match eval {
        "0" => 0b101010 << 6,
        "1" => 0b111111 << 6,
        "-1" => 0b111010 << 6,
        "D" => 0b001100 << 6,
        "A" | "M" => 0b110000 << 6,
        "!D" => 0b001101 << 6,
        "!A" | "!M" => 0b110001 << 6,
        "-D" => 0b001111 << 6,
        "-A" | "-M" => 0b110011 << 6,
        "D+1" => 0b011111 << 6,
        "A+1" | "M+1" => 0b110111 << 6,
        "D-1" => 0b001110 << 6,
        "A-1" | "M-1" => 0b110010 << 6,
        "D+A" | "D+M" => 0b000010 << 6,
        "D-A" | "D-M" => 0b010011 << 6,
        "A-D" | "M-D" => 0b000111 << 6,
        "D&A" | "D&M" => 0b000000 << 6,
        "D|A" | "D|M" => 0b010101 << 6,
        _ => panic!("invalid evaluation"),
    };

    binary_instr
}

fn remove_block_comments(line: &str, comment_started: &mut bool) -> String {
    let mut new = String::new();
    if !*comment_started {
        new += line
            .split_once("/*")
            .map(|(line, _)| {
                *comment_started = true;
                line
            })
            .unwrap_or(line);
    }
    new += line
        .split_once("*/")
        .map(|(_, line)| {
            *comment_started = false;
            line
        })
        .unwrap_or("");

    if new.contains('*') {
        remove_block_comments(&new, comment_started)
    } else {
        new
    }
}
