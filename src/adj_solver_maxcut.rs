use grb::prelude::*;
use grb::expr::QuadExpr;
use crate::parser::MdpData;
use std::collections::HashSet;

/// Represents a QUBO problem: minimize x^T Q x
/// Q is upper triangular
pub struct QuboInstance {
    pub n: usize,
    pub q: Vec<Vec<f64>>,
}

/// Represents a MaxCut problem on a weighted graph
pub struct MaxCutGraph {
    pub n: usize,
    pub weights: Vec<Vec<f64>>,
}

/// Step 1: Convert MDP to QUBO
/// Minimize -Σ d_ij x_i x_j + P(Σ x_i - k)²
pub fn mdp_to_qubo(data: &MdpData, penalty: f64) -> QuboInstance {
    let n = data.n;
    let k = data.k as f64;
    
    // Initialize Q matrix (upper triangular)
    let mut q = vec![vec![0.0; n]; n];
    
    // 1. Diversity term: maximize Σ d_ij x_i x_j  =>  minimize Σ -d_ij x_i x_j
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = data.get_dist(i, j);
            q[i][j] -= dist;
        }
    }
    
    // 2. Penalty term: P(Σ x_i - k)²
    // Expansion: P(Σ x_i² + 2Σ_{i<j} x_i x_j - 2kΣ x_i + k²)
    // Since x_i is binary, x_i² = x_i.
    
    // Diagonal updates: P * x_i - 2Pk * x_i  = P(1 - 2k)x_i
    for i in 0..n {
        q[i][i] += penalty * (1.0 - 2.0 * k);
    }
    
    // Off-diagonal updates: 2P * x_i * x_j
    // Note: The expansion of (Σx)^2 has 2*x_i*x_j for i<j. 
    // Q is upper triangular, so Q[i][j] captures the whole interaction.
    for i in 0..n {
        for j in (i + 1)..n {
            q[i][j] += 2.0 * penalty;
        }
    }
    
    QuboInstance { n, q }
}

/// Step 2: Convert QUBO to MaxCut
/// 
/// Uses the Barahona et al. reduction with a SINGLE auxiliary node.
/// Transformation: x_i = (s_i + 1) / 2
/// 
/// Weights W derived from expansion of x^T Q x:
/// W_ij = Q_ij / 4                    (for i < j < n)
/// W_i,aux = Q_ii/2 + Σ_{k≠i} Q_ik/4  (edge to auxiliary node)
pub fn qubo_to_maxcut(qubo: &QuboInstance) -> MaxCutGraph {
    let n = qubo.n;
    let aux_idx = n; // The auxiliary node is at index n
    let total_nodes = n + 1;
    
    let mut weights = vec![vec![0.0; total_nodes]; total_nodes];
    
    // Accumulate linear terms here before assigning to W_i,aux
    let mut linear_terms = vec![0.0; n];
    
    // Process Diagonal Terms (Q_ii)
    for i in 0..n {
        // Q_ii x_i  =>  Q_ii/2 * s_i * s_aux  (plus constant)
        linear_terms[i] += qubo.q[i][i] / 2.0;
    }
    
    // Process Off-Diagonal Terms (Q_ij)
    for i in 0..n {
        for j in (i + 1)..n {
            let q_val = qubo.q[i][j];
            
            // Interaction term: Q_ij x_i x_j
            // Becomes: (Q_ij/4) * s_i * s_j  + linear parts
            
            // 1. Edge between i and j
            weights[i][j] += q_val / 4.0;
            weights[j][i] += q_val / 4.0;
            
            // 2. Contributions to linear terms (edges to aux)
            linear_terms[i] += q_val / 4.0;
            linear_terms[j] += q_val / 4.0;
        }
    }
    
    // Assign linear terms to edges connecting to the auxiliary node
    for i in 0..n {
        let w = linear_terms[i];
        if w != 0.0 {
            weights[i][aux_idx] = w;
            weights[aux_idx][i] = w;
        }
    }
    
    MaxCutGraph { n: total_nodes, weights }
}

/// Solve MaxCut using Gurobi
pub fn solve_maxcut(
    graph: &MaxCutGraph,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, Vec<usize>, f64)> {
    let mut model = Model::new("MaxCut")?;
    let n = graph.n;

    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::OutputFlag, 0)?;

    // x[i] = 1 if in partition 1, 0 if in partition 0
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // MaxCut Objective: Maximize Σ w_ij * (x_i + x_j - 2*x_i*x_j)
    // The term (x_i + x_j - 2*x_i*x_j) is 1 if x_i != x_j, else 0.
    let mut obj = QuadExpr::new();
    
    for i in 0..n {
        for j in (i + 1)..n {
            let w = graph.weights[i][j];
            if w != 0.0 {
                // Add w * x_i
                obj.add_term(w, x[i]);
                // Add w * x_j
                obj.add_term(w, x[j]);
                // Add -2w * x_i * x_j
                obj.add_qterm(-2.0 * w, x[i], x[j]);
            }
        }
    }

    model.set_objective(obj, Maximize)?;
    model.optimize()?;

    let status = model.status()?;
    if status == Status::Optimal || status == Status::TimeLimit {
        let mut part0 = Vec::new();
        let mut part1 = Vec::new();
        
        for i in 0..n {
            let val = model.get_obj_attr(attr::X, &x[i])?;
            if val > 0.5 {
                part1.push(i);
            } else {
                part0.push(i);
            }
        }
        
        // Calculate cut value purely for reporting
        let cut_val = model.get_attr(attr::ObjVal)?;
        
        Ok((part0, part1, cut_val))
    } else {
        Ok((vec![], vec![], 0.0))
    }
}

/// Main pipeline
pub fn solve_mdp_via_maxcut(
    data: &MdpData,
    penalty: f64,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    // 1. MDP -> QUBO
    let qubo = mdp_to_qubo(data, penalty);
    
    // 2. QUBO -> MaxCut (Barahona Single Auxiliary Node)
    let maxcut_graph = qubo_to_maxcut(&qubo);
    
    // 3. Solve MaxCut
    let (part0, part1, _) = solve_maxcut(&maxcut_graph, time_limit)?;
    
    if part0.is_empty() && part1.is_empty() {
        return Ok((vec![], 0.0));
    }

    // 4. Extract Solution
    // The auxiliary node is at index `data.n`.
    // We find which partition contains the auxiliary node.
    // Nodes in the SAME partition as aux are "selected" (x=1).
    // Nodes in the OPPOSITE partition are "not selected" (x=0).
    
    let aux_node = data.n;
    let aux_in_part1 = part1.contains(&aux_node);

    let mut selected = Vec::new();
    
    // Determine which partition represents "1"
    let selected_partition = if aux_in_part1 { &part1 } else { &part0 };

    for &node_idx in selected_partition {
        // Ignore the auxiliary node itself
        if node_idx != aux_node {
            selected.push(node_idx);
        }
    }

    // 5. Repair Solution (Heuristic) if |S| != k
    if selected.len() != data.k {
        // repair_solution function assumed to be available from previous context
        // or imported from a utility module
        selected = repair_solution(data, selected);
    }
    
    // Calculate final diversity
    let mut diversity = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            diversity += data.get_dist(selected[i], selected[j]);
        }
    }
    
    Ok((selected, diversity))
}

// ---------------------------------------------------------
// Helper functions for repair (same as before)
// ---------------------------------------------------------

fn repair_solution(data: &MdpData, mut selected: Vec<usize>) -> Vec<usize> {
    let k = data.k;
    if selected.len() > k {
        while selected.len() > k {
            let worst = find_worst_item(data, &selected);
            selected.remove(worst);
        }
    } else if selected.len() < k {
        let mut unselected: Vec<usize> = (0..data.n)
            .filter(|i| !selected.contains(i))
            .collect();
        while selected.len() < k && !unselected.is_empty() {
            let best = find_best_item(data, &selected, &unselected);
            selected.push(best);
            unselected.retain(|&x| x != best);
        }
    }
    selected
}

fn find_worst_item(data: &MdpData, selected: &[usize]) -> usize {
    let mut worst_idx = 0;
    let mut min_contrib = f64::INFINITY;
    for (idx, &item) in selected.iter().enumerate() {
        let contrib: f64 = selected.iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, &other)| data.get_dist(item, other))
            .sum();
        if contrib < min_contrib {
            min_contrib = contrib;
            worst_idx = idx;
        }
    }
    worst_idx
}

fn find_best_item(data: &MdpData, selected: &[usize], unselected: &[usize]) -> usize {
    let mut best_item = unselected[0];
    let mut max_contrib = f64::NEG_INFINITY;
    for &item in unselected {
        let contrib: f64 = selected.iter()
            .map(|&other| data.get_dist(item, other))
            .sum();
        if contrib > max_contrib {
            max_contrib = contrib;
            best_item = item;
        }
    }
    best_item
}