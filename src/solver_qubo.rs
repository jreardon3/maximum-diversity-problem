use grb::prelude::*;
use crate::parser::MdpData;
use grb::expr::QuadExpr;

pub fn solve_with_qubo(
    data: &MdpData,
    penalty_param: f64,
    time_limit: f64,  // Time limit in seconds
) -> grb::Result<(Vec<usize>, f64)> {
    let mut model = Model::new("MDP_QUBO")?;
    let n = data.n;
    let k = data.k as f64;

    // Set Gurobi parameters for time limit and gap tolerance
    model.set_param(param::TimeLimit, time_limit)?;
    model.set_param(param::MIPGap, 0.01)?;  // 1% optimality gap
    model.set_param(param::OutputFlag, 0)?;  // Suppress output for cleaner logs

    // ---------------- Variables ----------------
    let x: Vec<Var> = (0..n)
        .map(|i| add_binvar!(model, name: &format!("x{}", i)))
        .collect::<grb::Result<_>>()?;

    // ---------------- Objective ----------------
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

    // Penalty term: -λ (sum_i x_i - k)^2
    // Expanded explicitly for binary variables

    // -λ * x_i^2 = -λ * x_i
    for i in 0..n {
        obj.add_term(-penalty_param, x[i]);
    }

    // -λ * 2 x_i x_j
    for i in 0..n {
        for j in (i + 1)..n {
            obj.add_qterm(-2.0 * penalty_param, x[i], x[j]);
        }
    }

    // -λ * (-2k x_i) = +2kλ x_i
    for i in 0..n {
        obj.add_term(2.0 * k * penalty_param, x[i]);
    }

    // -λ * k^2
    obj.add_constant(-penalty_param * k * k);

    // ---------------- Solve ----------------
    model.set_objective(obj, Maximize)?;
    model.optimize()?;

    // Check if we got a solution (might have timed out)
    let status = model.status()?;
    
    // Even if timed out, we can still extract the best solution found
    if status == Status::Optimal || 
       status == Status::TimeLimit ||
       status == Status::Interrupted {
        
        // ---------------- Extract solution ----------------
        let mut selected = Vec::new();
        for i in 0..n {
            let val = model.get_obj_attr(attr::X, &x[i])?;
            if val > 0.5 {
                selected.push(i);
            }
        }

        let actual_diversity = calculate_true_diversity(&selected, data);
        Ok((selected, actual_diversity))
    } else {
        // Return a simple error - use panic! or just return a default empty solution
        // Since grb::Error doesn't support creating custom errors easily,
        // we'll return an empty solution with 0 diversity
        Ok((Vec::new(), 0.0))
    }
}

// ---------------------------------------------------

fn calculate_true_diversity(selected: &[usize], data: &MdpData) -> f64 {
    let mut sum = 0.0;
    for i in 0..selected.len() {
        for j in (i + 1)..selected.len() {
            sum += data.get_dist(selected[i], selected[j]);
        }
    }
    sum
}