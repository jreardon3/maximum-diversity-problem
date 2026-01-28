
// // TODO (Performance Analysis Enhancements)
// // Add more metrics beyond just diversity score and time:
//     // Solution quality gap: Compare heuristic solutions to the optimal/best known solution
//     // Time-to-target: How long to reach X% of best solution
//     // Convergence plots: Show solution quality over time/iterations
//     // Memory usage: Track RAM consumption
//     // Scalability analysis: Plot time/quality vs. instance size
//     // Success rate: Run multiple times with different seeds, report % of runs finding optimal
// // Statistical rigor:
//     // Run each solver multiple times (10-30 runs) with different random seeds
//     // Report mean, median, std dev, min, max
//     // Add confidence intervals

// // TODO!! (Add MaxCut)
// // It's also NP-hard and has similar algorithmic approaches
// // You can use the same heuristics (GRASP, Tabu, GA, Local Search)
// // Gurobi can solve MaxCut via QUBO or ILP formulation
// // Shows your framework generalizes beyond MDP
// // ./your_program --problem maxcut --solver grasp --input graph.txt

// // TODO (Instance Analysis) - Characterize why certain solvers work better on certain instances:
// // Instance features: Size, density, structure
// // Clustering analysis: Group similar instances
// // Performance profiles: Which solver is best on which type?
// // Hardness prediction: Can you predict which instances will be hard?

// // TODO!! (Visualization & Reporting)
// // Performance profiles (like in optimization papers)
// // Scatter plots: Quality vs Time for each solver
// // Heatmaps: Solver performance across instance types
// // Pareto frontier: Time-quality tradeoffs
// // Solution visualization: Show selected subset graphically

// // TODO!! (Practical Recommendations Section)
// // End with guidance like:
// // "Use Gurobi for instances n < 50 when optimality is critical"
// // "Use GRASP for instances 50 < n < 500 for best quality/time tradeoff"
// // "Use First Improvement LS for real-time applications (sub-millisecond)")



// mod parser;
// mod solver_qubo;
// mod solver_grasp;
// mod solver_local_search;
// mod solver_population;

// use std::time::{Instant, Duration};
// use std::fs;
// use std::path::Path;
// use std::collections::HashMap;
// use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
// use solver_grasp::GraspConfig;
// use solver_population::GeneticConfig;

// #[derive(Clone)]
// struct SolverResult {
//     name: String,
//     diversity: f64,
//     time: Duration,
//     success: bool,
// }

// struct InstanceResults {
//     filename: String,
//     n: usize,
//     k: usize,
//     results: Vec<SolverResult>,
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     println!("\n{:=<80}", "");
//     println!("MAXIMUM DIVERSITY PROBLEM - COMPREHENSIVE SOLVER COMPARISON");
//     println!("{:=<80}\n", "");

//     // Discover all test files
//     let base_dir = "examples_from_mdp";
//     let subdirs = vec!["GKD-a", "GKD-b", "MDG-a", "MDG-b", "MDG-c", "SOM-a", "SOM-b"];
    
//     let mut all_results: HashMap<String, Vec<InstanceResults>> = HashMap::new();
    
//     for subdir in subdirs {
//         let dir_path = format!("{}/{}", base_dir, subdir);
        
//         if !Path::new(&dir_path).exists() {
//             println!("Skipping {} (directory not found)", dir_path);
//             continue;
//         }
        
//         println!("\n{:-<80}", "");
//         println!("Processing directory: {}", dir_path);
//         println!("{:-<80}", "");
        
//         let files = discover_test_files(&dir_path)?;
//         println!("Found {} files\n", files.len());
        
//         let category = subdir.split('-').next().unwrap().to_string();
//         all_results.entry(category.clone()).or_insert_with(Vec::new);
        
//         for (idx, path) in files.iter().enumerate() {
//             println!("[{}/{}] Testing: {}", idx + 1, files.len(), path);
            
//             match test_single_file(path) {
//                 Ok(result) => {
//                     all_results.get_mut(&category).unwrap().push(result);
//                 }
//                 Err(_e) => {
//                     println!("  ERROR: Failed to process file\n");
//                 }
//             }
//         }
//     }
    
//     // Print comprehensive summary
//     print_comprehensive_summary(&all_results);
    
//     Ok(())
// }

// fn test_single_file(path: &str) -> Result<InstanceResults, Box<dyn std::error::Error>> {
//     let data = parser::MdpData::load(path);
//     println!("  Size: n={}, k={}", data.n, data.k);
    
//     let filename = Path::new(path)
//         .file_name()
//         .and_then(|s| s.to_str())
//         .unwrap_or(path)
//         .to_string();
    
//     let results = if data.n > 1000 {
//         println!("  (Large instance - using fast solvers only)");
//         run_fast_solvers(&data)
//     } else if data.n > 500 {
//         println!("  (Medium instance - reduced Gurobi time limit)");
//         run_medium_solvers(&data)?
//     } else {
//         println!("  (Small instance - full solver suite)");
//         run_all_solvers(&data)?
//     };
    
//     Ok(InstanceResults {
//         filename,
//         n: data.n,
//         k: data.k,
//         results,
//     })
// }

// fn run_all_solvers(data: &parser::MdpData) -> Result<Vec<SolverResult>, Box<dyn std::error::Error>> {
//     let mut results = Vec::new();

//     // 1. QUBO with time limit
//     print!("  [1/6] QUBO... ");
//     let start = Instant::now();
//     match solver_qubo::solve_with_qubo(data, 1000.0, 300.0) { // 5 min limit
//         Ok((_, div)) => {
//             let time = start.elapsed();
//             println!("✓ {:.2} ({:?})", div, time);
//             results.push(SolverResult {
//                 name: "QUBO (Gurobi)".to_string(),
//                 diversity: div,
//                 time,
//                 success: true,
//             });
//         }
//         Err(e) => {
//             println!("✗ Timeout/Error");
//             results.push(SolverResult {
//                 name: "QUBO (Gurobi)".to_string(),
//                 diversity: 0.0,
//                 time: start.elapsed(),
//                 success: false,
//             });
//         }
//     }

//     // 2. GRASP
//     print!("  [2/6] GRASP... ");
//     let config = GraspConfig {
//         iterations: 50,
//         alpha: 0.3,
//         local_search_iters: 500,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_grasp::solve_grasp(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GRASP".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // 3. First Improvement LS
//     print!("  [3/6] LS: First... ");
//     let config = LocalSearchConfig {
//         method: LocalSearchMethod::FirstImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_local_search::solve_local_search(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS: First".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // 4. Best Improvement LS
//     print!("  [4/6] LS: Best... ");
//     let config = LocalSearchConfig {
//         method: LocalSearchMethod::BestImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_local_search::solve_local_search(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS: Best".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // 5. Tabu Search
//     print!("  [5/6] Tabu... ");
//     let config = LocalSearchConfig {
//         method: LocalSearchMethod::TabuSearch { tabu_tenure: 10 },
//         max_iters: 1000,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_local_search::solve_local_search(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "Tabu".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // 6. Genetic Algorithm
//     print!("  [6/6] GA... ");
//     let config = GeneticConfig {
//         population_size: 30,
//         generations: 50,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 3,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_population::solve_genetic(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GA".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     println!();
//     Ok(results)
// }

// fn run_medium_solvers(data: &parser::MdpData) -> Result<Vec<SolverResult>, Box<dyn std::error::Error>> {
//     let mut results = Vec::new();

//     // QUBO with shorter time limit
//     print!("  [1/4] QUBO... ");
//     let start = Instant::now();
//     match solver_qubo::solve_with_qubo(data, 1000.0, 120.0) { // 2 min limit
//         Ok((_, div)) => {
//             let time = start.elapsed();
//             println!("✓ {:.2} ({:?})", div, time);
//             results.push(SolverResult {
//                 name: "QUBO (Gurobi)".to_string(),
//                 diversity: div,
//                 time,
//                 success: true,
//             });
//         }
//         Err(_) => {
//             println!("✗ Timeout");
//             results.push(SolverResult {
//                 name: "QUBO (Gurobi)".to_string(),
//                 diversity: 0.0,
//                 time: start.elapsed(),
//                 success: false,
//             });
//         }
//     }

//     // Reduced GRASP
//     print!("  [2/4] GRASP... ");
//     let config = GraspConfig {
//         iterations: 30,
//         alpha: 0.3,
//         local_search_iters: 300,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_grasp::solve_grasp(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GRASP".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // Best Improvement
//     print!("  [3/4] LS: Best... ");
//     let config = LocalSearchConfig {
//         method: LocalSearchMethod::BestImprovement,
//         max_iters: 2000,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_local_search::solve_local_search(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS: Best".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // GA
//     print!("  [4/4] GA... ");
//     let config = GeneticConfig {
//         population_size: 20,
//         generations: 30,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 2,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_population::solve_genetic(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GA".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     println!();
//     Ok(results)
// }

// fn run_fast_solvers(data: &parser::MdpData) -> Vec<SolverResult> {
//     let mut results = Vec::new();

//     // GRASP only
//     print!("  [1/3] GRASP... ");
//     let config = GraspConfig {
//         iterations: 20,
//         alpha: 0.3,
//         local_search_iters: 200,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_grasp::solve_grasp(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GRASP".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // First Improvement
//     print!("  [2/3] LS: First... ");
//     let config = LocalSearchConfig {
//         method: LocalSearchMethod::FirstImprovement,
//         max_iters: 1000,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_local_search::solve_local_search(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS: First".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     // GA
//     print!("  [3/3] GA... ");
//     let config = GeneticConfig {
//         population_size: 15,
//         generations: 20,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 2,
//     };
//     let start = Instant::now();
//     let (_, div) = solver_population::solve_genetic(data, &config);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GA".to_string(),
//         diversity: div,
//         time,
//         success: true,
//     });

//     println!();
//     results
// }

// fn discover_test_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//     let mut files = Vec::new();
    
//     for entry in fs::read_dir(dir)? {
//         let entry = entry?;
//         let path = entry.path();
        
//         if path.is_file() {
//             if let Some(ext) = path.extension() {
//                 if ext == "txt" {
//                     if let Some(path_str) = path.to_str() {
//                         files.push(path_str.to_string());
//                     }
//                 }
//             }
//         }
//     }
    
//     files.sort();
//     Ok(files)
// }

// fn print_comprehensive_summary(all_results: &HashMap<String, Vec<InstanceResults>>) {
//     println!("\n\n{:=<100}", "");
//     println!("COMPREHENSIVE RESULTS SUMMARY");
//     println!("{:=<100}\n", "");

//     for category in ["GKD", "MDG", "SOM"] {
//         if let Some(instances) = all_results.get(category) {
//             if instances.is_empty() {
//                 continue;
//             }

//             println!("\n{:-<100}", "");
//             println!("{} INSTANCES ({} files)", category, instances.len());
//             println!("{:-<100}", "");
            
//             // Header
//             println!("{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
//                 "File", "n", "k", "QUBO", "GRASP", "LS:First", "LS:Best", "Tabu", "GA");
//             println!("{:-<100}", "");

//             // Results for each instance
//             for inst in instances {
//                 print!("{:<30} {:>8} {:>6} |", 
//                     truncate_filename(&inst.filename, 30),
//                     inst.n, 
//                     inst.k
//                 );
                
//                 // Find diversity for each solver
//                 let solver_names = ["QUBO (Gurobi)", "GRASP", "LS: First", "LS: Best", "Tabu", "GA"];
//                 for solver in &solver_names {
//                     if let Some(result) = inst.results.iter().find(|r| r.name == *solver) {
//                         if result.success {
//                             print!(" {:>12.2}", result.diversity);
//                         } else {
//                             print!(" {:>12}", "TIMEOUT");
//                         }
//                     } else {
//                         print!(" {:>12}", "-");
//                     }
//                 }
//                 println!();
//             }

//             // Average times per solver
//             println!("\n{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
//                 "Average Time", "", "", "QUBO", "GRASP", "LS:First", "LS:Best", "Tabu", "GA");
//             println!("{:-<100}", "");
            
//             let solver_names = ["QUBO (Gurobi)", "GRASP", "LS: First", "LS: Best", "Tabu", "GA"];
//             print!("{:<30} {:>8} {:>6} |", "", "", "");
            
//             for solver in &solver_names {
//                 let times: Vec<Duration> = instances.iter()
//                     .flat_map(|inst| inst.results.iter())
//                     .filter(|r| r.name == *solver && r.success)
//                     .map(|r| r.time)
//                     .collect();
                
//                 if !times.is_empty() {
//                     let avg_ms = times.iter().map(|t| t.as_millis()).sum::<u128>() / times.len() as u128;
//                     print!(" {:>10}ms", avg_ms);
//                 } else {
//                     print!(" {:>12}", "-");
//                 }
//             }
//             println!("\n");
//         }
//     }

//     // Overall statistics
//     println!("\n{:=<100}", "");
//     println!("OVERALL STATISTICS");
//     println!("{:=<100}", "");
    
//     let total_instances: usize = all_results.values().map(|v| v.len()).sum();
//     println!("Total instances tested: {}", total_instances);
    
//     // Best solver by category
//     for category in ["GKD", "MDG", "SOM"] {
//         if let Some(instances) = all_results.get(category) {
//             if instances.is_empty() {
//                 continue;
//             }
            
//             let mut solver_wins: HashMap<String, usize> = HashMap::new();
            
//             for inst in instances {
//                 if let Some(best) = inst.results.iter()
//                     .filter(|r| r.success)
//                     .max_by(|a, b| a.diversity.partial_cmp(&b.diversity).unwrap()) 
//                 {
//                     *solver_wins.entry(best.name.clone()).or_insert(0) += 1;
//                 }
//             }
            
//             println!("\n{} - Best solver frequency:", category);
//             let mut sorted_wins: Vec<_> = solver_wins.iter().collect();
//             sorted_wins.sort_by(|a, b| b.1.cmp(a.1));
//             for (solver, wins) in sorted_wins {
//                 println!("  {:<20} {:>3} wins ({:.1}%)", solver, wins, 
//                     (*wins as f64 / instances.len() as f64) * 100.0);
//             }
//         }
//     }
    
//     println!("\n{:=<100}\n", "");
// }

// fn truncate_filename(filename: &str, max_len: usize) -> String {
//     if filename.len() <= max_len {
//         filename.to_string()
//     } else {
//         format!("...{}", &filename[filename.len() - (max_len - 3)..])
//     }
// }


mod parser;
mod solver_qubo;
mod solver_grasp;
mod solver_local_search;
mod solver_population;

use std::time::Instant;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
use solver_grasp::GraspConfig;
use solver_population::GeneticConfig;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
struct SolverResult {
    name: String,
    diversity: f64,
    time_ms: u128,
    success: bool,
}

#[derive(Serialize, Deserialize)]
struct InstanceResults {
    filename: String,
    category: String,
    n: usize,
    k: usize,
    results: Vec<SolverResult>,
}

#[derive(Serialize, Deserialize)]
struct ExperimentResults {
    timestamp: String,
    instances: Vec<InstanceResults>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{:=<80}", "");
    println!("MAXIMUM DIVERSITY PROBLEM - COMPREHENSIVE SOLVER COMPARISON");
    println!("{:=<80}\n", "");

    // Discover all test files
    let base_dir = "examples_from_mdp";
    let subdirs = vec!["GKD-a", "GKD-b", "MDG-a", "MDG-b", "MDG-c", "SOM-a", "SOM-b"];
    
    let mut all_instances: Vec<InstanceResults> = Vec::new();
    
    for subdir in subdirs {
        let dir_path = format!("{}/{}", base_dir, subdir);
        
        if !Path::new(&dir_path).exists() {
            println!("Skipping {} (directory not found)", dir_path);
            continue;
        }
        
        println!("\n{:-<80}", "");
        println!("Processing directory: {}", dir_path);
        println!("{:-<80}", "");
        
        let files = discover_test_files(&dir_path)?;
        println!("Found {} files\n", files.len());
        
        let category = subdir.split('-').next().unwrap().to_string();
        
        for (idx, path) in files.iter().enumerate() {
            println!("[{}/{}] Testing: {}", idx + 1, files.len(), path);
            
            match test_single_file(path, &category) {
                Ok(result) => {
                    all_instances.push(result);
                }
                Err(_e) => {
                    println!("  ERROR: Failed to process file\n");
                }
            }
        }
    }
    
    // Save results to JSON
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let results = ExperimentResults {
        timestamp: timestamp.clone(),
        instances: all_instances,
    };
    
    let json_file = format!("results_{}.json", timestamp);
    save_results_to_json(&results, &json_file)?;
    println!("\n✓ Results saved to: {}", json_file);
    
    // Print comprehensive summary
    print_comprehensive_summary(&results);
    
    // Generate visualization script
    generate_visualization_script(&json_file)?;
    println!("\n✓ Visualization script saved to: visualize_results.py");
    println!("  Run with: python visualize_results.py {}", json_file);
    
    Ok(())
}

fn test_single_file(path: &str, category: &str) -> Result<InstanceResults, Box<dyn std::error::Error>> {
    let data = parser::MdpData::load(path);
    println!("  Size: n={}, k={}", data.n, data.k);
    
    let filename = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();
    
    let results = if data.n > 1000 {
        println!("  (Large instance - using fast solvers only)");
        run_fast_solvers(&data)
    } else if data.n > 500 {
        println!("  (Medium instance - reduced Gurobi time limit)");
        run_medium_solvers(&data)?
    } else {
        println!("  (Small instance - full solver suite)");
        run_all_solvers(&data)?
    };
    
    Ok(InstanceResults {
        filename,
        category: category.to_string(),
        n: data.n,
        k: data.k,
        results,
    })
}

fn run_all_solvers(data: &parser::MdpData) -> Result<Vec<SolverResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // 1. QUBO with time limit
    print!("  [1/6] QUBO... ");
    let start = Instant::now();
    match solver_qubo::solve_with_qubo(data, 1000.0, 300.0) {
        Ok((_, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: div,
                time_ms: time.as_millis(),
                success: div > 0.0,
            });
        }
        Err(_) => {
            println!("✗ Timeout/Error");
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: 0.0,
                time_ms: start.elapsed().as_millis(),
                success: false,
            });
        }
    }

    // 2. GRASP
    print!("  [2/6] GRASP... ");
    let config = GraspConfig {
        iterations: 50,
        alpha: 0.3,
        local_search_iters: 500,
    };
    let start = Instant::now();
    let (_, div) = solver_grasp::solve_grasp(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GRASP".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    // 3. First Improvement LS
    print!("  [3/6] LS: First... ");
    let config = LocalSearchConfig {
        method: LocalSearchMethod::FirstImprovement,
        max_iters: 5000,
    };
    let start = Instant::now();
    let (_, div) = solver_local_search::solve_local_search(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-First".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    // 4. Best Improvement LS
    print!("  [4/6] LS: Best... ");
    let config = LocalSearchConfig {
        method: LocalSearchMethod::BestImprovement,
        max_iters: 5000,
    };
    let start = Instant::now();
    let (_, div) = solver_local_search::solve_local_search(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-Best".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    // 5. Tabu Search
    print!("  [5/6] Tabu... ");
    let config = LocalSearchConfig {
        method: LocalSearchMethod::TabuSearch { tabu_tenure: 10 },
        max_iters: 1000,
    };
    let start = Instant::now();
    let (_, div) = solver_local_search::solve_local_search(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "Tabu".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    // 6. Genetic Algorithm
    print!("  [6/6] GA... ");
    let config = GeneticConfig {
        population_size: 30,
        generations: 50,
        crossover_rate: 0.8,
        mutation_rate: 0.15,
        elite_size: 3,
    };
    let start = Instant::now();
    let (_, div) = solver_population::solve_genetic(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GA".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    println!();
    Ok(results)
}

fn run_medium_solvers(data: &parser::MdpData) -> Result<Vec<SolverResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    print!("  [1/4] QUBO... ");
    let start = Instant::now();
    match solver_qubo::solve_with_qubo(data, 1000.0, 120.0) {
        Ok((_, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: div,
                time_ms: time.as_millis(),
                success: div > 0.0,
            });
        }
        Err(_) => {
            println!("✗ Timeout");
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: 0.0,
                time_ms: start.elapsed().as_millis(),
                success: false,
            });
        }
    }

    print!("  [2/4] GRASP... ");
    let config = GraspConfig {
        iterations: 30,
        alpha: 0.3,
        local_search_iters: 300,
    };
    let start = Instant::now();
    let (_, div) = solver_grasp::solve_grasp(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GRASP".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    print!("  [3/4] LS: Best... ");
    let config = LocalSearchConfig {
        method: LocalSearchMethod::BestImprovement,
        max_iters: 2000,
    };
    let start = Instant::now();
    let (_, div) = solver_local_search::solve_local_search(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-Best".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    print!("  [4/4] GA... ");
    let config = GeneticConfig {
        population_size: 20,
        generations: 30,
        crossover_rate: 0.8,
        mutation_rate: 0.15,
        elite_size: 2,
    };
    let start = Instant::now();
    let (_, div) = solver_population::solve_genetic(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GA".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    println!();
    Ok(results)
}

fn run_fast_solvers(data: &parser::MdpData) -> Vec<SolverResult> {
    let mut results = Vec::new();

    print!("  [1/3] GRASP... ");
    let config = GraspConfig {
        iterations: 20,
        alpha: 0.3,
        local_search_iters: 200,
    };
    let start = Instant::now();
    let (_, div) = solver_grasp::solve_grasp(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GRASP".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    print!("  [2/3] LS: First... ");
    let config = LocalSearchConfig {
        method: LocalSearchMethod::FirstImprovement,
        max_iters: 1000,
    };
    let start = Instant::now();
    let (_, div) = solver_local_search::solve_local_search(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-First".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    print!("  [3/3] GA... ");
    let config = GeneticConfig {
        population_size: 15,
        generations: 20,
        crossover_rate: 0.8,
        mutation_rate: 0.15,
        elite_size: 2,
    };
    let start = Instant::now();
    let (_, div) = solver_population::solve_genetic(data, &config);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GA".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
    });

    println!();
    results
}

fn discover_test_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "txt" {
                    if let Some(path_str) = path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    files.sort();
    Ok(files)
}

fn save_results_to_json(results: &ExperimentResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(results)?;
    let mut file = File::create(filename)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn print_comprehensive_summary(results: &ExperimentResults) {
    let mut by_category: HashMap<String, Vec<&InstanceResults>> = HashMap::new();
    
    for instance in &results.instances {
        by_category.entry(instance.category.clone())
            .or_insert_with(Vec::new)
            .push(instance);
    }

    println!("\n\n{:=<100}", "");
    println!("COMPREHENSIVE RESULTS SUMMARY");
    println!("{:=<100}\n", "");

    for category in ["GKD", "MDG", "SOM"] {
        if let Some(instances) = by_category.get(category) {
            if instances.is_empty() {
                continue;
            }

            println!("\n{:-<100}", "");
            println!("{} INSTANCES ({} files)", category, instances.len());
            println!("{:-<100}", "");
            
            println!("{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
                "File", "n", "k", "QUBO", "GRASP", "LS-First", "LS-Best", "Tabu", "GA");
            println!("{:-<100}", "");

            for inst in instances {
                print!("{:<30} {:>8} {:>6} |", 
                    truncate_filename(&inst.filename, 30),
                    inst.n, 
                    inst.k
                );
                
                let solver_names = ["QUBO", "GRASP", "LS-First", "LS-Best", "Tabu", "GA"];
                for solver in &solver_names {
                    if let Some(result) = inst.results.iter().find(|r| r.name == *solver) {
                        if result.success {
                            print!(" {:>12.2}", result.diversity);
                        } else {
                            print!(" {:>12}", "TIMEOUT");
                        }
                    } else {
                        print!(" {:>12}", "-");
                    }
                }
                println!();
            }

            println!("\n{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
                "Average Time (ms)", "", "", "QUBO", "GRASP", "LS-First", "LS-Best", "Tabu", "GA");
            println!("{:-<100}", "");
            
            let solver_names = ["QUBO", "GRASP", "LS-First", "LS-Best", "Tabu", "GA"];
            print!("{:<30} {:>8} {:>6} |", "", "", "");
            
            for solver in &solver_names {
                let times: Vec<u128> = instances.iter()
                    .flat_map(|inst| inst.results.iter())
                    .filter(|r| r.name == *solver && r.success)
                    .map(|r| r.time_ms)
                    .collect();
                
                if !times.is_empty() {
                    let avg_ms = times.iter().sum::<u128>() / times.len() as u128;
                    print!(" {:>12}", avg_ms);
                } else {
                    print!(" {:>12}", "-");
                }
            }
            println!("\n");
        }
    }

    println!("\n{:=<100}\n", "");
}

fn truncate_filename(filename: &str, max_len: usize) -> String {
    if filename.len() <= max_len {
        filename.to_string()
    } else {
        format!("...{}", &filename[filename.len() - (max_len - 3)..])
    }
}

fn generate_visualization_script(_json_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"#!/usr/bin/env python3
"""
Visualization script for MDP solver comparison results.
Generated automatically by the Rust benchmark program.

Usage: python visualize_results.py <results.json>
"""

import json
import sys
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np
from pathlib import Path

# Set style
sns.set_style("whitegrid")
plt.rcParams['figure.figsize'] = (12, 8)

def load_results(json_file):
    with open(json_file, 'r') as f:
        return json.load(f)

def create_scatter_plot(df, output_dir):
    """Quality vs Time scatter plot for each solver"""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    solvers = df['solver'].unique()
    colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
    for solver, color in zip(solvers, colors):
        solver_data = df[df['solver'] == solver]
        ax.scatter(solver_data['time_ms'], solver_data['diversity'], 
                  label=solver, alpha=0.6, s=100, color=color)
    
    ax.set_xlabel('Time (ms)', fontsize=12)
    ax.set_ylabel('Diversity Score', fontsize=12)
    ax.set_title('Solution Quality vs Computation Time', fontsize=14, fontweight='bold')
    ax.set_xscale('log')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'scatter_quality_vs_time.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: scatter_quality_vs_time.png")
    plt.close()

def create_heatmap(df, output_dir):
    """Heatmap of solver performance across instance types"""
    pivot_data = df.groupby(['category', 'solver'])['diversity'].mean().unstack(fill_value=0)
    
    fig, ax = plt.subplots(figsize=(10, 6))
    sns.heatmap(pivot_data, annot=True, fmt='.1f', cmap='YlOrRd', 
                cbar_kws={'label': 'Avg Diversity'}, ax=ax)
    ax.set_title('Average Solver Performance by Instance Category', fontsize=14, fontweight='bold')
    ax.set_xlabel('Solver', fontsize=12)
    ax.set_ylabel('Instance Category', fontsize=12)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'heatmap_performance.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: heatmap_performance.png")
    plt.close()

def create_pareto_frontier(df, output_dir):
    """Pareto frontier showing time-quality tradeoffs"""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    categories = df['category'].unique()
    colors = plt.cm.tab10(np.linspace(0, 1, len(categories)))
    
    for category, color in zip(categories, colors):
        cat_data = df[df['category'] == category]
        solver_avgs = cat_data.groupby('solver').agg({'time_ms': 'mean', 'diversity': 'mean'}).reset_index()
        
        ax.scatter(solver_avgs['time_ms'], solver_avgs['diversity'], 
                  label=category, alpha=0.7, s=150, color=color, edgecolors='black', linewidth=1.5)
        
        for _, row in solver_avgs.iterrows():
            ax.annotate(row['solver'], 
                       (row['time_ms'], row['diversity']),
                       xytext=(5, 5), textcoords='offset points', fontsize=8)
    
    ax.set_xlabel('Average Time (ms)', fontsize=12)
    ax.set_ylabel('Average Diversity Score', fontsize=12)
    ax.set_title('Pareto Frontier: Time-Quality Tradeoffs by Category', fontsize=14, fontweight='bold')
    ax.set_xscale('log')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'pareto_frontier.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: pareto_frontier.png")
    plt.close()

def create_box_plots(df, output_dir):
    """Box plots showing distribution of results"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
    df_success = df[df['success'] == True]
    df_success.boxplot(column='diversity', by='solver', ax=ax1)
    ax1.set_title('Distribution of Diversity Scores by Solver', fontsize=12, fontweight='bold')
    ax1.set_xlabel('Solver', fontsize=11)
    ax1.set_ylabel('Diversity Score', fontsize=11)
    plt.sca(ax1)
    plt.xticks(rotation=45)
    
    df_success.boxplot(column='time_ms', by='solver', ax=ax2)
    ax2.set_title('Distribution of Computation Times by Solver', fontsize=12, fontweight='bold')
    ax2.set_xlabel('Solver', fontsize=11)
    ax2.set_ylabel('Time (ms)', fontsize=11)
    ax2.set_yscale('log')
    plt.sca(ax2)
    plt.xticks(rotation=45)
    
    plt.suptitle('')
    plt.tight_layout()
    plt.savefig(output_dir / 'boxplot_distributions.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: boxplot_distributions.png")
    plt.close()

def create_scaling_plot(df, output_dir):
    """How solvers scale with problem size"""
    fig, ax = plt.subplots(figsize=(12, 8))
    
    solvers = df['solver'].unique()
    colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
    for solver, color in zip(solvers, colors):
        solver_data = df[df['solver'] == solver].copy()
        solver_data = solver_data.sort_values('n')
        grouped = solver_data.groupby('n')['time_ms'].mean().reset_index()
        
        ax.plot(grouped['n'], grouped['time_ms'], 
               marker='o', label=solver, color=color, linewidth=2, markersize=8)
    
    ax.set_xlabel('Problem Size (n)', fontsize=12)
    ax.set_ylabel('Average Time (ms)', fontsize=12)
    ax.set_title('Solver Scalability: Time vs Problem Size', fontsize=14, fontweight='bold')
    ax.set_yscale('log')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'scaling_analysis.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: scaling_analysis.png")
    plt.close()

def create_win_rate_chart(df, output_dir):
    """Bar chart showing which solver wins most often"""
    df_success = df[df['success'] == True].copy()
    best_solvers = df_success.loc[df_success.groupby('filename')['diversity'].idxmax(), 'solver']
    win_counts = best_solvers.value_counts()
    
    fig, ax = plt.subplots(figsize=(10, 6))
    win_counts.plot(kind='bar', ax=ax, color='steelblue', edgecolor='black')
    ax.set_title('Solver Win Frequency (Best Diversity Score)', fontsize=14, fontweight='bold')
    ax.set_xlabel('Solver', fontsize=12)
    ax.set_ylabel('Number of Instances Won', fontsize=12)
    ax.set_xticklabels(ax.get_xticklabels(), rotation=45, ha='right')
    
    for i, v in enumerate(win_counts):
        ax.text(i, v + 0.5, str(v), ha='center', va='bottom', fontweight='bold')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'win_rate.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: win_rate.png")
    plt.close()

def main():
    if len(sys.argv) < 2:
        print("Usage: python visualize_results.py <results.json>")
        sys.exit(1)
    
    json_file = sys.argv[1]
    
    print(f"\nLoading results from: {json_file}")
    data = load_results(json_file)
    
    rows = []
    for instance in data['instances']:
        for result in instance['results']:
            rows.append({
                'filename': instance['filename'],
                'category': instance['category'],
                'n': instance['n'],
                'k': instance['k'],
                'solver': result['name'],
                'diversity': result['diversity'],
                'time_ms': result['time_ms'],
                'success': result['success']
            })
    
    df = pd.DataFrame(rows)
    
    print(f"Loaded {len(df)} result records from {len(data['instances'])} instances")
    print(f"Solvers: {', '.join(df['solver'].unique())}")
    print(f"Categories: {', '.join(df['category'].unique())}\n")
    
    timestamp = data['timestamp']
    output_dir = Path(f'visualizations_{timestamp}')
    output_dir.mkdir(exist_ok=True)
    
    print(f"Generating visualizations in: {output_dir}/\n")
    
    create_scatter_plot(df, output_dir)
    create_heatmap(df, output_dir)
    create_pareto_frontier(df, output_dir)
    create_box_plots(df, output_dir)
    create_scaling_plot(df, output_dir)
    create_win_rate_chart(df, output_dir)
    
    print(f"\n✓ All visualizations saved to: {output_dir}/")
    print(f"  Total: 6 plots generated")

if __name__ == '__main__':
    main()
"#;
    
    let mut file = File::create("visualize_results.py")?;
    file.write_all(script.as_bytes())?;
    Ok(())
}