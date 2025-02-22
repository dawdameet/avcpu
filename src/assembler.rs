use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

fn main() {
    let input_file = "program.asmcpu";
    let output_file = "program.bin";

    let file = File::open(input_file).expect("Failed to open assembly file");
    let reader = BufReader::new(file);
    
    let mut label_map: HashMap<String, usize> = HashMap::new();
    let mut instructions: Vec<String> = Vec::new();
    let mut binary_output = Vec::new();
    let mut byte_position = 0;

    for line in reader.lines() {
        let mut line = line.unwrap().trim().to_string();
        line = line.replace(",", ""); // ✅ Remove commas

        if line.is_empty() || line.starts_with("#") { continue; }

        if line.ends_with(":") {
            let label = line.trim_end_matches(":").to_string();
            if label_map.contains_key(&label) {
                println!("❌ Error: Duplicate label `{}` found.", label);
                return;
            }
            label_map.insert(label, byte_position);
        } else {
            instructions.push(line.clone());
            byte_position += calculate_byte_size(&line);
        }
    }

    for line in &instructions {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens.get(0) {
            Some(&"MOV") => {
                if tokens.len() != 3 {
                    println!("❌ Syntax error: MOV requires 2 arguments (register, value). Found: {:?}", tokens);
                    continue;
                }
                let reg = tokens[1].trim_start_matches("R").parse::<u8>();
                let val = tokens[2].parse::<u8>();

                if let (Ok(reg), Ok(val)) = (reg, val) {
                    binary_output.extend_from_slice(&[0x01, reg, val]);
                } else {
                    println!("❌ Invalid MOV instruction: {:?}", tokens);
                }
            }
            Some(&"ADD") => {
                if tokens.len() != 3 {
                    println!("❌ Syntax error: ADD requires 2 registers.");
                    continue;
                }
                let reg1 = tokens[1].trim_start_matches("R").parse::<u8>();
                let reg2 = tokens[2].trim_start_matches("R").parse::<u8>();

                if let (Ok(reg1), Ok(reg2)) = (reg1, reg2) {
                    binary_output.extend_from_slice(&[0x02, reg1, reg2]);
                } else {
                    println!("❌ Invalid ADD instruction: {:?}", tokens);
                }
            }
            Some(&"JMP") => {
                if tokens.len() != 2 {
                    println!("❌ Syntax error: JMP requires 1 argument (label). Found: {:?}", tokens);
                    continue;
                }
                if let Some(&address) = label_map.get(tokens[1]) {
                    binary_output.extend_from_slice(&[0x03, address as u8]);
                } else {
                    println!("❌ Error: Undefined label `{}`", tokens[1]);
                    return;
                }
            }
            Some(&"HALT") => {
                if tokens.len() != 1 {
                    println!("❌ Syntax error: HALT should not have arguments.");
                    continue;
                }
                binary_output.push(0xFF);
            }
            Some(_) => {
                println!("❌ Unknown instruction: {}", line);
            }
            None => continue, 
        }
    }
    let mut output = OpenOptions::new().write(true).create(true).truncate(true).open(output_file).unwrap();
    output.write_all(&binary_output).unwrap();

    println!("✅ Compilation complete! Binary saved as '{}'", output_file);
}

fn calculate_byte_size(instruction: &str) -> usize {
    let tokens: Vec<&str> = instruction.split_whitespace().collect();
    match tokens.get(0) {
        Some(&"MOV") => 3,
        Some(&"ADD") => 3,
        Some(&"JMP") => 2,
        Some(&"HALT") => 1,
        _ => 0
    }
}
