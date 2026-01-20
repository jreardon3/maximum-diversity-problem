use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct GeneticConfig {
    pub population_size: usize,
    pub generations: usize,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
    pub elite_size: usize,  // Number of best individuals to carry over
}

impl Default for GeneticConfig {
    fn default() -> Self {
        GeneticConfig {
            population_size: 50,
            generations: 100,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
            elite_size: 5,
        }
    }
}

#[derive(Clone)]
struct Individual {
    selected: Vec<usize>,
    fitness: f64,
}

pub fn solve_genetic(data: &MdpData, config: &GeneticConfig) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    
    // Initialize population
    let mut population = initialize_population(data, config.population_size);
    evaluate_population(&mut population, data);
    
    for generation in 0..config.generations {
        // Sort by fitness (descending)
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        
        // Elitism: keep best individuals
        let mut new_population = population[0..config.elite_size].to_vec();
        
        // Generate rest of population through crossover and mutation
        while new_population.len() < config.population_size {
            // Selection: tournament selection
            let parent1 = tournament_selection(&population, 3, &mut rng);
            let parent2 = tournament_selection(&population, 3, &mut rng);
            
            // Crossover
            let mut offspring = if rng.gen::<f64>() < config.crossover_rate {
                crossover(&parent1.selected, &parent2.selected, data, &mut rng)
            } else {
                parent1.selected.clone()
            };
            
            // Mutation
            if rng.gen::<f64>() < config.mutation_rate {
                mutate(&mut offspring, data, &mut rng);
            }
            
            let fitness = calculate_diversity(&offspring, data);
            new_population.push(Individual {
                selected: offspring,
                fitness,
            });
        }
        
        population = new_population;
        
        // Optional: local search on best individual every N generations
        if generation % 10 == 0 {
            let best = &population[0];
            let (improved, fitness) = local_improvement(&best.selected, data);
            if fitness > best.fitness {
                population[0] = Individual {
                    selected: improved,
                    fitness,
                };
            }
        }
    }
    
    // Return best individual
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    let best = &population[0];
    (best.selected.clone(), best.fitness)
}

fn initialize_population(data: &MdpData, size: usize) -> Vec<Individual> {
    let mut rng = rand::thread_rng();
    let mut population = Vec::with_capacity(size);
    
    for _ in 0..size {
        let mut all_indices: Vec<usize> = (0..data.n).collect();
        all_indices.shuffle(&mut rng);
        let selected = all_indices[0..data.k].to_vec();
        
        population.push(Individual {
            selected,
            fitness: 0.0,
        });
    }
    
    population
}

fn evaluate_population(population: &mut [Individual], data: &MdpData) {
    for individual in population.iter_mut() {
        individual.fitness = calculate_diversity(&individual.selected, data);
    }
}

fn tournament_selection<'a>(
    population: &'a [Individual],
    tournament_size: usize,
    rng: &mut impl Rng,
) -> &'a Individual {
    let mut best = &population[rng.gen_range(0..population.len())];
    
    for _ in 1..tournament_size {
        let competitor = &population[rng.gen_range(0..population.len())];
        if competitor.fitness > best.fitness {
            best = competitor;
        }
    }
    
    best
}

fn crossover(
    parent1: &[usize],
    parent2: &[usize],
    data: &MdpData,
    rng: &mut impl Rng,
) -> Vec<usize> {
    // Path crossover: take elements from parent1, fill rest with parent2
    let crossover_point = rng.gen_range(1..data.k);
    
    let mut offspring = parent1[0..crossover_point].to_vec();
    
    // Add elements from parent2 that aren't already in offspring
    for &elem in parent2 {
        if !offspring.contains(&elem) && offspring.len() < data.k {
            offspring.push(elem);
        }
    }
    
    // If still not enough elements, add random ones
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(rng);
    
    for &elem in &all_indices {
        if offspring.len() >= data.k {
            break;
        }
        if !offspring.contains(&elem) {
            offspring.push(elem);
        }
    }
    
    offspring
}

fn mutate(solution: &mut Vec<usize>, data: &MdpData, rng: &mut impl Rng) {
    // Swap mutation: remove one element, add a different one
    let remove_idx = rng.gen_range(0..solution.len());
    let removed = solution.remove(remove_idx);
    
    // Find an element not in solution
    let mut candidates: Vec<usize> = (0..data.n)
        .filter(|&i| !solution.contains(&i))
        .collect();
    
    if candidates.is_empty() {
        solution.push(removed);
        return;
    }
    
    candidates.shuffle(rng);
    solution.push(candidates[0]);
}

fn local_improvement(solution: &[usize], data: &MdpData) -> (Vec<usize>, f64) {
    let mut selected = solution.to_vec();
    let mut unselected: Vec<usize> = (0..data.n)
        .filter(|&i| !selected.contains(&i))
        .collect();
    
    let mut current_diversity = calculate_diversity(&selected, data);
    let mut improved = true;
    
    while improved {
        improved = false;
        let mut best_gain = 0.0;
        let mut best_swap = None;
        
        for i in 0..selected.len() {
            for j in 0..unselected.len() {
                let gain = calculate_swap_gain(selected[i], unselected[j], &selected, data);
                if gain > best_gain {
                    best_gain = gain;
                    best_swap = Some((i, j));
                }
            }
        }
        
        if let Some((i, j)) = best_swap {
            let temp = selected[i];
            selected[i] = unselected[j];
            unselected[j] = temp;
            current_diversity += best_gain;
            improved = true;
        }
    }
    
    (selected, current_diversity)
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
        if s == old {
            continue;
        }
        gain += data.get_dist(new, s) - data.get_dist(old, s);
    }
    gain
}