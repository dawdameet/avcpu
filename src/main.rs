mod cpu;
use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();

    // Load AI Execution Data
    cpu.load_execution_history();

    let program: [u8; 18] = [
        0x01, 0x00, 5,   
        0x01, 0x00, 5,   // Redundant MOV (AI should remove after learning)
        0x01, 0x01, 10,  
        0x02, 0x00, 0x01, 
        0x02, 0x00, 0x01, // Repeated ADD (AI should merge after learning)
        0x03, 0x09,      // Useless JMP (AI should remove)
        0xFF            
    ];

    println!("🔧 Optimizing Program...");
    let optimized_program = cpu.optimize_program(&program);
    println!("🚀 Running Optimized Program...");

    cpu.load_program(&optimized_program);
    cpu.execute();

    // Save AI Execution Data
    cpu.save_execution_history();

    println!("Final CPU State: {:?}", cpu);
}
