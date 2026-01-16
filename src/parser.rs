use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct MdpData {
    pub n: usize,
    pub k: usize,
    pub distances: Vec<f64>, // Flattened 1D vector for better memory performance
}

impl MdpData {
    pub fn load(path: &str) -> Self {
        let file = File::open(path).expect("Could not open MDPLIB file");
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Parse first line: n k
        let first_line = lines.next().unwrap().unwrap();
        let header: Vec<usize> = first_line.split_whitespace()
            .map(|s| s.parse().unwrap()).collect();
        
        let n = header[0];
        let k = header[1];
        let mut distances = vec![0.0; n * n];

        // Parse distance lines: i j dist
        for line in lines {
            let l = line.unwrap();
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() < 3 { continue; }
            
            let i: usize = parts[0].parse().unwrap();
            let j: usize = parts[1].parse().unwrap();
            let d: f64 = parts[2].parse().unwrap();

            // Fill symmetrically
            distances[i * n + j] = d;
            distances[j * n + i] = d;
        }

        MdpData { n, k, distances }
    }

    pub fn get_dist(&self, i: usize, j: usize) -> f64 {
        self.distances[i * self.n + j]
    }
}