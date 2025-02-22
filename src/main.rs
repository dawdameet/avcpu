mod cpu;
use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();

    let program: [u8; 21] = [
        0x01, 0x00, 5,   
        0x01, 0x00, 5,   // Redundant 
        0x01, 0x01, 10,  
        0x02, 0x00, 0x01, 
        0x02, 0x00, 0x01, // Repeated 
        0x02, 0x00, 0x01, // Repeated 
        0x03, 0x09,
        0xFF            
    ];

    println!("🔧 Optimizing Program...");
    let optimized_program = cpu.optimize_program(&program);
    println!("🚀 Running Optimized Program...");
    
    cpu.load_program(&optimized_program);
    cpu.execute();

    println!("Final CPU State: {:?}", cpu);
}
