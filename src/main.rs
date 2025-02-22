mod ai_optimizer;
mod cpu;
mod assembler;
use cpu::CPU;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("❌ Usage: cargo run --bin avcpu <program.bin>");
        return;
    }
    let program_path = &args[1];

    let mut file = File::open(program_path).expect("❌ Failed to open binary file");
    let mut binary_data = Vec::new();
    file.read_to_end(&mut binary_data).expect("❌ Failed to read binary file");

    let mut cpu = CPU::new();
    cpu.load_execution_history();
    cpu.load_program(&binary_data);
    cpu.execute();
}
