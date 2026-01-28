use crate::parser::MdpData;
use rand::seq::SliceRandom;

pub struct LocalSearchConfig {
    pub method: LocalSearchMethod,
    pub max_iters: usize,
}

pub enum LocalSearchMethod {
    FirstImprovement,
    BestImprovement,
    TabuSearch { tabu_tenure: usize },
}

impl Default for LocalSearchConfig {
    fn default() -> Self {
        LocalSearchConfig {
            method: LocalSearchMethod::BestImprovement,
            max_iters: 5000,
        }
    }
}

pub fn solve_local_search(data: &MdpData, config: &LocalSearchConfig) -> (Vec<usize>, f64) {
    match &config.method {
        LocalSearchMethod::FirstImprovement => first_improvement_search(data, config.max_iters),
        LocalSearchMethod::BestImprovement => best_improvement_search(data, config.max_iters),
        LocalSearchMethod::TabuSearch { tabu_tenure } => tabu_search(data, config.max_iters, *tabu_tenure),
    }
}

// ============ First Improvement (like your original) ============
fn first_improvement_search(data: &MdpData, max_iters: usize) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);
    
    let mut selected: Vec<usize> = all_indices[0..data.k].to_vec();
    let mut unselected: Vec<usize> = all_indices[data.k..].to_vec();
    let mut current_diversity = calculate_diversity(&selected, data);
    
    for _ in 0..max_iters {
        let mut improved = false;
        
        'outer: for i in 0..selected.len() {
            for j in 0..unselected.len() {
                let gain = calculate_swap_gain(selected[i], unselected[j], &selected, data);
                
                if gain > 1e-9 { // Small epsilon for floating point
                    let temp = selected[i];
                    selected[i] = unselected[j];
                    unselected[j] = temp;
                    current_diversity += gain;
                    improved = true;
                    break 'outer;
                }
            }
        }
        
        if !improved {
            break;
        }
    }
    
    (selected, current_diversity)
}

// ============ Best Improvement ============
fn best_improvement_search(data: &MdpData, max_iters: usize) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);
    
    let mut selected: Vec<usize> = all_indices[0..data.k].to_vec();
    let mut unselected: Vec<usize> = all_indices[data.k..].to_vec();
    let mut current_diversity = calculate_diversity(&selected, data);
    
    for _ in 0..max_iters {
        let mut best_swap = None;
        let mut best_gain = 0.0;
        
        // Examine all possible swaps, find the best one
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
        } else {
            break; // No improvement found
        }
    }
    
    (selected, current_diversity)
}

// ============ Tabu Search ============
fn tabu_search(data: &MdpData, max_iters: usize, tabu_tenure: usize) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);
    
    let mut current_selected: Vec<usize> = all_indices[0..data.k].to_vec();
    let mut current_unselected: Vec<usize> = all_indices[data.k..].to_vec();
    let mut current_diversity = calculate_diversity(&current_selected, data);
    
    let mut best_selected = current_selected.clone();
    let mut best_diversity = current_diversity;
    
    // Tabu list: stores (element_in, element_out, iteration_when_tabu_expires)
    let mut tabu_list: Vec<(usize, usize, usize)> = Vec::new();
    
    for iter in 0..max_iters {
        let mut best_swap = None;
        let mut best_swap_gain = f64::NEG_INFINITY;
        
        // Find best non-tabu move (or best tabu move if it's better than best known)
        for i in 0..current_selected.len() {
            for j in 0..current_unselected.len() {
                let elem_out = current_selected[i];
                let elem_in = current_unselected[j];
                let gain = calculate_swap_gain(elem_out, elem_in, &current_selected, data);
                
                let is_tabu = tabu_list.iter().any(|(tin, tout, expires)| {
                    *tin == elem_in && *tout == elem_out && *expires > iter
                });
                
                // Aspiration criterion: accept tabu move if it beats best known
                let new_diversity = current_diversity + gain;
                let aspiration = new_diversity > best_diversity;
                
                if !is_tabu || aspiration {
                    if gain > best_swap_gain {
                        best_swap_gain = gain;
                        best_swap = Some((i, j, elem_in, elem_out));
                    }
                }
            }
        }
        
        if let Some((i, j, elem_in, elem_out)) = best_swap {
            // Perform swap
            current_selected[i] = elem_in;
            current_unselected[j] = elem_out;
            current_diversity += best_swap_gain;
            
            // Update tabu list
            tabu_list.push((elem_out, elem_in, iter + tabu_tenure));
            
            // Clean old tabu entries
            tabu_list.retain(|(_, _, expires)| *expires > iter);
            
            // Update best solution
            if current_diversity > best_diversity {
                best_diversity = current_diversity;
                best_selected = current_selected.clone();
            }
        } else {
            break; // No valid moves
        }
    }
    
    (best_selected, best_diversity)
}

// ============ Helper Functions ============
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