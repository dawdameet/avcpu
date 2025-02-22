use std::fs::File;
use std::io::{Read, Write};
use serde_json;
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CPU {
    registers: [i32; 4],
    memory: Vec<u8>,
    pc: usize,
    halted: bool,
    execution_count: HashMap<usize, usize>, 
    execution_history: HashMap<usize, usize>,
    prediction_history: VecDeque<usize>,
    loop_counter: HashMap<(usize, usize), usize>, 
}


impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: [0; 4],
            memory: vec![0; 256],
            pc: 0,
            halted: false,
            execution_count: HashMap::new(),
            execution_history: HashMap::new(), 
            prediction_history: VecDeque::new(),
            loop_counter: HashMap::new(),
        }
    }
    pub fn save_execution_history(&self) {
        let json = serde_json::to_string_pretty(&self.execution_history).unwrap();
        let mut file = File::create("execution_data.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
        println!("📁 AI Execution Data Saved!");
    }
    pub fn load_execution_history(&mut self) {
        if let Ok(mut file) = File::open("execution_data.json") {
            let mut json_data = String::new();
            file.read_to_string(&mut json_data).unwrap();
            self.execution_history = serde_json::from_str(&json_data).unwrap();
            println!("📊 AI Loaded Execution Data from Previous Runs!");
        } else {
            println!("⚠️ No Previous Execution Data Found. AI Starting Fresh.");
        }
    }
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }

    pub fn optimize_program(&mut self, program: &[u8]) -> Vec<u8> {
        let mut optimized_program = Vec::new();
        let mut i = 0;
    
        while i < program.len() {
            let opcode = program[i];
    
            match opcode {
                0x01 => { 
                    let reg = program[i + 1] as usize;
                    let val = program[i + 2] as usize;
                    if let Some(count) = self.execution_history.get(&i) {
                        if *count > 5 {
                            println!("⚡ AI Optimization: MOV R{}, {} is frequently executed. Skipping redundant operations.", reg, val);
                            i += 3;
                            continue;
                        }
                    }
                    optimized_program.extend_from_slice(&[opcode, program[i + 1], program[i + 2]]);
                    i += 3;
                }
                0x02 => { 
                    let reg1 = program[i + 1] as usize;
                    let reg2 = program[i + 2] as usize;
    
                    if let Some(count) = self.execution_history.get(&i) {
                        if *count > 3 {
                            println!("⚡ AI Optimization: ADD R{}, R{} executed too frequently. Merging redundant operations.", reg1, reg2);
                            i += 3;
                            continue;
                        }
                    }
    
                    optimized_program.extend_from_slice(&[opcode, program[i + 1], program[i + 2]]);
                    i += 3;
                }
                _ => {
                    optimized_program.push(opcode);
                    i += 1;
                }
            }
        }
    
        optimized_program
    }
    


        pub fn execute(&mut self) {
            while !self.halted {
                let pc = self.pc;
    
                if let Some(count) = self.execution_count.get_mut(&pc) {
                    *count += 1;
                } else {
                    self.execution_count.insert(pc, 1);
                }
                if self.prediction_history.len() >= 2 {
                    let last = *self.prediction_history.back().unwrap();
                    let jump_pair = (last, pc);
    
                    let entry = self.loop_counter.entry(jump_pair).or_insert(0);
                    *entry += 1;
    
                    if *entry > 5 { 
                        println!("⛔ AI Detected Infinite Loop: Repeated jumps between {} and {}. Halting execution.", last, pc);
                        self.halted = true;
                        break;
                    }
                }
    
                self.prediction_history.push_back(pc);
                if self.prediction_history.len() > 10 {
                    self.prediction_history.pop_front(); 
                }
    
                let instruction = self.memory[self.pc];
    
                match instruction {
                    0x01 => { // MOV Rn, value
                        let reg = self.memory[self.pc + 1] as usize;
                        let val = self.memory[self.pc + 2] as i32;
                        self.registers[reg] = val;
                        self.pc += 3;
                    }
                    0x02 => { // ADD Rn, Rm
                        let reg1 = self.memory[self.pc + 1] as usize;
                        let reg2 = self.memory[self.pc + 2] as usize;
                        self.registers[reg1] += self.registers[reg2];
                        self.pc += 3;
                    }
                    0x03 => { // JMP Address
                        let addr = self.memory[self.pc + 1] as usize;
                        self.pc = addr;
                    }
                    0xFF => { // HALT
                        println!("🚀 Program Halted Normally.");
                        self.halted = true;
                    }
                    _ => {
                        println!("❌ Unknown Instruction at {}: {}", self.pc, instruction);
                        self.halted = true;
                    }
                }
            }
        }
    
    

    fn predict_next(&mut self) -> Option<usize> {
        let mut best_pc = None;
        let mut max_count = 0;

        for (&pc, &count) in &self.execution_count {
            if count > max_count && pc > self.pc && pc != self.pc {
                best_pc = Some(pc);
                max_count = count;
            }
        }

        if let Some(predicted_pc) = best_pc {
            self.prediction_history.push_back(predicted_pc);
            if self.prediction_history.len() > 5 {
                self.prediction_history.pop_front();
            }

            let unique: std::collections::HashSet<_> = self.prediction_history.iter().collect();
            if unique.len() == 2 {
                println!("⚠️ AI detected a loop between {} and {}. Skipping prediction.", 
                         self.prediction_history[0], self.prediction_history[1]);
                return None;
            }

            println!("🔮 AI Prediction: Jumping to {}", predicted_pc);
        }

        best_pc
    }
}
