#![allow(dead_code)]

use ndarray::Array2;
// use statrs::statistics::{MeanN, VarianceN};
use std::collections::HashMap;

pub struct AIOptimizer {
    pub pattern_matrix: Array2<f64>,
    pub execution_graph: HashMap<usize, Vec<usize>>,
}

impl AIOptimizer {
    pub fn new(program_size: usize) -> Self {
        AIOptimizer {
            pattern_matrix: Array2::zeros((program_size, program_size)),
            execution_graph: HashMap::new(),
        }
    }

    pub fn analyze_execution_path(&mut self, history: &[usize]) {
        for window in history.windows(2) {
            let (from, to) = (window[0], window[1]);
            self.pattern_matrix[[from, to]] += 1.0;
            self.execution_graph.entry(from).or_default().push(to);
        }
    }

    pub fn find_redundant_blocks(&self, threshold: f64) -> Vec<usize> {
        self.pattern_matrix
            .rows()
            .into_iter()
            .enumerate()
            .filter(|(_, row)| row.mean().unwrap() < threshold)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn suggest_optimizations(&self, program: &[String]) -> Vec<String> {
        program
            .iter()
            .enumerate()
            .map(|(pc, line)| {
                let connectivity = self
                    .execution_graph
                    .get(&pc)
                    .map(|v| v.len() as f64)
                    .unwrap_or(0.0);
                
                if connectivity < 2.0 && self.pattern_matrix.row(pc).sum() < 1.0 {
                    format!("; AI SUGGESTION: Remove redundant instruction\n; {}", line)
                } else {
                    line.clone()
                }
            })
            .collect()
    }
}
