use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Instant;
use std::collections::HashSet;

/// Configuration for the Opposition-Based Memetic Algorithm
pub struct ObmaConfig {
    pub population_size: usize,
    pub max_iterations: usize,
    pub tabu_tenure: usize,
    pub tabu_max_iters: usize,
    pub opposition_mining: bool,
}

impl Default for ObmaConfig {
    fn default() -> Self {
        ObmaConfig {
            population_size: 10,   // Small population is standard for Memetic Algos
            max_iterations: usize::MAX, 
            tabu_tenure: 15,       // Tenure for the Attribute-based memory
            tabu_max_iters: 2000,  // Short, intensive local search bursts
            opposition_mining: true,
        }
    }
}

#[derive(Clone, Debug)]
struct Individual {
    solution: Vec<usize>,
    fitness: f64,
}

/// Opposition-Based Memetic Algorithm (OBMA)
/// Solves Maximum Diversity Problem using Opposition-Based Learning and Tabu Search
pub fn solve_obma(
    data: &MdpData,
    config: &ObmaConfig,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    // 1. Initialization with Opposition-Based Learning
    let mut population = initialize_population(data, config, deadline);
    
    if population.is_empty() {
        return (Vec::new(), 0.0);
    }

    // Track Global Best
    let mut best_solution = population[0].solution.clone();
    let mut best_fitness = population[0].fitness;

    let mut iter = 0;
    while iter < config.max_iterations {
        if Instant::now() >= deadline {
            break;
        }

        // 2. Crossover (Greedy Crossover to preserve high-quality common blocks)
        let offspring = crossover(data, &population, deadline);
        
        // 3. Opposition Calculation (The "Jumping" Step)
        // Generates a solution from the complementary region of the search space
        let opposite = if config.opposition_mining {
            calculate_opposite(&offspring, data)
        } else {
            random_solution(data)
        };

        // 4. Double Trajectory Local Search
        // Apply intensive Tabu Search to BOTH the offspring and its opposite
        let (imp_offspring, fit_off) = tabu_search(data, offspring, config.tabu_max_iters, config.tabu_tenure, deadline);
        let (imp_opposite, fit_opp) = tabu_search(data, opposite, config.tabu_max_iters, config.tabu_tenure, deadline);

        // Update Global Best
        if fit_off > best_fitness {
            best_fitness = fit_off;
            best_solution = imp_offspring.clone();
        }
        if fit_opp > best_fitness {
            best_fitness = fit_opp;
            best_solution = imp_opposite.clone();
        }

        // 5. Population Update using RBQD (Rank-Based Quality and Distance)
        // Maintains diversity by considering both fitness and distance to other individuals
        update_population_rbqd(&mut population, imp_offspring, fit_off, data);
        update_population_rbqd(&mut population, imp_opposite, fit_opp, data);

        iter += 1;
    }

    (best_solution, best_fitness)
}

// ==============================================================================
// 1. OPPOSITION-BASED LEARNING
// ==============================================================================

/// Generates the "Opposite" solution.
/// For MDP, the opposite of a set S is the set of k items in (V \ S) 
/// that are "most different" (furthest) from S.
fn calculate_opposite(solution: &[usize], data: &MdpData) -> Vec<usize> {
    let sol_set: HashSet<usize> = solution.iter().cloned().collect();
    
    // Identify universe V \ S (Complementary Set)
    let universe: Vec<usize> = (0..data.n)
        .filter(|x| !sol_set.contains(x))
        .collect();

    // Calculate "distance" of each unselected element to the current solution set.
    // We select elements that are furthest from the current solution.
    let mut candidates: Vec<(usize, f64)> = universe.iter().map(|&u| {
        let dist_to_set: f64 = solution.iter().map(|&s| data.get_dist(u, s)).sum();
        (u, dist_to_set)
    }).collect();

    // Sort by distance descending
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Take top k
    candidates.iter().take(data.k).map(|(idx, _)| *idx).collect()
}

// ==============================================================================
// 2. TABU SEARCH (Fixed: Attribute-Based)
// ==============================================================================

fn tabu_search(
    data: &MdpData,
    mut solution: Vec<usize>,
    max_iters: usize,
    tabu_tenure: usize,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    let mut current_fitness = calculate_diversity(&solution, data);
    let mut best_sol = solution.clone();
    let mut best_fitness = current_fitness;

    // Attribute-Based Tabu List (Fixed Size Array)
    // Stores the iteration number until which a node is "tabu"
    let mut tabu_list = vec![0; data.n]; 
    
    // Fast lookup for current solution membership
    let mut in_solution = vec![false; data.n];
    for &x in &solution { in_solution[x] = true; }

    for iter in 0..max_iters {
        if iter % 100 == 0 && Instant::now() >= deadline { break; }

        let mut best_move = None;
        let mut best_move_gain = f64::NEG_INFINITY;

        let unselected: Vec<usize> = (0..data.n).filter(|&x| !in_solution[x]).collect();

        // 1. Evaluate Neighborhood (1-swap)
        for (i, &u) in solution.iter().enumerate() { // u is element to REMOVE
            for &v in &unselected { // v is element to ADD
                
                let gain = calculate_swap_gain(u, v, &solution, data);
                
                // TABU CHECK (Attribute-Based):
                // A move is tabu if 'v' (added) was recently dropped 
                // OR 'u' (removed) was recently added.
                let is_tabu = tabu_list[u] > iter || tabu_list[v] > iter;

                // ASPIRATION CRITERION:
                // Override tabu status if this move finds a new global best
                let aspiration = (current_fitness + gain) > best_fitness;

                if !is_tabu || aspiration {
                    if gain > best_move_gain {
                        best_move_gain = gain;
                        best_move = Some((i, u, v));
                    }
                }
            }
        }

        // 2. Perform Move
        if let Some((idx_in_sol, u, v)) = best_move {
            // Execute swap
            solution[idx_in_sol] = v;
            in_solution[u] = false;
            in_solution[v] = true;
            
            current_fitness += best_move_gain;
            
            // Update Tabu List (Attribute-Based)
            // Forbid adding 'u' back for 'tenure' iterations
            tabu_list[u] = iter + tabu_tenure;
            // Forbid removing 'v' for 'tenure' iterations
            tabu_list[v] = iter + tabu_tenure;

            // Update Best Found
            if current_fitness > best_fitness {
                best_fitness = current_fitness;
                best_sol = solution.clone();
            }
        } else {
            // No valid move found (fully tabu locked without aspiration)
            break;
        }
    }

    (best_sol, best_fitness)
}

// ==============================================================================
// 3. RBQD POPULATION UPDATE
// ==============================================================================

fn update_population_rbqd(
    population: &mut Vec<Individual>, 
    candidate_sol: Vec<usize>, 
    candidate_fit: f64,
    data: &MdpData
) {
    // Check for duplicates (fitness based shortcut)
    if population.iter().any(|ind| (ind.fitness - candidate_fit).abs() < 1e-6) {
        return; 
    }

    // Create temp population (Current + Candidate)
    let mut temp_pop = population.clone();
    temp_pop.push(Individual { solution: candidate_sol, fitness: candidate_fit });

    // Rank 1: Quality (Fitness) - Descending (0 is best)
    let mut sorted_by_fit: Vec<usize> = (0..temp_pop.len()).collect();
    sorted_by_fit.sort_by(|&a, &b| temp_pop[b].fitness.partial_cmp(&temp_pop[a].fitness).unwrap());
    
    let mut quality_rank = vec![0; temp_pop.len()];
    for (rank, &idx) in sorted_by_fit.iter().enumerate() {
        quality_rank[idx] = rank;
    }

    // Rank 2: Distance (Diversity) - Descending (0 is most distant/best)
    let mut distances = vec![0.0; temp_pop.len()];
    for i in 0..temp_pop.len() {
        let mut min_dist = f64::MAX;
        for j in 0..temp_pop.len() {
            if i == j { continue; }
            let d = hamming_distance(&temp_pop[i].solution, &temp_pop[j].solution);
            if d < min_dist { min_dist = d; }
        }
        distances[i] = min_dist;
    }

    let mut sorted_by_dist: Vec<usize> = (0..temp_pop.len()).collect();
    sorted_by_dist.sort_by(|&a, &b| distances[b].partial_cmp(&distances[a]).unwrap());
    
    let mut dist_rank = vec![0; temp_pop.len()];
    for (rank, &idx) in sorted_by_dist.iter().enumerate() {
        dist_rank[idx] = rank;
    }

    // Combined Score: Sum of ranks (Lower is better)
    // Identify the individual with the WORST (Highest) score
    let mut worst_idx = 0;
    let mut max_score = -1;

    for i in 0..temp_pop.len() {
        let score = quality_rank[i] + dist_rank[i];
        if score as i32 > max_score {
            max_score = score as i32;
            worst_idx = i;
        }
    }

    // Remove the worst individual to maintain population size
    temp_pop.swap_remove(worst_idx);
    *population = temp_pop;
}

fn hamming_distance(sol1: &[usize], sol2: &[usize]) -> f64 {
    let set1: HashSet<usize> = sol1.iter().cloned().collect();
    let mut diff = 0;
    for x in sol2 {
        if !set1.contains(x) {
            diff += 1;
        }
    }
    diff as f64
}

// ==============================================================================
// HELPERS (Init, Crossover, Math)
// ==============================================================================

fn initialize_population(data: &MdpData, config: &ObmaConfig, deadline: Instant) -> Vec<Individual> {
    let mut population = Vec::new();

    for _ in 0..config.population_size {
        if Instant::now() >= deadline { break; }

        // 1. Random Init
        let rand_sol = random_solution(data);
        
        // 2. Opposition Init
        let opp_sol = calculate_opposite(&rand_sol, data);

        // 3. Improve both with SHORT Tabu (Initialization phase uses reduced iterations)
        let init_tabu_iters = config.tabu_max_iters / 5;
        let (imp_rand, fit_rand) = tabu_search(data, rand_sol, init_tabu_iters, config.tabu_tenure, deadline);
        let (imp_opp, fit_opp) = tabu_search(data, opp_sol, init_tabu_iters, config.tabu_tenure, deadline);

        // 4. Selection (OBL Rule): Pick the fitter of the pair
        if fit_rand > fit_opp {
            population.push(Individual { solution: imp_rand, fitness: fit_rand });
        } else {
            population.push(Individual { solution: imp_opp, fitness: fit_opp });
        }
    }
    
    // Initial Sort
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    population
}

fn crossover(data: &MdpData, population: &[Individual], _deadline: Instant) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    
    // Tournament Selection
    let p1 = &population[rng.gen_range(0..population.len())];
    let p2 = &population[rng.gen_range(0..population.len())];

    // Greedy Crossover: 
    // 1. Inherit common elements (Backbone)
    let set1: HashSet<usize> = p1.solution.iter().cloned().collect();
    let mut offspring: Vec<usize> = p2.solution.iter().filter(|x| set1.contains(x)).cloned().collect();

    // 2. Fill remaining slots greedily
    let mut candidates: Vec<usize> = (0..data.n)
        .filter(|x| !offspring.contains(x))
        .collect();

    while offspring.len() < data.k {
        let mut best_cand = 0;
        let mut best_gain = f64::NEG_INFINITY;
        
        // Use sampling for speed on very large instances
        let sample_size = if candidates.len() > 100 { 100 } else { candidates.len() };
        candidates.shuffle(&mut rng);

        for &cand in candidates.iter().take(sample_size) {
            // Calculate contribution to current partial solution
            let gain: f64 = offspring.iter().map(|&s| data.get_dist(cand, s)).sum();
            if gain > best_gain {
                best_gain = gain;
                best_cand = cand;
            }
        }
        
        offspring.push(best_cand);
        candidates.retain(|&x| x != best_cand);
    }

    offspring
}

fn random_solution(data: &MdpData) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut all: Vec<usize> = (0..data.n).collect();
    all.shuffle(&mut rng);
    all[0..data.k].to_vec()
}

fn calculate_diversity(selected: &[usize], data: &MdpData) -> f64 {
    let mut sum = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            sum += data.get_dist(selected[i], selected[j]);
        }
    }
    sum
}

fn calculate_swap_gain(old: usize, new: usize, selected: &[usize], data: &MdpData) -> f64 {
    let mut gain = 0.0;
    for &s in selected {
        if s == old { continue; }
        gain += data.get_dist(new, s) - data.get_dist(old, s);
    }
    // Note: The distance between 'old' and 'new' is not part of the objective function
    // because they never exist in the solution at the same time.
    gain
}