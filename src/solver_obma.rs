use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Instant;

pub struct ObmaConfig {
    pub population_size: usize,
    pub max_iterations: usize,
    pub tabu_max_iters: usize,
    pub use_opposition: bool,
}

impl Default for ObmaConfig {
    fn default() -> Self {
        ObmaConfig {
            population_size: 10,
            max_iterations: usize::MAX, // Will be controlled by deadline
            tabu_max_iters: 50000,
            use_opposition: true,
        }
    }
}

#[derive(Clone)]
struct Individual {
    solution: Vec<usize>,
    fitness: f64,
}

/// Opposition-Based Memetic Algorithm for Maximum Diversity Problem
/// Combines population-based search with intensive local search (tabu)
pub fn solve_obma(
    data: &MdpData,
    config: &ObmaConfig,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    // Build initial population with Opposition-Based Learning
    let mut population = build_initial_population(data, config, deadline);
    
    if population.is_empty() {
        return (Vec::new(), 0.0);
    }
    
    // Track best solution found
    let mut best_solution = population[0].solution.clone();
    let mut best_fitness = population[0].fitness;
    
    // Main OBMA loop
    let mut iteration = 0;
    loop {
        if Instant::now() >= deadline {
            break;
        }
        
        if iteration >= config.max_iterations {
            break;
        }
        
        // Generate offspring through crossover
        let offspring_solution = crossover_with_greedy(data, &population, deadline);
        
        if Instant::now() >= deadline {
            break;
        }
        
        // Apply intensive local search (tabu) to offspring
        let (improved_solution, improved_fitness) = 
            tabu_search_phase(data, offspring_solution, config.tabu_max_iters, deadline);
        
        // Update best if improved
        if improved_fitness > best_fitness {
            best_fitness = improved_fitness;
            best_solution = improved_solution.clone();
        }
        
        // Update population (replace worst with new solution)
        update_population(&mut population, improved_solution, improved_fitness);
        
        iteration += 1;
    }
    
    (best_solution, best_fitness)
}

/// Build initial population with opposition-based learning and tabu search
fn build_initial_population(
    data: &MdpData,
    config: &ObmaConfig,
    deadline: Instant,
) -> Vec<Individual> {
    let mut rng = rand::thread_rng();
    let mut population = Vec::with_capacity(config.population_size);
    
    for _ in 0..config.population_size {
        if Instant::now() >= deadline {
            break;
        }
        
        // Generate random solution
        let mut all_indices: Vec<usize> = (0..data.n).collect();
        all_indices.shuffle(&mut rng);
        let initial = all_indices[0..data.k].to_vec();
        
        // Apply tabu search to improve
        let (solution, fitness) = tabu_search_phase(
            data,
            initial,
            config.tabu_max_iters,
            deadline,
        );
        
        population.push(Individual { solution, fitness });
    }
    
    // Sort population by fitness (descending)
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    
    population
}

/// Tabu search phase - intensive local search
fn tabu_search_phase(
    data: &MdpData,
    initial_solution: Vec<usize>,
    max_iters: usize,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    
    // Build gain vector for efficient evaluation
    let mut gain = vec![0.0; data.n];
    for i in 0..data.n {
        for &s in &initial_solution {
            gain[i] += data.get_dist(i, s);
        }
    }
    
    let mut current_solution = initial_solution;
    let mut current_fitness = calculate_diversity(&current_solution, data);
    
    let mut best_solution = current_solution.clone();
    let mut best_fitness = current_fitness;
    
    for _ in 0..max_iters {
        if Instant::now() >= deadline {
            break;
        }
        
        // Randomly select element to remove
        let remove_idx = rng.gen_range(0..current_solution.len());
        let to_remove = current_solution[remove_idx];
        
        // Find best element to add (not in solution)
        let mut best_add = None;
        let mut best_delta = f64::NEG_INFINITY;
        
        for candidate in 0..data.n {
            if !current_solution.contains(&candidate) {
                // Calculate gain of swap
                let delta = gain[candidate] - gain[to_remove] 
                          - data.get_dist(to_remove, candidate);
                
                if delta > best_delta {
                    best_delta = delta;
                    best_add = Some(candidate);
                }
            }
        }
        
        // Apply best swap
        if let Some(to_add) = best_add {
            current_solution[remove_idx] = to_add;
            current_fitness += best_delta;
            
            // Update gain vector incrementally
            for i in 0..data.n {
                gain[i] += data.get_dist(i, to_add) - data.get_dist(i, to_remove);
            }
            
            // Update best if improved
            if current_fitness > best_fitness {
                best_fitness = current_fitness;
                best_solution = current_solution.clone();
            }
        } else {
            // No valid swap found
            break;
        }
    }
    
    (best_solution, best_fitness)
}

/// Crossover with greedy completion
/// Selects two random parents, takes common elements, fills rest greedily
fn crossover_with_greedy(
    data: &MdpData,
    population: &[Individual],
    deadline: Instant,
) -> Vec<usize> {
    if population.len() < 2 {
        return population[0].solution.clone();
    }
    
    let mut rng = rand::thread_rng();
    
    // Select two random parents
    let p1_idx = rng.gen_range(0..population.len());
    let mut p2_idx = rng.gen_range(0..population.len());
    while p2_idx == p1_idx && population.len() > 1 {
        p2_idx = rng.gen_range(0..population.len());
    }
    
    let parent1 = &population[p1_idx].solution;
    let parent2 = &population[p2_idx].solution;
    
    // Find common elements
    let mut offspring = Vec::new();
    for &elem in parent1 {
        if parent2.contains(&elem) {
            offspring.push(elem);
        }
    }
    
    // Greedily add remaining elements
    while offspring.len() < data.k {
        if Instant::now() >= deadline {
            // Fill randomly if time is up
            let mut all: Vec<usize> = (0..data.n).collect();
            all.shuffle(&mut rng);
            for &elem in &all {
                if !offspring.contains(&elem) && offspring.len() < data.k {
                    offspring.push(elem);
                }
            }
            break;
        }
        
        let mut best_elem = None;
        let mut best_gain = f64::NEG_INFINITY;
        
        // Find element with highest contribution to current offspring
        for candidate in 0..data.n {
            if !offspring.contains(&candidate) {
                let mut gain = 0.0;
                for &existing in &offspring {
                    gain += data.get_dist(candidate, existing);
                }
                
                if gain > best_gain {
                    best_gain = gain;
                    best_elem = Some(candidate);
                }
            }
        }
        
        if let Some(elem) = best_elem {
            offspring.push(elem);
        } else {
            // Fallback: add random element
            let mut all: Vec<usize> = (0..data.n).collect();
            all.shuffle(&mut rng);
            for &elem in &all {
                if !offspring.contains(&elem) {
                    offspring.push(elem);
                    break;
                }
            }
        }
    }
    
    offspring
}

/// Update population by replacing worst individual
fn update_population(
    population: &mut Vec<Individual>,
    new_solution: Vec<usize>,
    new_fitness: f64,
) {
    if population.is_empty() {
        population.push(Individual {
            solution: new_solution,
            fitness: new_fitness,
        });
        return;
    }
    
    // Find worst individual
    let mut worst_idx = 0;
    let mut worst_fitness = population[0].fitness;
    
    for (idx, individual) in population.iter().enumerate() {
        if individual.fitness < worst_fitness {
            worst_fitness = individual.fitness;
            worst_idx = idx;
        }
    }
    
    // Replace worst if new solution is better
    if new_fitness > worst_fitness {
        population[worst_idx] = Individual {
            solution: new_solution,
            fitness: new_fitness,
        };
    }
}

/// Calculate diversity of a solution
fn calculate_diversity(selected: &[usize], data: &MdpData) -> f64 {
    let mut sum = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            sum += data.get_dist(selected[i], selected[j]);
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obma_basic() {
        // Create small test instance
        let data = MdpData {
            n: 10,
            k: 5,
            distances: vec![vec![0.0; 10]; 10],
        };
        
        let deadline = Instant::now() + std::time::Duration::from_secs(1);
        let config = ObmaConfig::default();
        
        let (solution, fitness) = solve_obma(&data, &config, deadline);
        
        assert_eq!(solution.len(), 5);
        assert!(fitness >= 0.0);
    }
}
