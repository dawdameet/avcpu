mod cpu;
use cpu::CPU;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("❌ Usage: cargo run -- <source.asm>");
        return;
    }

    let source_file = &args[1];
    let binary_file = "program.bin";

    println!("📜 Assembling {}...", source_file);
    let assembler_output = Command::new("cargo")
        .args(&["run", "--bin", "assembler"])
        .output()
        .expect("❌ Failed to run assembler");

    if !assembler_output.status.success() {
        println!("❌ Assembler Error:\n{}", String::from_utf8_lossy(&assembler_output.stderr));
        return;
    }

    println!("⚙️ Running AI-powered CPU Emulator...");
    let mut file = File::open(binary_file).expect("❌ Failed to open binary file");
    let mut binary_data = Vec::new();
    file.read_to_end(&mut binary_data).expect("❌ Failed to read binary file");
    let mut cpu = CPU::new();
    cpu.load_execution_history();
    cpu.load_program(&binary_data);
    cpu.execute();
    cpu.save_execution_history();

    println!("✅ Execution complete!");
    println!("🏁 Final CPU State: {:?}", cpu);
}
