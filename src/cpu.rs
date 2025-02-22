use std::fs::File;
use std::io::{Read, Write};
use serde_json;
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
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

    pub fn load_execution_history(&mut self) {
        if let Ok(mut file) = File::open("execution_data.json") {
            let mut json_data = String::new();
            if file.read_to_string(&mut json_data).is_ok() {
                if let Ok(prev_data) = serde_json::from_str::<HashMap<usize, usize>>(&json_data) {
                    println!("📊 AI Loaded Execution Data from Previous Runs!");
                    for (pc, count) in prev_data {
                        *self.execution_history.entry(pc).or_insert(0) += count; // ✅ Merge data
                    }
                }
            }
        } else {
            println!("⚠️ No Previous Execution Data Found. AI Starting Fresh.");
        }
    }
    

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }

    pub fn save_execution_history(&self) {
        println!("📋 Execution History Before Saving: {:?}", self.execution_history);
    
        match serde_json::to_string_pretty(&self.execution_history) {
            Ok(json) => {
                let mut file = match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open("execution_data.json")
                {
                    Ok(file) => file,
                    Err(e) => {
                        println!("❌ Error opening file: {}", e);
                        return;
                    }
                };
    
                if let Err(e) = file.write_all(json.as_bytes()) {
                    println!("❌ Error writing to file: {}", e);
                } else {
                    println!("📁 AI Execution Data Successfully Saved!");
                }
            }
            Err(e) => println!("❌ Error serializing execution data: {}", e),
        }
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
    
            // Update both execution_count and execution_history
            *self.execution_count.entry(pc).or_insert(0) += 1;
            *self.execution_history.entry(pc).or_insert(0) += 1;
    
            if self.prediction_history.len() >= 2 {
                let last = *self.prediction_history.back().unwrap();
                let jump_pair = (last, pc);
    
                let entry = self.loop_counter.entry(jump_pair).or_insert(0);
                *entry += 1;
    
                if *entry > 5 {
                    println!(
                        "⛔ AI Detected Infinite Loop: Repeated jumps between {} and {}. Halting execution.",
                        last, pc
                    );
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
                0x01 => {
                    let reg = self.memory[self.pc + 1] as usize;
                    let val = self.memory[self.pc + 2] as i32;
                    self.registers[reg] = val;
                    self.pc += 3;
                }
                0x02 => {
                    let reg1 = self.memory[self.pc + 1] as usize;
                    let reg2 = self.memory[self.pc + 2] as usize;
                    self.registers[reg1] += self.registers[reg2];
                    self.pc += 3;
                }
                0x03 => {
                    let addr = self.memory[self.pc + 1] as usize;
                    self.pc = addr;
                }
                0xFF => {
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
