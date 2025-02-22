use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

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
            execution_count: HashMap::new(),
            prediction_history: VecDeque::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }

    pub fn optimize_program(&self, program: &[u8]) -> Vec<u8> {
        let mut optimized_program = Vec::new();
        let mut last_instruction: Option<(u8, usize, usize)> = None;
        let mut i = 0;

        while i < program.len() {
            let opcode = program[i];

            match opcode {
                0x01 => { // MOV Rn, value
                    let reg = program[i + 1] as usize;
                    let val = program[i + 2] as usize;

                    if let Some((last_op, last_reg, last_val)) = last_instruction {
                        if last_op == 0x01 && last_reg == reg && last_val == val {
                            println!("⚡ Optimized: Removed redundant MOV R{}, {}", reg, val);
                            i += 3;
                            continue;
                        }
                    }

                    optimized_program.extend_from_slice(&[opcode, program[i + 1], program[i + 2]]);
                    last_instruction = Some((opcode, reg, val));
                    i += 3;
                }
                0x02 => { // ADD Rn, Rm
                    let reg1 = program[i + 1] as usize;
                    let reg2 = program[i + 2] as usize;

                    if let Some((last_op, last_reg1, last_reg2)) = last_instruction {
                        if last_op == 0x02 && last_reg1 == reg1 && last_reg2 == reg2 {
                            println!("⚡ Optimized: Merging repeated ADD R{}, R{}", reg1, reg2);
                            i += 3;
                            continue;
                        }
                    }

                    optimized_program.extend_from_slice(&[opcode, program[i + 1], program[i + 2]]);
                    last_instruction = Some((opcode, reg1, reg2));
                    i += 3;
                }
                0x03 => { // JMP Address
                    let addr = program[i + 1] as usize;

                    if addr == i + 2 {
                        println!("⚡ Optimized: Removed useless JMP to next instruction.");
                        i += 2;
                        continue;
                    }

                    optimized_program.extend_from_slice(&[opcode, program[i + 1]]);
                    last_instruction = None;
                    i += 2;
                }
                _ => {
                    optimized_program.push(opcode);
                    last_instruction = None;
                    i += 1;
                }
            }
        }

        optimized_program
    }

    pub fn execute(&mut self) {
        let mut loop_count = 0;
        let mut last_pc = None;

        while !self.halted {
            println!("✅ Execution Count: {:?}", self.execution_count);

            if let Some(predicted_pc) = self.predict_next() {
                if predicted_pc != self.pc {
                    println!("🔮 AI Prediction: Jumping to {}", predicted_pc);
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
                println!("⛔ AI detected a persistent infinite loop. Halting execution.");
                self.halted = true;
                break;
            }

            match opcode {
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
                0xFF => {
                    println!("🛑 Program Halted.");
                    self.halted = true;
                }
                _ => panic!("Unknown opcode: {:#X} at PC: {}", opcode, self.pc),
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
