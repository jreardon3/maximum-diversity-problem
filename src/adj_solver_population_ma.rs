use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;
use std::time::Instant;

// Configuration based on literature standards for MAMDP
pub struct GeneticConfig {
    pub population_size: usize,
    pub generations: usize,
    pub tabu_tenure: usize,
    pub depth_of_local_search: usize,
}

impl Default for GeneticConfig {
    fn default() -> Self {
        GeneticConfig {
            population_size: 10, // MAMDP typically uses small populations (e.g., 10-20)
            generations: 1000,
            tabu_tenure: 7,
            depth_of_local_search: 50, // Iterations per local search call
        }
    }
}

#[derive(Clone, Debug)]
struct Individual {
    selected: Vec<usize>, // The set M
    fitness: f64,
}

pub fn solve_memetic_mamdp(data: &MdpData, config: &GeneticConfig, deadline: Instant) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();

    // 1. Initialize Population (Random generation)
    let mut population = initialize_population(data, config.population_size, &mut rng);

    // Track best global
    let mut best_global = population
        .iter()
        .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
        .unwrap()
        .clone();

    // Calculate D_max (Maximum distance in the matrix) for Constrained Neighborhood
    // This is done once to avoid O(N^2) scans inside the loop
    let d_max = calculate_d_max(data);

    let mut generation = 0;
    while generation < config.generations {
        if Instant::now() >= deadline {
            break;
        }

        // 2. Parent Selection
        // Algorithm 2: "Randomly select two parent solutions s1 and s2"
        let p1_idx = rng.gen_range(0..population.len());
        let mut p2_idx = rng.gen_range(0..population.len());
        while p1_idx == p2_idx {
            p2_idx = rng.gen_range(0..population.len());
        }
        let parent1 = &population[p1_idx];
        let parent2 = &population[p2_idx];

        // 3. Crossover (Cardinality-constrained uniform crossover)
        // Adjusted to use strict "alternating selection" from distinct sets
        let offspring_genes = crossover(&parent1.selected, &parent2.selected, data.k, &mut rng);
        
        // 4. Local Optimization (Tabu Search with Constrained Neighborhood)
        let (improved_genes, improved_fitness) = tabu_search_constrained(
            &offspring_genes,
            data,
            config.tabu_tenure,
            config.depth_of_local_search,
            d_max,
            &mut rng
        );

        let offspring = Individual {
            selected: improved_genes,
            fitness: improved_fitness,
        };

        // Update Global Best
        if offspring.fitness > best_global.fitness {
            best_global = offspring.clone();
        }

        // 5. Population Update (Quality-and-Distance)
        // Algorithm 2: "Update population according to a quality-diversity rule"
        update_population(&mut population, offspring, data);

        generation += 1;
    }

    (best_global.selected, best_global.fitness)
}

fn initialize_population(data: &MdpData, size: usize, rng: &mut impl Rng) -> Vec<Individual> {
    let mut pop = Vec::with_capacity(size);
    let all_indices: Vec<usize> = (0..data.n).collect();

    for _ in 0..size {
        let mut selected = all_indices.clone();
        selected.shuffle(rng);
        selected.truncate(data.k);
        selected.sort(); // Keep sorted for consistency

        let fitness = calculate_objective(&selected, data);
        pop.push(Individual { selected, fitness });
    }
    pop
}

/// Cardinality-constrained uniform crossover
/// Section 4.1.1: "retains all common elements... remaining elements alternately selected"
/// CORRECTED: Now strictly alternates between Parent 1's distinct elements and Parent 2's.
fn crossover(p1: &[usize], p2: &[usize], k: usize, rng: &mut impl Rng) -> Vec<usize> {
    let set1: HashSet<_> = p1.iter().cloned().collect();
    let set2: HashSet<_> = p2.iter().cloned().collect();

    // 1. Keep common elements M0 = M1 n M2
    let mut offspring: Vec<usize> = set1.intersection(&set2).cloned().collect();

    // 2. Identify distinct elements
    let mut p1_distinct: Vec<usize> = set1.difference(&set2).cloned().collect();
    let mut p2_distinct: Vec<usize> = set2.difference(&set1).cloned().collect();

    // Shuffle distinct sets to ensure random selection order within the specific parent's set
    p1_distinct.shuffle(rng);
    p2_distinct.shuffle(rng);

    // 3. Fill until size k by ALTERNATING selection
    let mut take_from_p1 = true;
    while offspring.len() < k {
        if take_from_p1 {
            if let Some(val) = p1_distinct.pop() {
                offspring.push(val);
            } else if let Some(val) = p2_distinct.pop() {
                // Fallback if P1 runs out
                offspring.push(val);
            }
        } else {
            if let Some(val) = p2_distinct.pop() {
                offspring.push(val);
            } else if let Some(val) = p1_distinct.pop() {
                // Fallback if P2 runs out
                offspring.push(val);
            }
        }
        take_from_p1 = !take_from_p1;
        
        // Safety break if both distinct sets are empty (should not happen if inputs are size k)
        if p1_distinct.is_empty() && p2_distinct.is_empty() {
            break;
        }
    }

    // Fallback if unions didn't have enough size (rare but technically possible if N is very small)
    if offspring.len() < k {
        let current_set: HashSet<usize> = offspring.iter().cloned().collect();
        let mut remaining: Vec<usize> = (0..k * 3) // Check a wider range
            .filter(|x| !current_set.contains(x))
            .collect();
        remaining.shuffle(rng);
        for x in remaining {
            if offspring.len() < k { offspring.push(x); } else { break; }
        }
    }
    
    offspring.sort();
    offspring
}

/// Tabu Search with Constrained Swap Neighborhood and Fast Evaluation
fn tabu_search_constrained(
    start_solution: &[usize],
    data: &MdpData,
    tabu_tenure: usize,
    max_iter: usize,
    d_max: f64,
    rng: &mut impl Rng,
) -> (Vec<usize>, f64) {
    let n = data.n;
    // Current solution state
    let mut current_solution = start_solution.to_vec();
    // Use a boolean vector for O(1) membership check
    let mut in_solution = vec![false; n];
    for &idx in &current_solution {
        in_solution[idx] = true;
    }

    // Initialize Potentials (Delta Table)
    // p[i] = Sum of distances from i to all x in M
    let mut potentials = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0;
        for &j in &current_solution {
            if i != j {
                sum += data.get_dist(i, j);
            }
        }
        potentials[i] = sum;
    }

    let mut current_fitness = calculate_objective(&current_solution, data);
    let mut best_sol = current_solution.clone();
    let mut best_fitness = current_fitness;

    // Tabu list: stores iteration number until which a move is forbidden
    let mut tabu_list = vec![0; n];

    for iter in 0..max_iter {
        // 1. Define Constrained Neighborhood (CN)
        let d_prime_min = current_solution.iter()
            .map(|&i| potentials[i])
            .fold(f64::INFINITY, |a, b| a.min(b));
        
        let d_prime_max = (0..n)
            .filter(|&i| !in_solution[i])
            .map(|i| potentials[i])
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        // Construct subsets X and Y (Formula 3 in text)
        // X = { u in M : p_u <= d'_min + D_max }
        // Y = { v not in M : p_v >= d'_max - D_max }
        let x_set: Vec<usize> = current_solution.iter()
            .cloned()
            .filter(|&u| potentials[u] <= d_prime_min + d_max)
            .collect();
        
        let y_set: Vec<usize> = (0..n)
            .filter(|&v| !in_solution[v] && potentials[v] >= d_prime_max - d_max)
            .collect();

        // 2. Find Best Non-Tabu Move in CN
        let mut best_move = None;
        let mut best_move_gain = f64::NEG_INFINITY;

        for &u in &x_set {
            for &v in &y_set {
                // Formula for gain: Delta = p_v - p_u - d_uv
                let gain = potentials[v] - potentials[u] - data.get_dist(u, v);
                
                let is_tabu = tabu_list[u] > iter || tabu_list[v] > iter;
                // Aspiration criteria: accept tabu if it improves global best
                let aspiration = (current_fitness + gain) > best_fitness;

                if !is_tabu || aspiration {
                    if gain > best_move_gain {
                        best_move_gain = gain;
                        best_move = Some((u, v));
                    }
                }
            }
        }

        // 3. Apply Move
        if let Some((u, v)) = best_move {
            // Remove u
            if let Some(pos) = current_solution.iter().position(|&x| x == u) {
                current_solution.swap_remove(pos);
            }
            in_solution[u] = false;
            // Add v
            current_solution.push(v);
            in_solution[v] = true;

            // Update Fitness
            current_fitness += best_move_gain;

            // Update Global Best
            if current_fitness > best_fitness {
                best_fitness = current_fitness;
                best_sol = current_solution.clone();
            }

            // Update Tabu List
            tabu_list[u] = iter + tabu_tenure; // u cannot come back
            tabu_list[v] = iter + tabu_tenure; // v cannot leave

            // Update Potentials (Fast incremental update)
            // --------------------------------------------------------------------------
            // Mathematical Note:
            // 1. For k != u and k != v: The set M changed by losing u and gaining v.
            //    New potential = Old - d(k,u) + d(k,v).
            // 2. For u (now outside): It was the sum to M. New M is M \ {u} U {v}.
            //    p[u]_new = sum_{z in M, z!=u} d(u,z) + d(u,v) = p[u]_old + d(u,v).
            // 3. For v (now inside): It was sum to M.
            //    p[v]_new = sum_{z in M} d(v,z) - d(v,u) (since u is gone).
            //    p[v]_new = p[v]_old - d(v,u).
            // --------------------------------------------------------------------------
            for k in 0..n {
                if k != u && k != v {
                    potentials[k] = potentials[k] - data.get_dist(k, u) + data.get_dist(k, v);
                }
            }
            potentials[u] = potentials[u] + data.get_dist(u, v);
            potentials[v] = potentials[v] - data.get_dist(v, u);

        } else {
            // No valid move found (stagnation)
            break;
        }
    }
    
    best_sol.sort();
    (best_sol, best_fitness)
}

/// Population Update Strategy: Quality and Diversity
/// "offspring is first inserted... then a quality-and-distance scoring function is used to identify the worst solution... deleted"
fn update_population(population: &mut Vec<Individual>, offspring: Individual, data: &MdpData) {
    // 1. Insert Offspring
    population.push(offspring);

    // 2. Identify Worst (Score = Fitness + Beta * Diversity)
    // Pre-calculate distances between all pairs
    let mut dists = vec![vec![0.0; population.len()]; population.len()];
    for i in 0..population.len() {
        for j in (i+1)..population.len() {
            let d = hamming_distance(&population[i].selected, &population[j].selected);
            dists[i][j] = d;
            dists[j][i] = d;
        }
    }

    // Normalize Fitness to combine with Diversity
    let max_fit = population.iter().map(|ind| ind.fitness).fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let min_fit = population.iter().map(|ind| ind.fitness).fold(f64::INFINITY, |a, b| a.min(b));
    
    let mut best_score = f64::INFINITY;
    let mut worst_idx = 0;

    for i in 0..population.len() {
        // Diversity: Average distance to all other individuals
        let mut sum_dist = 0.0;
        for j in 0..population.len() {
            if i != j { sum_dist += dists[i][j]; }
        }
        let avg_dist = sum_dist / (population.len() as f64 - 1.0);

        // Normalized fitness (0.0 to 1.0)
        let norm_fit = if max_fit - min_fit > 1e-6 { 
            (population[i].fitness - min_fit) / (max_fit - min_fit) 
        } else { 
            1.0 
        };
        
        // Score = Quality + Diversity. We want to maximize this property.
        // Therefore, the individual with the LOWEST score is the "worst" one to delete.
        let score = norm_fit + avg_dist; 

        if score < best_score {
            best_score = score;
            worst_idx = i;
        }
    }

    // 3. Delete Worst
    population.swap_remove(worst_idx);
}

fn calculate_objective(selected: &[usize], data: &MdpData) -> f64 {
    let mut sum = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            sum += data.get_dist(selected[i], selected[j]);
        }
    }
    sum
}

fn calculate_d_max(data: &MdpData) -> f64 {
    let mut max_d = 0.0;
    // Scanning the full matrix might be slow if N is huge, but necessary for D_max.
    // Done once per problem solve.
    for i in 0..data.n {
        for j in (i+1)..data.n {
            let d = data.get_dist(i, j);
            if d > max_d {
                max_d = d;
            }
        }
    }
    max_d
}

// Simple set difference size
fn hamming_distance(s1: &[usize], s2: &[usize]) -> f64 {
    let set1: HashSet<_> = s1.iter().collect();
    let set2: HashSet<_> = s2.iter().collect();
    let intersection = set1.intersection(&set2).count();
    // Hamming distance for fixed size sets is 2 * (k - intersection)
    // We can just return k - intersection (number of differing elements)
    (s1.len() - intersection) as f64
}