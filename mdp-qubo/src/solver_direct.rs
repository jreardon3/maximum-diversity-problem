use crate::parser::MdpData;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn solve_direct(data: &MdpData) -> (Vec<usize>, f64) {
    let mut rng = thread_rng();
    let n = data.n;
    let k = data.k;

    // 1. Initial Solution: Randomly select k indices
    let mut all_indices: Vec<usize> = (0..n).collect();
    all_indices.shuffle(&mut rng);
    let mut selected: Vec<usize> = all_indices[0..k].to_vec();
    let mut unselected: Vec<usize> = all_indices[k..n].to_vec();

    let mut current_diversity = calculate_diversity(&selected, data);
    let mut improved = true;

    // 2. Local Search (First Improvement)
    while improved {
        improved = false;
        'outer: for i in 0..selected.len() {
            for j in 0..unselected.len() {
                let old_val = selected[i];
                let new_val = unselected[j];

                // Calculate gain of swapping old_val for new_val
                let gain = calculate_swap_gain(old_val, new_val, &selected, data);

                if gain > 0.0 {
                    selected[i] = new_val;
                    unselected[j] = old_val;
                    current_diversity += gain;
                    improved = true;
                    break 'outer; // Greedily take the first improvement found
                }
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
        if s == old { continue; }
        gain += data.get_dist(new, s) - data.get_dist(old, s);
    }
    gain
}