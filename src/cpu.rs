use crate::ai_optimizer::AIOptimizer;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
// use ndarray::Array1;
use rayon::prelude::*;
use statrs::statistics::Statistics;
use serde::{Serialize, Deserialize};
use serde_json;


#[derive(Debug, Serialize, Deserialize)]
pub struct CPU {
    // Basic CPU state
    pub registers: [i32; 4],
    pub memory: Vec<u8>,
    pub pc: usize,
    pub halted: bool,
    pub execution_count: HashMap<usize, usize>,
    pub execution_history: HashMap<usize, usize>,
    pub prediction_history: VecDeque<usize>,
    pub loop_counter: HashMap<(usize, usize), usize>,
    // Enhanced fields for AI-based optimization:
    pub pattern_db: HashMap<Vec<u8>, usize>,      // Patterns and frequencies
    pub value_patterns: HashMap<usize, Vec<i32>>,   // Register value trends
    pub statistical_model: Vec<f64>,                // Probability model for instructions
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
            pattern_db: HashMap::new(),
            value_patterns: HashMap::new(),
            statistical_model: Vec::new(),
        }
    }

    /// Loads a program (binary) into memory.
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }

    /// Loads execution history from a JSON file.
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

    /// Saves execution history to a JSON file.
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

    /// Build a statistical model of the instruction sequence.
    pub fn build_statistical_model(&mut self, program: &[u8]) {
        let mut sequence = Vec::new();
        let mut probabilities = vec![0.0; program.len()];
        for (i, &byte) in program.iter().enumerate() {
            sequence.push(byte);
            if byte == 0xFF || i == program.len() - 1 {
                let prob = self.calculate_sequence_probability(&sequence);
                for j in (i + 1 - sequence.len())..=i {
                    probabilities[j] = prob;
                }
                sequence.clear();
            }
        }
        self.statistical_model = probabilities;
    }

    /// Calculate the probability of an instruction sequence.
    pub fn calculate_sequence_probability(&self, sequence: &[u8]) -> f64 {
        let base_prob = self.execution_history.get(&sequence.len()).cloned().unwrap_or(0) as f64;
        let pattern_prob = self.pattern_db.get(sequence).cloned().unwrap_or(0) as f64;
        (base_prob + pattern_prob) / (self.total_executions() as f64 + 1.0)
    }

    /// Returns the total executions from the history.
    pub fn total_executions(&self) -> usize {
        self.execution_history.values().sum()
    }

    /// Analyze current patterns in execution (parallelized).
    pub fn analyze_patterns(&mut self) {
        let current_state = self.current_execution_context();
        self.pattern_db.par_iter_mut().for_each(|(pattern, count)| {
            if Self::pattern_matches(&current_state, pattern) {
                *count += 1;
            }
        });
    }
    pub fn current_execution_context(&self) -> Vec<u8> {
        self.prediction_history.iter().map(|&pc| pc as u8).collect()
    }
    pub fn pattern_matches(current: &Vec<u8>, pattern: &Vec<u8>) -> bool {
        current == pattern
    }
    pub fn detect_value_redundancies(&mut self) {
        for reg in 0..4 {
            let values = self.value_patterns.entry(reg).or_default();
            values.push(self.registers[reg]);
            if values.len() > 10 {
                let mean = values.iter().map(|&x| x as f64).collect::<Vec<_>>().mean();
                let std_dev = values.iter().map(|&x| x as f64).collect::<Vec<_>>().std_dev();
                if std_dev < 1.0 {
                    self.flag_redundant_register_writes(reg, mean as i32);
                }
            }
        }
    }
    pub fn flag_redundant_register_writes(&self, reg: usize, constant: i32) {
        println!("⚡ AI Optimization: Register R{} is nearly constant at {}", reg, constant);
    }
    pub fn optimize_code(&self, program_path: &str, assembly_lines: &[String]) {
        let optimizations: Vec<String> = assembly_lines
            .par_iter()
            .enumerate()
            .map(|(pc, line)| {
                let probability = self.statistical_model.get(pc).cloned().unwrap_or(0.0);
                if probability < 0.1 && self.is_redundant_instruction(pc) {
                    format!("; AI OPTIMIZED: {}", line)
                } else {
                    line.clone()
                }
            })
            .collect();
        self.write_optimized_code(program_path, &optimizations);
    }
    pub fn is_redundant_instruction(&self, pc: usize) -> bool {
        self.execution_count.get(&pc).cloned().unwrap_or(0) > 5
    }
    pub fn write_optimized_code(&self, program_path: &str, optimized_lines: &[String]) {
        let optimized_file = format!("optimized_{}", program_path);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(optimized_file)
            .expect("❌ Failed to open optimized file for writing!");
        for line in optimized_lines {
            writeln!(file, "{}", line).expect("❌ Failed to write optimized line!");
        }
        println!("💾 AI Optimization: Optimized assembly code saved.");
    }
    pub fn execute(&mut self) {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            panic!("❌ Please provide the program file path as an argument!");
        }
        let program_path = &args[1];

        let mut file = File::open(program_path).expect("❌ Failed to open program file!");
        let mut program = Vec::new();
        file.read_to_end(&mut program).expect("❌ Failed to read program file!");
        self.load_program(&program);

        self.build_statistical_model(&program);

        let mut ai_optimizer = AIOptimizer::new(program.len());
        while !self.halted {
            let pc = self.pc;
            *self.execution_count.entry(pc).or_insert(0) += 1;
            *self.execution_history.entry(pc).or_insert(0) += 1;

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
                0x01 => { 
                    let reg = self.memory[self.pc + 1] as usize;
                    let val = self.memory[self.pc + 2] as i32;
                    if self.modify(pc, reg, val) {
                    }
                    self.pc += 3;
                }
                0x02 => { 
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
                0x90 => { // NOP (Our chosen opcode for no operation)
                    self.pc += 1;
                }
                _ => {
                    println!("❌ Unknown Instruction at {}: {}", self.pc, instruction);
                    self.halted = true;
                }
            }

            // Run AI analyses.
            self.analyze_patterns();
            self.detect_value_redundancies();

            // Feed execution history to the AI optimizer.
            let history: Vec<usize> = self.prediction_history.iter().copied().collect();
            ai_optimizer.analyze_execution_path(&history);
        }

        // Optimize the assembly file (for demonstration, using dummy assembly lines).
        let assembly_lines = vec![
            "MOV R0, 5".to_string(),
            "MOV R1, 10".to_string(),
            "LOOP:".to_string(),
            "ADD R0, R1".to_string(),
            "JMP LOOP".to_string(),
            "HALT".to_string(),
        ];
        self.optimize_code(program_path, &assembly_lines);
        self.save_execution_history();
    }

    /// Dynamically modifies redundant MOV instructions based on historical patterns.
    pub fn modify(&mut self, pc: usize, reg: usize, val: i32) -> bool {
        // If the register already has the same value, mark this MOV as redundant.
        if self.registers[reg] == val {
            println!("🛠 AI Optimization: Redundant MOV at PC {}. Replacing with NOP.", pc);
            // Replace with our NOP opcode (0x90).
            self.memory[pc] = 0x90;
            return true;
        }
        self.registers[reg] = val;
        false
    }
}
