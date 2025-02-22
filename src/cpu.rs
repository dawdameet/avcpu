use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize)]
pub struct CPU {
    registers: [i32; 4],
    memory: Vec<u8>,
    pc: usize,
    halted: bool,
    execution_count: HashMap<usize, usize>,
    prediction_history: VecDeque<usize>, 
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: [0; 4],
            memory: vec![0; 256],
            pc: 0,
            halted: false,
            prediction_history: VecDeque::new(),
            execution_count: HashMap::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
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
            if self.prediction_history.len() == 5 {
                let unique: std::collections::HashSet<_> = self.prediction_history.iter().collect();
                if unique.len() == 2 {
                    println!("AI detected a loop between {} and {}. Skipping prediction.", 
                             self.prediction_history[0], self.prediction_history[1]);
                    return None;
                }
            }
    
            println!("AI Prediction: Jumping to {}", predicted_pc);
        }
    
        best_pc
    }
    
    

    pub fn execute(&mut self) {
        let mut loop_count = 0;  
        let mut last_pc = None;  
    
        while !self.halted {
            println!("Execution Count: {:?}", self.execution_count);
    
            if let Some(predicted_pc) = self.predict_next() {
                if predicted_pc != self.pc {
                    println!("AI Prediction: Jumping to {}", predicted_pc);
                    self.pc = predicted_pc;
                }
            }
    
            let opcode = self.memory[self.pc];
            *self.execution_count.entry(self.pc).or_insert(0) += 1;
            if last_pc == Some(self.pc) {
                loop_count += 1;
            } else {
                loop_count = 0;
            }
            last_pc = Some(self.pc);
            if loop_count > 5 {
                println!("AI detected a persistent infinite loop. Halting execution.");
                self.halted = true;
                break;
            }
    
            match opcode {
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
                    println!("🛑 Program Halted.");
                    self.halted = true;
                }
                _ => panic!("Unknown opcode: {:#X} at PC: {}", opcode, self.pc),
            }
        }
    }
    
    
}
