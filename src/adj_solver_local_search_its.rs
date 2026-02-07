use crate::parser::MdpData;
use rand::prelude::*;
use std::time::{Instant, Duration};

/// Configuration for Iterated Tabu Search based on Palubeckis (2007) parameters.
pub struct ItsConfig {
    pub timeout_secs: f64,
    /// "A good strategy is to fix T at 20" [cite: 205]
    pub tabu_tenure: usize,
    /// Size of the candidate list in GSP (b). "Best results... when b in [5, 10]" [cite: 254]
    pub gsp_candidate_list_size: usize,
    /// Minimum perturbation strength (alpha_1).
    pub alpha_1: usize,
    /// Maximum perturbation factor (alpha_2). p drawn from [alpha_1, alpha_2 * n] [cite: 236]
    pub alpha_2: f64,
    /// Iteration limit for the inner Tabu Search (c_bar) [cite: 209]
    pub ts_max_iters_factor: usize, 
}

impl Default for ItsConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60.0,
            tabu_tenure: 20,
            gsp_candidate_list_size: 5, 
            alpha_1: 2, 
            alpha_2: 0.1, 
            ts_max_iters_factor: 1000, // c_bar = max(10000, beta * n)
        }
    }
}

pub fn solve_its(data: &MdpData, config: &ItsConfig) -> (Vec<usize>, f64) {
    let start_time = Instant::now();
    let time_limit = Duration::from_secs_f64(config.timeout_secs);
    let mut rng = rand::thread_rng();

    // 1. STA Initialization: Steepest Ascent Algorithm 
    // "Used as an alternative method for generating initial solutions"
    let mut current_solution = sta_initialization(data);
    
    // Initialize aux structures
    let mut d_vals = calculate_all_d_values(data, &current_solution);
    let mut current_obj = calculate_objective_from_d(&current_solution, &d_vals);

    let mut best_solution = current_solution.clone();
    let mut best_obj = current_obj;

    // Tabu list: stores the iteration number until which a move involving node i is forbidden.
    // "Indices of positive T_i... constitute the tabu list"
    let mut tabu_list = vec![0usize; data.n];
    let mut total_iter = 0;

    loop {
        // Check timeout
        if start_time.elapsed() >= time_limit {
            break;
        }

        // 2. Tabu Search Procedure
        // The paper's ITS structure alternates between TS and GSP.
        // We pass the current solution and the global best.
        
        let ts_limit = std::cmp::max(10_000, config.ts_max_iters_factor * data.n);
        let mut ts_iter = 0;
        
        // Inner Tabu Search Loop
        while ts_iter < ts_limit {
             if start_time.elapsed() >= time_limit { break; }
             total_iter += 1;
             ts_iter += 1;

            let (in_sol, out_sol): (Vec<usize>, Vec<usize>) = (0..data.n).partition(|&i| current_solution.contains(&i));
            
            let mut best_move = None;
            let mut best_move_gain = f64::NEG_INFINITY;

            // Search Neighborhood: Swaps (assuming m1=m2=k)
            // "Short-term memory tabu list without aspiration criterion" [cite: 173]
            for &u in &in_sol {
                for &v in &out_sol {
                    // Check Tabu status
                    let is_tabu = tabu_list[u] > total_iter || tabu_list[v] > total_iter;

                    if !is_tabu {
                        let dist_uv = data.get_dist(u, v);
                        let gain = d_vals[v] - d_vals[u] - dist_uv;

                        if gain > best_move_gain {
                            best_move_gain = gain;
                            best_move = Some((u, v));
                        }
                    }
                }
            }

            // Apply Move
            if let Some((u, v)) = best_move {
                apply_swap(data, &mut current_solution, u, v, &mut d_vals);
                current_obj += best_move_gain;

                // Update Tabu Tenure
                // "Decrement by one... If r < 0 set T_q := T" [cite: 201-202]
                // We use absolute iteration counts for efficiency.
                tabu_list[u] = total_iter + config.tabu_tenure;
                tabu_list[v] = total_iter + config.tabu_tenure;

                // "In the case of finding... a solution x that is better than x*, a local search... is executed" [cite: 177]
                if current_obj > best_obj + 1e-6 {
                     // Perform LS ascent
                     steepest_ascent_ls(data, &mut current_solution, &mut d_vals, &mut current_obj);
                     
                     // Update global best
                     if current_obj > best_obj {
                         best_obj = current_obj;
                         best_solution = current_solution.clone();
                     }
                }
            } else {
                // No valid non-tabu moves found, break inner TS to trigger perturbation
                break;
            }
        }

        // 3. Solution Perturbation (GSP) 
        // "Check if stopping criterion is met... Otherwise proceed to 4. Apply GSP" [cite: 162-164]
        
        // Determine p (perturbation strength)
        // "p is an integer number randomly and uniformly drawn from [alpha_1, floor(alpha_2 * n)]" [cite: 164]
        let max_p = (config.alpha_2 * data.n as f64).floor() as usize;
        let p = rng.gen_range(config.alpha_1..=max_p.max(config.alpha_1));

        gsp_perturbation(data, &mut current_solution, &mut d_vals, p, config.gsp_candidate_list_size, &mut rng);
        
        // Recalculate obj after perturbation to be safe
        current_obj = calculate_objective_from_d(&current_solution, &d_vals);
        
        // "The perturbed 0-1 vector x... serves as a starting point for the tabu search" [cite: 233]
        // Reset tabu list to allow free movement in new region
        tabu_list.fill(0); 
    }

    (best_solution, best_obj)
}

// ==============================================================================
// STA (Steepest Ascent) Initialization 
// ==============================================================================

fn sta_initialization(data: &MdpData) -> Vec<usize> {
    // Reformulation: f(x) = g(y)/n^2 where y_i in {0, n} [cite: 97-101]
    // Start with y^0 = (m, m, ..., m)
    
    let m_val = data.k as f64; // m in paper
    let n_val = data.n as f64; // n in paper
    
    // W is the set of "free" vertices (not yet fixed to 0 or n)
    let mut w_set: Vec<usize> = (0..data.n).collect();
    let mut u_set: Vec<usize> = Vec::with_capacity(data.k); // Fixed to n (selected)
    
    // Calculate initial d_tilde (sum of dists to W\{i}) and d (sum of dists to U)
    // Initially U is empty, d_i = 0.
    // d_tilde_i = sum of all edges from i to other nodes (since W=V initially)
    let mut d_tilde = vec![0.0; data.n];
    let mut d_fixed = vec![0.0; data.n]; // d_i in paper (sum to U)

    for i in 0..data.n {
        for j in 0..data.n {
            if i != j {
                d_tilde[i] += data.get_dist(i, j);
            }
        }
    }

    let mut rng = rand::thread_rng();

    // Main Loop STA [cite: 126-141]
    while !w_set.is_empty() {
        // Compute gradients Delta_i(0) and Delta_i(n) for all i in W
        // Delta_i(a) = (a - m)(m * d_tilde_i + n * d_fixed_i) [cite: 111]
        
        let mut best_k = None;
        let mut max_lambda = f64::NEG_INFINITY;
        let mut best_action_is_n = false; // true if fixing to n (select), false if 0 (reject)

        // Find k with max Lambda_k = max(Delta_k(0), Delta_k(n)) [cite: 129]
        for &i in &w_set {
            // Option 0: Fix y_i = 0
            // Delta_i(0) = (0 - m) * (...) = -m * (m * d_tilde[i] + n * d_fixed[i])
            let delta_0 = -m_val * (m_val * d_tilde[i] + n_val * d_fixed[i]);

            // Option n: Fix y_i = n
            // Delta_i(n) = (n - m) * (...)
            let delta_n = (n_val - m_val) * (m_val * d_tilde[i] + n_val * d_fixed[i]);

            // Choose larger of the two
            let (lambda_i, is_n) = if delta_n > delta_0 {
                (delta_n, true)
            } else {
                (delta_0, false)
            };

            // Maximize Lambda
            if lambda_i > max_lambda {
                max_lambda = lambda_i;
                best_k = Some(i);
                best_action_is_n = is_n;
            } else if (lambda_i - max_lambda).abs() < 1e-6 {
                // Tie breaking random [cite: 131]
                if rng.gen_bool(0.5) {
                    best_k = Some(i);
                    best_action_is_n = is_n;
                }
            }
        }

        let k = best_k.unwrap();

        // 3. Fix y_k [cite: 130-132]
        // Remove k from W
        if let Some(pos) = w_set.iter().position(|&x| x == k) {
            w_set.swap_remove(pos);
        }

        if best_action_is_n {
            u_set.push(k);
            // 4. Update d values [cite: 133]
            // For each i in W: d_tilde_i -= d_ik; d_fixed_i += d_ik
            for &i in &w_set {
                let dist = data.get_dist(i, k);
                d_tilde[i] -= dist;
                d_fixed[i] += dist;
            }
        } else {
            // y_k = 0
            // For each i in W: d_tilde_i -= d_ik (removed from free set)
            // d_fixed_i does not change (k is not added to U)
            for &i in &w_set {
                let dist = data.get_dist(i, k);
                d_tilde[i] -= dist;
            }
        }

        // 5. Check constraints [cite: 134-135]
        if u_set.len() == data.k {
            // If we have selected enough, force remaining W to 0
            break;
        }
        
        let remaining_needed = data.k - u_set.len();
        if w_set.len() == remaining_needed {
            // Force all remaining W to n (select all)
            for &j in &w_set {
                u_set.push(j);
            }
            break;
        }
    }

    u_set
}

// ==============================================================================
// GSP (Get Start Point) Perturbation [cite: 239-248]
// ==============================================================================

fn gsp_perturbation(
    data: &MdpData,
    solution: &mut Vec<usize>,
    d_vals: &mut Vec<f64>,
    p_target: usize,
    b_size: usize,
    rng: &mut ThreadRng
) {
    let mut p_count = 0;

    // Loop until we have performed p flips
    while p_count < p_target {
        let (in_sol, out_sol): (Vec<usize>, Vec<usize>) = (0..data.n).partition(|&i| solution.contains(&i));
        
        // Construct Candidate List S'
        // "Form a subset S'... pick the b largest values z_i" [cite: 244]
        // z_i = d_k - d_j - d_jk (Move gain) [cite: 243]
        
        let mut candidates = Vec::with_capacity(in_sol.len() * out_sol.len());

        for &j in &in_sol { // j in U
            for &k in &out_sol { // k in V\U
                let dist_jk = data.get_dist(j, k);
                // Gain calculation: d[k] (gain from adding k) - d[j] (loss from removing j) - dist (adj)
                let z = d_vals[k] - d_vals[j] - dist_jk;
                candidates.push((z, j, k));
            }
        }

        // Sort candidates by z descending
        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Select top b
        let limit = std::cmp::min(b_size, candidates.len());
        if limit == 0 { break; } 

        // "Randomly select s_i in S'" [cite: 245]
        let pick_idx = rng.gen_range(0..limit);
        let (_, u, v) = candidates[pick_idx]; // u is in sol (to remove), v is out (to add)

        // Apply swap
        apply_swap(data, solution, u, v, d_vals);
        
        // "p := p + 2" (Since we swapped a pair, we flipped 2 variables) [cite: 246]
        p_count += 2;
    }
}

// ==============================================================================
// Standard Local Search (Steepest Ascent) [cite: 215-225]
// ==============================================================================

fn steepest_ascent_ls(
    data: &MdpData, 
    solution: &mut Vec<usize>, 
    d_vals: &mut Vec<f64>,
    current_obj: &mut f64
) {
    loop {
        let (in_sol, out_sol): (Vec<usize>, Vec<usize>) = (0..data.n).partition(|&i| solution.contains(&i));
        
        let mut best_swap = None;
        let mut best_gain = 1e-9; // Strict improvement required
        
        // Full neighborhood scan (Steepest Ascent)
        for &u in &in_sol {
            for &v in &out_sol {
                let dist_uv = data.get_dist(u, v);
                let gain = d_vals[v] - d_vals[u] - dist_uv;
                
                if gain > best_gain {
                    best_gain = gain;
                    best_swap = Some((u, v));
                }
            }
        }
        
        if let Some((u, v)) = best_swap {
            apply_swap(data, solution, u, v, d_vals);
            *current_obj += best_gain;
        } else {
            break; // Local Optimum reached
        }
    }
}

// ==============================================================================
// Helper: Efficient Updates
// ==============================================================================

/// Perform swap and update D-values incrementally in O(N)
fn apply_swap(
    data: &MdpData, 
    solution: &mut Vec<usize>, 
    u: usize, // remove
    v: usize, // add
    d_vals: &mut Vec<f64>
) {
    // Update solution vector
    if let Some(pos) = solution.iter().position(|&x| x == u) {
        solution[pos] = v;
    }
    
    // Update D-values (Contribution of every node to the set S)
    // For any node k: New_D[k] = Old_D[k] - dist(k, u) + dist(k, v) [cite: 133, 141]
    for k in 0..data.n {
        let dist_ku = data.get_dist(k, u);
        let dist_kv = data.get_dist(k, v);
        d_vals[k] = d_vals[k] - dist_ku + dist_kv;
    }
}

fn calculate_all_d_values(data: &MdpData, solution: &[usize]) -> Vec<f64> {
    let mut d = vec![0.0; data.n];
    for i in 0..data.n {
        let mut sum = 0.0;
        for &s in solution {
            sum += data.get_dist(i, s);
        }
        d[i] = sum;
    }
    d
}

fn calculate_objective_from_d(solution: &[usize], d_vals: &[f64]) -> f64 {
    let mut sum = 0.0;
    for &s in solution {
        sum += d_vals[s];
    }
    sum / 2.0 // Distances are double counted in sum of D-values
}