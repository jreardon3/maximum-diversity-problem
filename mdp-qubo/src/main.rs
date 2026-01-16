mod parser;
mod solver_qubo;
mod solver_direct;

use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let path = r"C:\Users\ASUS\Downloads\MDPLIB\mdplib_2010\examples from mdplib\GKD-a\GKD-a_9_n10_m3.txt";
    let data = parser::MdpData::load(path);

    println!("Loaded: n={}, k={}", data.n, data.k);

    let start_g = Instant::now();
    let (indices_g, div_g) = solver_qubo::solve_with_qubo(&data, 1000.0)?;
    let time_g = start_g.elapsed();

    let start_d = Instant::now();
    let (indices_d, div_d) = solver_direct::solve_direct(&data);
    let time_d = start_d.elapsed();

    println!("--- Gurobi QUBO ---");
    println!("Time: {:?}, Diversity: {}, Selected: {:?}", time_g, div_g, indices_g);

    println!("--- Direct Solution ---");
    println!("Time: {:?}, Diversity: {}, Selected: {:?}", time_d, div_d, indices_d);

    Ok(())
}