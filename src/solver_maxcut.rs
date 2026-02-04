use grb::prelude::*;
use grb::expr::QuadExpr;
use crate::parser::MdpData;
use std::collections::HashSet;

/// Represents a QUBO problem in standard form: minimize x^T Q x
/// where Q is an upper triangular matrix
pub struct QuboInstance {
    pub n: usize,
    pub q: Vec<Vec<f64>>,  // Upper triangular Q matrix
}

/// Represents a MaxCut problem as a weighted graph
pub struct MaxCutGraph {
    pub n: usize,
    pub weights: Vec<Vec<f64>>,  // Symmetric weight matrix
}

/// Step 1: Convert MDP to QUBO (using penalty method)
/// 
/// MDP: maximize Σ_{i<j} d_ij x_i x_j  subject to  Σx_i = k
/// 
/// QUBO: minimize -Σ_{i<j} d_ij x_i x_j + P(Σx_i - k)²
///       where P is the penalty parameter
pub fn mdp_to_qubo(data: &MdpData, penalty: f64) -> QuboInstance {
    let n = data.n;
    let k = data.k as f64;
    
    // Initialize Q matrix (upper triangular)
    let mut q = vec![vec![0.0; n]; n];
    
    // Add diversity terms: -d_ij x_i x_j (negative because we maximize diversity)
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = data.get_dist(i, j);
            q[i][j] += -dist;  // Upper triangular: store at Q[i][j] for i < j
        }
    }
    
    // Add penalty term: P(Σx_i - k)²
    // Expanded: P·Σx_i² + P·Σ_{i≠j} x_i x_j - 2Pk·Σx_i + Pk²
    // For binary variables: x_i² = x_i
    
    // Diagonal terms: P·x_i - 2Pk·x_i = P(1 - 2k)·x_i
    for i in 0..n {
        q[i][i] += penalty * (1.0 - 2.0 * k);
    }
    
    // Off-diagonal terms: P·x_i·x_j for i < j
    for i in 0..n {
        for j in (i + 1)..n {
            q[i][j] += penalty;
        }
    }
    
    // Constant term Pk² doesn't affect optimization, so we ignore it
    
    QuboInstance { n, q }
}

/// Step 2: Convert QUBO to MaxCut using Barahona et al. method
/// 
/// Creates a MaxCut graph where:
/// - For diagonal Q_ii > 0: add edge (i, dummy) with weight Q_ii
/// - For diagonal Q_ii < 0: add edge (i, dummy) with weight -Q_ii, and flip variable
/// - For off-diagonal Q_ij > 0: add edge (i, j) with weight Q_ij
/// - For off-diagonal Q_ij < 0: add edge (i, j) with weight -Q_ij/4, add edges to dummy
///
/// Reference: Barahona, Jünger, and Reinelt (1989)
pub fn qubo_to_maxcut(qubo: &QuboInstance) -> (MaxCutGraph, Vec<bool>) {
    let n = qubo.n;
    
    // We may need dummy nodes for negative diagonal entries
    let mut num_dummies = 0;
    let mut var_flipped = vec![false; n];  // Track which variables are flipped
    
    // Count how many dummy nodes we need and which variables to flip
    for i in 0..n {
        if qubo.q[i][i] < 0.0 {
            num_dummies += 1;
            var_flipped[i] = true;
        } else if qubo.q[i][i] > 0.0 {
            num_dummies += 1;
        }
    }
    
    // Check for negative off-diagonal entries
    for i in 0..n {
        for j in (i + 1)..n {
            if qubo.q[i][j] < 0.0 {
                num_dummies = num_dummies.max(1); // Need at least one dummy for negative edges
            }
        }
    }
    
    let total_nodes = n + num_dummies;
    let mut weights = vec![vec![0.0; total_nodes]; total_nodes];
    let mut dummy_idx = n;
    
    // Process diagonal entries
    for i in 0..n {
        let q_ii = qubo.q[i][i];
        if q_ii != 0.0 {
            if q_ii > 0.0 {
                // Positive diagonal: add edge to dummy with weight q_ii
                weights[i][dummy_idx] = q_ii;
                weights[dummy_idx][i] = q_ii;
                dummy_idx += 1;
            } else {
                // Negative diagonal: flip variable, add edge to dummy with weight -q_ii
                weights[i][dummy_idx] = -q_ii;
                weights[dummy_idx][i] = -q_ii;
                dummy_idx += 1;
            }
        }
    }
    
    // Process off-diagonal entries
    let dummy_for_negative = n; // Use first dummy for negative off-diagonals
    
    for i in 0..n {
        for j in (i + 1)..n {
            let q_ij = qubo.q[i][j];
            if q_ij > 0.0 {
                // Positive off-diagonal: direct edge with weight q_ij
                weights[i][j] += q_ij;
                weights[j][i] += q_ij;
            } else if q_ij < 0.0 {
                // Negative off-diagonal: use transformation
                // Add edges: (i,j) with weight -q_ij/4
                //            (i,dummy) with weight -q_ij/4
                //            (j,dummy) with weight -q_ij/4
                let w = -q_ij / 4.0;
                weights[i][j] += w;
                weights[j][i] += w;
                weights[i][dummy_for_negative] += w;
                weights[dummy_for_negative][i] += w;
                weights[j][dummy_for_negative] += w;
                weights[dummy_for_negative][j] += w;
            }
        }
    }
    
    (MaxCutGraph { n: total_nodes, weights }, var_flipped)
}

/// Solve MaxCut problem using Gurobi
pub fn solve_maxcut(
    graph: &MaxCutGraph,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, Vec<usize>, f64)> {
    let mut model = Model::new("MaxCut")?;
    let n = graph.n;

    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::MIPGap, 0.01)?;
    model.set_param(param::OutputFlag, 0)?;

    // Variables: x_i ∈ {0, 1} (partition indicator)
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // MaxCut objective: maximize Σ_{i<j} w_ij (x_i + x_j - 2x_i x_j)
    let mut obj = QuadExpr::new();
    
    for i in 0..n {
        for j in (i + 1)..n {
            let w = graph.weights[i][j];
            if w != 0.0 {
                obj.add_term(w, x[i]);
                obj.add_term(w, x[j]);
                obj.add_qterm(-2.0 * w, x[i], x[j]);
            }
        }
    }

    model.set_objective(obj, Maximize)?;
    model.optimize()?;

    let status = model.status()?;
    
    if status == Status::Optimal || 
       status == Status::TimeLimit ||
       status == Status::Interrupted {
        
        let mut partition_0 = Vec::new();
        let mut partition_1 = Vec::new();
        
        for i in 0..n {
            let val = model.get_obj_attr(attr::X, &x[i])?;
            if val > 0.5 {
                partition_1.push(i);
            } else {
                partition_0.push(i);
            }
        }
        
        let cut_value = calculate_cut_value(&partition_0, &partition_1, graph);
        
        Ok((partition_0, partition_1, cut_value))
    } else {
        Ok((Vec::new(), Vec::new(), 0.0))
    }
}

fn calculate_cut_value(part0: &[usize], part1: &[usize], graph: &MaxCutGraph) -> f64 {
    let set0: HashSet<_> = part0.iter().copied().collect();
    let set1: HashSet<_> = part1.iter().copied().collect();
    
    let mut cut = 0.0;
    for i in 0..graph.n {
        for j in (i + 1)..graph.n {
            let w = graph.weights[i][j];
            if w != 0.0 {
                if (set0.contains(&i) && set1.contains(&j)) ||
                   (set1.contains(&i) && set0.contains(&j)) {
                    cut += w;
                }
            }
        }
    }
    cut
}

/// Complete pipeline: MDP → QUBO → MaxCut → MDP solution
pub fn solve_mdp_via_maxcut(
    data: &MdpData,
    penalty: f64,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    // Step 1: MDP → QUBO
    let qubo = mdp_to_qubo(data, penalty);
    
    // Step 2: QUBO → MaxCut
    let (maxcut_graph, var_flipped) = qubo_to_maxcut(&qubo);
    
    // Step 3: Solve MaxCut - try both partitions
    let (part0, part1, _cut_value) = solve_maxcut(&maxcut_graph, time_limit)?;
    
    // Step 4: Extract MDP solution from MaxCut solution
    // We need to try both partitions and see which gives a valid solution
    let n = data.n;
    
    // Try partition 1 as selected
    let (selected1, div1) = extract_solution(data, &part1, &var_flipped, n);
    
    // Try partition 0 as selected
    let (selected0, div0) = extract_solution(data, &part0, &var_flipped, n);
    
    // Choose the better valid solution (closer to k items)
    let (mut selected, mut diversity) = if (selected1.len() as i32 - data.k as i32).abs() 
                                             <= (selected0.len() as i32 - data.k as i32).abs() {
        (selected1, div1)
    } else {
        (selected0, div0)
    };
    
    // If constraint is violated, repair the solution
    if selected.len() != data.k {
        eprintln!("Warning: MaxCut selected {} items (need {}), applying repair", 
                  selected.len(), data.k);
        selected = repair_solution(data, selected);
        
        // Recalculate diversity
        diversity = 0.0;
        for i in 0..selected.len() {
            for j in (i + 1)..selected.len() {
                diversity += data.get_dist(selected[i], selected[j]);
            }
        }
    }
    
    Ok((selected, diversity))
}

/// Extract solution from a partition
fn extract_solution(
    data: &MdpData,
    partition: &[usize],
    var_flipped: &[bool],
    n: usize,
) -> (Vec<usize>, f64) {
    let mut selected = Vec::new();
    let set: HashSet<_> = partition.iter().copied().collect();
    
    for i in 0..n {
        let in_partition = set.contains(&i);
        let flipped = var_flipped[i];
        
        // If variable was flipped, we need to invert the result
        let selected_var = if flipped { !in_partition } else { in_partition };
        
        if selected_var {
            selected.push(i);
        }
    }
    
    // Calculate diversity
    let mut diversity = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            diversity += data.get_dist(selected[i], selected[j]);
        }
    }
    
    (selected, diversity)
}

/// Repair solution to have exactly k items
fn repair_solution(data: &MdpData, mut selected: Vec<usize>) -> Vec<usize> {
    let k = data.k;
    
    if selected.len() > k {
        // Remove items with lowest marginal contribution
        while selected.len() > k {
            let worst_idx = find_worst_item(data, &selected);
            selected.remove(worst_idx);
        }
    } else if selected.len() < k {
        // Add items with highest marginal contribution
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

/// Heuristic MaxCut solver (greedy randomized, for comparison)
pub fn solve_maxcut_greedy(graph: &MaxCutGraph) -> (Vec<usize>, Vec<usize>, f64) {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    
    let mut rng = thread_rng();
    let mut vertices: Vec<usize> = (0..graph.n).collect();
    vertices.shuffle(&mut rng);
    
    let mut part0 = Vec::new();
    let mut part1 = Vec::new();
    
    // Greedy: assign each vertex to partition that maximizes cut
    for &v in &vertices {
        let mut cut_if_0 = 0.0;
        let mut cut_if_1 = 0.0;
        
        for i in 0..graph.n {
            if i == v { continue; }
            
            let w = graph.weights[v][i];
            if w == 0.0 { continue; }
            
            if part0.contains(&i) {
                cut_if_1 += w;
            } else if part1.contains(&i) {
                cut_if_0 += w;
            }
        }
        
        if cut_if_1 >= cut_if_0 {
            part1.push(v);
        } else {
            part0.push(v);
        }
    }
    
    let cut_value = calculate_cut_value(&part0, &part1, graph);
    (part0, part1, cut_value)
}