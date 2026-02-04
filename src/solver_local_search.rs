use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;
use std::time::Instant;

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

// pub fn solve_local_search(data: &MdpData, config: &LocalSearchConfig) -> (Vec<usize>, f64) {
// pub fn solve_local_search(
//     data: &MdpData,
//     config: &LocalSearchConfig,
//     deadline: Instant,
// ) -> (Vec<usize>, f64)
// {
//     match &config.method {
//         LocalSearchMethod::FirstImprovement => first_improvement_search(data, config.max_iters),
//         LocalSearchMethod::BestImprovement => best_improvement_search(data, config.max_iters),
//         LocalSearchMethod::TabuSearch { tabu_tenure } => tabu_search(data, config.max_iters, *tabu_tenure),
//     }
// }
pub fn solve_local_search(
    data: &MdpData,
    config: &LocalSearchConfig,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    match &config.method {
        LocalSearchMethod::FirstImprovement =>
            first_improvement_search(data, config.max_iters, deadline),

        LocalSearchMethod::BestImprovement =>
            best_improvement_search(data, config.max_iters, deadline),

        LocalSearchMethod::TabuSearch { tabu_tenure } =>
            tabu_search(data, config.max_iters, *tabu_tenure, deadline),
    }
}


// ============ First Improvement (like your original) ============
fn first_improvement_search(data: &MdpData, max_iters: usize, deadline: Instant) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);
    
    let mut selected: Vec<usize> = all_indices[0..data.k].to_vec();
    let mut unselected: Vec<usize> = all_indices[data.k..].to_vec();
    let mut current_diversity = calculate_diversity(&selected, data);
    
    for _ in 0..max_iters {
        if Instant::now() >= deadline {
            break;
        }

        let mut improved = false;

        'outer: for i in 0..selected.len() {
            if Instant::now() >= deadline {
                break 'outer;
            }

            for j in 0..unselected.len() {
                if Instant::now() >= deadline {
                    break 'outer;
                }

                let gain = calculate_swap_gain(selected[i], unselected[j], &selected, data);

                if gain > 1e-9 {
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
fn best_improvement_search(data: &MdpData, max_iters: usize, deadline: Instant) -> (Vec<usize>, f64) {
    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);
    
    let mut selected: Vec<usize> = all_indices[0..data.k].to_vec();
    let mut unselected: Vec<usize> = all_indices[data.k..].to_vec();
    let mut current_diversity = calculate_diversity(&selected, data);
    
    for _ in 0..max_iters {
        if Instant::now() >= deadline {
            break;
        }

        let mut best_swap = None;
        let mut best_gain = 0.0;

        for i in 0..selected.len() {
            if Instant::now() >= deadline {
                break;
            }

            for j in 0..unselected.len() {
                if Instant::now() >= deadline {
                    break;
                }

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
            break;
        }
    }

    
    (selected, current_diversity)
}

// ============ Tabu Search ============
fn tabu_search(
    data: &MdpData,
    max_iters: usize,
    tabu_tenure: usize,
    deadline: Instant,
) -> (Vec<usize>, f64) {

    let mut rng = rand::thread_rng();
    let mut all_indices: Vec<usize> = (0..data.n).collect();
    all_indices.shuffle(&mut rng);

    let mut current_selected = all_indices[..data.k].to_vec();
    let mut current_unselected = all_indices[data.k..].to_vec();
    let mut current_div = calculate_diversity(&current_selected, data);

    let mut best_selected = current_selected.clone();
    let mut best_div = current_div;

    let mut tabu_list: Vec<(usize, usize, usize)> = Vec::new();

    for iter in 0..max_iters {
        // ⏱ HARD STOP
        if Instant::now() >= deadline {
            break;
        }

        let mut best_move = None;
        let mut best_gain = f64::NEG_INFINITY;

        for i in 0..current_selected.len() {
            // ⏱ CHECK INSIDE NESTED LOOP
            if Instant::now() >= deadline {
                break;
            }

            for j in 0..current_unselected.len() {
                if Instant::now() >= deadline {
                    break;
                }
                let out = current_selected[i];
                let inn = current_unselected[j];
                let gain = calculate_swap_gain(out, inn, &current_selected, data);

                let is_tabu = tabu_list.iter().any(|(ti, to, exp)| {
                    *ti == inn && *to == out && *exp > iter
                });

                let aspiration = current_div + gain > best_div;

                if (!is_tabu || aspiration) && gain > best_gain {
                    best_gain = gain;
                    best_move = Some((i, j, inn, out));
                }
            }
        }

        if let Some((i, j, inn, out)) = best_move {
            current_selected[i] = inn;
            current_unselected[j] = out;
            current_div += best_gain;

            tabu_list.push((out, inn, iter + tabu_tenure));
            tabu_list.retain(|(_, _, exp)| *exp > iter);

            if current_div > best_div {
                best_div = current_div;
                best_selected = current_selected.clone();
            }
        } else {
            break;
        }
    }

    (best_selected, best_div)
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