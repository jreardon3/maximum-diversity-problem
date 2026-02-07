use grb::prelude::*;
use crate::parser::MdpData;

/// Solves the MDP using the QUBO formulation described in the text (Transformation #1).
/// 
/// Steps implemented:
/// 1. Handle negative distances (add constant to make d'_ij >= 0).
/// 2. Transform into QUBO: Maximize sum(d_ij * xi * xj) - P * (sum(xi) - k)^2.
/// 3. Apply strict time limits.
/// 4. Apply a Greedy Repair Mechanism to ensure |S| = k.
pub fn solve(
    data: &MdpData,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    let mut model = Model::new("MDP_QUBO_Text_Description")?;
    let n = data.n;
    let k = data.k;

    // --- Step 1: Handle Negative Distances ---
    // The text states: "For instances with negative d_ij, we add a constant... ensuring d'_{ij} >= 0"
    let min_dist = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| data.get_dist(i, j)))
        .fold(f64::INFINITY, |a, b| a.min(b));

    let shift = if min_dist < 0.0 { -min_dist } else { 0.0 };

    // --- Step 2: Calculate Penalty P ---
    // P must be large enough to dominate the gain from adding an extra element.
    let mut sum_abs_dist = 0.0;
    for i in 0..n {
        for j in (i + 1..n) {
            sum_abs_dist += (data.get_dist(i, j) + shift).abs();
        }
    }
    let p_penalty = sum_abs_dist + 1.0; 

    // --- Step 3: Configure Gurobi ---
    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::OutputFlag, 0)?; // Silence output
    model.set_param(param::MIPGap, 0.01)?;  // 1% gap

    // --- Step 4: Build Variables ---
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // --- Step 5: Construct Objective Function ---
    // Maximize: Diversity - Penalty
    // Maximize: sum_{i<j} (d_ij + shift) x_i x_j  -  P * (sum x_i - k)^2
    
    let mut obj = grb::expr::QuadExpr::new();

    // Add Diversity Terms and Quadratic Penalty Terms
    for i in 0..n {
        for j in (i + 1..n) {
            let d_prime = data.get_dist(i, j) + shift;
            
            // Term: d'_ij * x_i * x_j
            // Term: -2 * P * x_i * x_j
            let coef = d_prime - (2.0 * p_penalty);
            
            if coef != 0.0 {
                obj.add_qterm(coef, x[i], x[j]);
            }
        }
    }

    // Add Linear Penalty Terms
    // Term: -P * (1 - 2k) * x_i  =>  P(2k - 1) * x_i
    let linear_penalty_coef = p_penalty * (2.0 * (k as f64) - 1.0);
    for i in 0..n {
        obj.add_term(linear_penalty_coef, x[i]);
    }

    // Constant term: -P * k^2 
    obj.add_constant(-p_penalty * (k as f64).powi(2));

    model.set_objective(obj, Maximize)?;

    // --- Step 6: Solve ---
    model.optimize()?;

    // --- Step 7: Extract and Repair ---
    let status = model.status()?;
    let mut selected_indices = Vec::new();

    if status == Status::Optimal || status == Status::TimeLimit || status == Status::Interrupted {
        for (i, var) in x.iter().enumerate() {
            if let Ok(val) = model.get_obj_attr(attr::X, var) {
                if val > 0.5 {
                    selected_indices.push(i);
                }
            }
        }
    }
    
    if selected_indices.is_empty() && status != Status::Optimal && status != Status::TimeLimit {
        return Ok((Vec::new(), 0.0));
    }

    // Perform Repair
    let repaired_solution = repair_solution(&selected_indices, data, k);
    
    // Calculate final true diversity (using original un-shifted distances)
    let final_diversity = calculate_true_diversity(&repaired_solution, data);

    Ok((repaired_solution, final_diversity))
}

/// Repair mechanism: Greedily adds or removes elements until |S| = k
fn repair_solution(current_set: &[usize], data: &MdpData, k: usize) -> Vec<usize> {
    let mut solution = current_set.to_vec();
    let n = data.n;

    // Case 1: Too many elements -> Remove worst
    while solution.len() > k {
        let mut best_remove_idx = 0;
        let mut min_loss = f64::MAX;

        for (idx_in_sol, &candidate) in solution.iter().enumerate() {
            let contribution: f64 = solution.iter()
                .filter(|&&other| other != candidate)
                .map(|&other| data.get_dist(candidate, other))
                .sum();
            
            if contribution < min_loss {
                min_loss = contribution;
                best_remove_idx = idx_in_sol;
            }
        }
        solution.swap_remove(best_remove_idx);
    }

    // Case 2: Too few elements -> Add best
    while solution.len() < k {
        let mut best_add_node = None;
        let mut max_gain = f64::MIN;

        for candidate in 0..n {
            if !solution.contains(&candidate) {
                let gain: f64 = solution.iter()
                    .map(|&existing| data.get_dist(candidate, existing))
                    .sum();
                
                if gain > max_gain {
                    max_gain = gain;
                    best_add_node = Some(candidate);
                }
            }
        }

        if let Some(node) = best_add_node {
            solution.push(node);
        } else {
            break;
        }
    }

    solution.sort(); 
    solution
}

fn calculate_true_diversity(selected: &[usize], data: &MdpData) -> f64 {
    let mut sum = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            sum += data.get_dist(selected[i], selected[j]);
        }
    }
    sum
}