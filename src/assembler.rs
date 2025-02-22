use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let input_file = "program.asmcpu";
    let output_file = "program.bin";
    
    let file = File::open(input_file).expect("Failed to open assembly file");
    let reader = BufReader::new(file);
    
    let mut output = File::create(output_file).expect("Failed to create binary file");
    
    for line in reader.lines() {
        let line = line.unwrap();
        
        // Remove commas and split by whitespace
        let line = line.replace(',', " ");
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        match tokens[0] {
            "MOV" => {
                if tokens.len() != 3 {
                    println!("❌ Syntax error: MOV requires 2 arguments (register, value). Found: {:?}", tokens);
                    continue;
                }
                let reg = tokens[1].strip_prefix("R").unwrap_or("").parse::<u8>();
                let val = tokens[2].parse::<u8>();
        
                if let (Ok(reg), Ok(val)) = (reg, val) {
                    output.write_all(&[0x01, reg, val]).unwrap();
                } else {
                    println!("❌ Invalid MOV instruction: {:?}", tokens);
                }
            }
            "ADD" => {
                if tokens.len() != 3 {
                    println!("❌ Syntax error: ADD requires 2 registers.");
                    continue;
                }
                let reg1 = tokens[1].strip_prefix("R").unwrap_or("").parse::<u8>();
                let reg2 = tokens[2].strip_prefix("R").unwrap_or("").parse::<u8>();
        
                if let (Ok(reg1), Ok(reg2)) = (reg1, reg2) {
                    output.write_all(&[0x02, reg1, reg2]).unwrap();
                } else {
                    println!("❌ Invalid ADD instruction: {:?}", tokens);
                }
            }
            "HALT" => {
                if tokens.len() != 1 {
                    println!("❌ Syntax error: HALT should not have arguments.");
                    continue;
                }
                output.write_all(&[0xFF]).unwrap();
            }
            _ => {
                println!("❌ Unknown instruction: {}", line);
            }
        }
    }
    
    println!("✅ Compilation complete! Binary saved as '{}'", output_file);
}