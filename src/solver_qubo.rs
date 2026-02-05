use grb::prelude::*;
use crate::parser::MdpData;
use grb::expr::QuadExpr;

/// QUBO with HARD CONSTRAINT (recommended approach)
/// Adds explicit constraint: sum(x_i) = k
pub fn solve_with_qubo_constrained(
    data: &MdpData,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    let mut model = Model::new("MDP_QUBO_Constrained")?;
    let n = data.n;
    let k = data.k;

    // Set Gurobi parameters
    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::MIPGap, 0.01)?;
    model.set_param(param::OutputFlag, 0)?;

    // Variables
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // Hard cardinality constraint
    model.add_constr(
        &format!("cardinality"),
        c!(x.iter().grb_sum() == k as i32)
    )?;

    // Objective: maximize diversity
    let mut obj = QuadExpr::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = data.get_dist(i, j);
            if dist != 0.0 {
                obj.add_qterm(dist, x[i], x[j]);
            }
        }
    }

    model.set_objective(obj, Maximize)?;
    model.optimize()?;

    extract_solution(&model, &x, data, k)
}

/// QUBO with PENALTY METHOD (soft constraint)
/// Objective: maximize diversity - penalty * (sum(x_i) - k)^2
pub fn solve_with_qubo_penalty(
    data: &MdpData,
    penalty_param: f64,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    let mut model = Model::new("MDP_QUBO_Penalty")?;
    let n = data.n;
    let k = data.k;

    // Set Gurobi parameters
    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::MIPGap, 0.01)?;
    model.set_param(param::OutputFlag, 0)?;

    // Variables
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // Objective: diversity term + penalty term
    let mut obj = QuadExpr::new();
    
    // Diversity term: sum_{i<j} d_ij x_i x_j
    for i in 0..n {
        for j in (i + 1)..n {
            let dist = data.get_dist(i, j);
            if dist != 0.0 {
                obj.add_qterm(dist, x[i], x[j]);
            }
        }
    }

    // Penalty term: -λ(sum_i x_i - k)^2
    // Expanded: -λ[sum_i x_i^2 + sum_{i≠j} x_i*x_j - 2k*sum_i x_i + k^2]
    // For binary: x_i^2 = x_i
    
    let k_f64 = k as f64;
    
    // -λ * x_i (from x_i^2 = x_i)
    for i in 0..n {
        obj.add_term(-penalty_param, x[i]);
    }
    
    // -λ * 2 * x_i * x_j (the cross terms, factor of 2 accounts for i<j)
    for i in 0..n {
        for j in (i + 1)..n {
            obj.add_qterm(-2.0 * penalty_param, x[i], x[j]);
        }
    }
    
    // +2kλ * x_i (from -2k*sum_i x_i)
    for i in 0..n {
        obj.add_term(2.0 * k_f64 * penalty_param, x[i]);
    }
    
    // -λk^2 (constant term, doesn't affect optimization but included for completeness)
    obj.add_constant(-penalty_param * k_f64 * k_f64);

    model.set_objective(obj, Maximize)?;
    model.optimize()?;

    extract_solution(&model, &x, data, k)
}

/// Helper function to extract solution from Gurobi model
fn extract_solution(
    model: &Model,
    x: &[Var],
    data: &MdpData,
    k: usize,
) -> grb::Result<(Vec<usize>, f64)> {
    let status = model.status()?;
    
    if status == Status::Optimal || 
       status == Status::TimeLimit ||
       status == Status::Interrupted {
        
        let mut selected = Vec::new();
        for (i, var) in x.iter().enumerate() {
            let val = model.get_obj_attr(attr::X, var)?;
            if val > 0.5 {
                selected.push(i);
            }
        }

        let actual_diversity = calculate_true_diversity(&selected, data);
        
        // Verify constraint satisfaction
        if selected.len() != k {
            eprintln!("Warning: QUBO selected {} elements, expected {}", selected.len(), k);
        }
        
        Ok((selected, actual_diversity))
    } else {
        Ok((Vec::new(), 0.0))
    }
}

/// Legacy wrapper for backward compatibility
pub fn solve_with_qubo(
    data: &MdpData,
    penalty_param: f64,
    time_limit: f64,
) -> grb::Result<(Vec<usize>, f64)> {
    // Default to constrained version
    solve_with_qubo_constrained(data, time_limit)
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

// use grb::prelude::*;
// use crate::parser::MdpData;
// use grb::expr::QuadExpr;

// pub fn solve_with_qubo(
//     data: &MdpData,
//     penalty_param: f64,
//     time_limit: f64,  // Time limit in seconds
// ) -> grb::Result<(Vec<usize>, f64)> {
//     let mut model = Model::new("MDP_QUBO")?;
//     let n = data.n;
//     let k = data.k;

//     // Set Gurobi parameters for time limit and gap tolerance
//     model.set_param(param::TimeLimit, time_limit)?;
//     model.set_param(param::MIPGap, 0.01)?;  // 1% optimality gap
//     model.set_param(param::OutputFlag, 0)?;  // Suppress output for cleaner logs

//     // ---------------- Variables ----------------
//     let x: Vec<Var> = (0..n)
//         .map(|i| add_binvar!(model, name: &format!("x{}", i)))
//         .collect::<grb::Result<_>>()?;

//     // ---------------- Hard Cardinality Constraint ----------------
//     // OPTION 1: Use this for CONSTRAINED version (recommended)
//     model.add_constr(
//         &format!("cardinality"),
//         c!(x.iter().grb_sum() == k as i32)
//     )?;


//     // ---------------- Objective ----------------
//     let mut obj = QuadExpr::new();

//     // Diversity term: sum_{i<j} d_ij x_i x_j
//     for i in 0..n {
//         for j in (i + 1)..n {
//             let dist = data.get_dist(i, j);
//             if dist != 0.0 {
//                 obj.add_qterm(dist, x[i], x[j]);
//             }
//         }
//     }

//     // OPTION 2: If you want UNCONSTRAINED (penalty-based), comment out the
//     // constraint above and uncomment this penalty term:
    
//     // let k_f64 = k as f64;
//     // for i in 0..n {
//     //     obj.add_term(-penalty_param, x[i]);
//     // }
//     // for i in 0..n {
//     //     for j in (i + 1)..n {
//     //         obj.add_qterm(-2.0 * penalty_param, x[i], x[j]);
//     //     }
//     // }
//     // for i in 0..n {
//     //     obj.add_term(2.0 * k_f64 * penalty_param, x[i]);
//     // }
//     // obj.add_constant(-penalty_param * k_f64 * k_f64);


//     // // Penalty term: -λ (sum_i x_i - k)^2
//     // // Expanded explicitly for binary variables

//     // // -λ * x_i^2 = -λ * x_i
//     // for i in 0..n {
//     //     // This penalty term tries to encourage exactly k selections
//     //     // But it's "soft" - the solver can violate it if beneficial
//     //     obj.add_term(-penalty_param, x[i]);  // Penalty for selecting
//     // }

//     // // -λ * 2 x_i x_j
//     // for i in 0..n {
//     //     for j in (i + 1)..n {
//     //         obj.add_qterm(-2.0 * penalty_param, x[i], x[j]);
//     //     }
//     // }

//     // // -λ * (-2k x_i) = +2kλ x_i
//     // for i in 0..n {
//     //     obj.add_term(2.0 * k * penalty_param, x[i]);
//     // }

//     // // -λ * k^2
//     // obj.add_constant(-penalty_param * k * k);

//     // ---------------- Solve ----------------
//     model.set_objective(obj, Maximize)?;
//     model.optimize()?;

//     // Check if we got a solution (might have timed out)
//     let status = model.status()?;
    
//     // Even if timed out, we can still extract the best solution found
//     if status == Status::Optimal || 
//        status == Status::TimeLimit ||
//        status == Status::Interrupted {
        
//         // ---------------- Extract solution ----------------
//         let mut selected = Vec::new();
//         for i in 0..n {
//             let val = model.get_obj_attr(attr::X, &x[i])?;
//             if val > 0.5 {
//                 selected.push(i);
//             }
//         }

//         let actual_diversity = calculate_true_diversity(&selected, data);

                
//         // Verify constraint is satisfied
//         if selected.len() != k {
//             eprintln!("Warning: QUBO selected {} elements, expected {}", selected.len(), k);
//         }
        

//         Ok((selected, actual_diversity))
//     } else {
//         // Return a simple error - use panic! or just return a default empty solution
//         // Since grb::Error doesn't support creating custom errors easily,
//         // we'll return an empty solution with 0 diversity
//         Ok((Vec::new(), 0.0))
//     }
// }

// // ---------------------------------------------------

// fn calculate_true_diversity(selected: &[usize], data: &MdpData) -> f64 {
//     let mut sum = 0.0;
//     for i in 0..selected.len() {
//         for j in (i + 1)..selected.len() {
//             sum += data.get_dist(selected[i], selected[j]);
//         }
//     }
//     sum
// }