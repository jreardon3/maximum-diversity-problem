use crate::parser::MdpData;
use rand::Rng;

pub struct GraspConfig {
    pub iterations: usize,
    pub alpha: f64,  // RCL parameter: 0.0 = pure greedy, 1.0 = pure random
    pub local_search_iters: usize,
}

impl Default for GraspConfig {
    fn default() -> Self {
        GraspConfig {
            iterations: 100,
            alpha: 0.3,
            local_search_iters: 1000,
        }
    }
}

pub fn solve_grasp(data: &MdpData, config: &GraspConfig) -> (Vec<usize>, f64) {
    let mut best_solution = Vec::new();
    let mut best_diversity = f64::NEG_INFINITY;

    for _iter in 0..config.iterations {
        // Construction phase: greedy randomized
        let solution = greedy_randomized_construction(data, config.alpha);
        
        // Local search phase
        let (improved_solution, diversity) = local_search(data, solution, config.local_search_iters);
        
        if diversity > best_diversity {
            best_diversity = diversity;
            best_solution = improved_solution;
        }
    }

    (best_solution, best_diversity)
}

fn greedy_randomized_construction(data: &MdpData, alpha: f64) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut selected = Vec::with_capacity(data.k);
    let mut available: Vec<usize> = (0..data.n).collect();

    for _ in 0..data.k {
        if available.is_empty() {
            break;
        }

        // Calculate contribution of each available element
        let mut contributions: Vec<(usize, f64)> = available
            .iter()
            .map(|&idx| {
                let contrib = calculate_marginal_contribution(idx, &selected, data);
                (idx, contrib)
            })
            .collect();

        // Sort by contribution (descending)
        contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Build RCL (Restricted Candidate List)
        let c_min = contributions.last().unwrap().1;
        let c_max = contributions.first().unwrap().1;
        let threshold = c_max - alpha * (c_max - c_min);

        let rcl: Vec<usize> = contributions
            .iter()
            .filter(|(_, contrib)| *contrib >= threshold)
            .map(|(idx, _)| *idx)
            .collect();

        // Randomly select from RCL
        let chosen = rcl[rng.gen_range(0..rcl.len())];
        selected.push(chosen);
        available.retain(|&x| x != chosen);
    }

    selected
}

fn calculate_marginal_contribution(candidate: usize, selected: &[usize], data: &MdpData) -> f64 {
    let mut contribution = 0.0;
    for &s in selected {
        contribution += data.get_dist(candidate, s);
    }
    contribution
}

fn local_search(data: &MdpData, mut selected: Vec<usize>, max_iters: usize) -> (Vec<usize>, f64) {
    let mut unselected: Vec<usize> = (0..data.n)
        .filter(|&i| !selected.contains(&i))
        .collect();

    let mut current_diversity = calculate_diversity(&selected, data);
    let mut no_improvement_count = 0;

    for _ in 0..max_iters {
        let mut best_swap = None;
        let mut best_gain = 0.0;

        // Find best swap
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
            // Perform swap
            let temp = selected[i];
            selected[i] = unselected[j];
            unselected[j] = temp;
            current_diversity += best_gain;
            no_improvement_count = 0;
        } else {
            no_improvement_count += 1;
            if no_improvement_count > 10 {
                break; // No improvement for several iterations
            }
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