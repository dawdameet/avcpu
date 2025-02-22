#![allow(dead_code)]
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;
use rayon::prelude::*;

fn main() {
    let input_file = "program.asmcpu";
    let output_file = "program.bin";

    let file = File::open(input_file).expect("❌ Failed to open assembly file");
    let reader = BufReader::new(file);

    let mut label_map: HashMap<String, usize> = HashMap::new();
    let mut instructions: Vec<String> = Vec::new();
    let mut binary_output = Vec::new();
    let mut byte_position = 0;

    // Pass 1: Collect Labels
    for line in reader.lines() {
        let mut line = line.unwrap();
        if let Some(idx) = line.find('#') {
            line.truncate(idx);
        }
        line = line.replace(",", "").trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.ends_with(":") {
            let label = line.trim_end_matches(":").to_string();
            if label_map.contains_key(&label) {
                eprintln!("❌ Error: Duplicate label `{}` found.", label);
                return;
            }
            label_map.insert(label, byte_position);
        } else {
            instructions.push(line.clone());
            byte_position += calculate_byte_size(&line);
        }
    }

    // AI-assisted assembly optimization: remove redundant moves/adds
    let optimized_instructions = optimize_assembly(&instructions, &label_map);

    // Pass 2: Assemble Instructions
    for line in &optimized_instructions {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens.as_slice() {
            ["MOV", reg, val] => {
                if let (Some(r), Some(v)) = (parse_register(reg), parse_value(val)) {
                    binary_output.extend_from_slice(&[0x01, r, v]);
                } else {
                    eprintln!("❌ Invalid MOV instruction: {:?}", tokens);
                }
            }
            ["ADD", reg1, reg2] => {
                if let (Some(r1), Some(r2)) = (parse_register(reg1), parse_register(reg2)) {
                    binary_output.extend_from_slice(&[0x02, r1, r2]);
                } else {
                    eprintln!("❌ Invalid ADD instruction: {:?}", tokens);
                }
            }
            ["JMP", label] => {
                if let Some(&address) = label_map.get(*label) {
                    binary_output.extend_from_slice(&[0x03, address as u8]);
                } else {
                    eprintln!("❌ Error: Undefined label `{}`", label);
                    return;
                }
            }
            ["HALT"] => binary_output.push(0xFF),
            _ => eprintln!("❌ Unknown instruction: {}", line),
        }
    }

    // Write binary output to file.
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .unwrap();
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
        _ => 0,
    }
}

fn parse_register(token: &str) -> Option<u8> {
    token.strip_prefix("R").and_then(|r| r.parse::<u8>().ok()).filter(|&r| r <= 3)
}

fn parse_value(token: &str) -> Option<u8> {
    token.parse::<u8>().ok()
}

// AI-assisted assembly optimization functions.

fn optimize_assembly(instructions: &[String], labels: &HashMap<String, usize>) -> Vec<String> {
    instructions
        .par_iter()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens.as_slice() {
                ["MOV", reg, val] if is_redundant_move(reg, val, instructions, labels) => None,
                ["ADD", reg1, reg2] if is_redundant_add(reg1, reg2, instructions) => None,
                _ => Some(line.clone()),
            }
        })
        .collect()
}

fn is_redundant_move(reg: &str, val: &str, instructions: &[String], _labels: &HashMap<String, usize>) -> bool {
    let count = instructions
        .iter()
        .filter(|line| line.starts_with("MOV") && line.contains(reg) && line.contains(val))
        .count();
    count > 1
}

fn is_redundant_add(reg1: &str, reg2: &str, instructions: &[String]) -> bool {
    let count = instructions
        .iter()
        .filter(|line| line.starts_with("ADD") && line.contains(reg1) && line.contains(reg2))
        .count();
    count > 2
}
