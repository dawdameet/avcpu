#![allow(unused_imports)]
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use serde_json;

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
            file.read_to_string(&mut json_data).unwrap();
            self.execution_history = serde_json::from_str(&json_data).unwrap_or_else(|_| {
                println!("⚠️ Warning: Failed to parse execution history. Starting fresh.");
                HashMap::new()
            });
            println!("📊 AI Loaded Execution Data from Previous Runs!");
        } else {
            println!("⚠️ No Previous Execution Data Found. AI Starting Fresh.");
        }
    }

    pub fn save_execution_history(&self) {
        match serde_json::to_string_pretty(&self.execution_history) {
            Ok(json) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open("execution_data.json")
                    .unwrap();

                if file.write_all(json.as_bytes()).is_ok() {
                    println!("📁 AI Execution Data Successfully Saved!");
                } else {
                    println!("❌ Error writing execution history.");
                }
            }
            Err(e) => println!("❌ Error serializing execution data: {}", e),
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }
    

    pub fn execute(&mut self) {
        while !self.halted {
            let pc = self.pc;

            // Update execution tracking
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
                0x03 => { // JMP address
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
}
