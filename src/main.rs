// mod parser;
// mod solver_qubo;
// mod solver_grasp;
// mod solver_local_search;
// mod solver_population;

// use std::time::Instant;
// use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
// use solver_grasp::GraspConfig;
// use solver_population::GeneticConfig;

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

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Test on multiple instances
//     let test_files = vec![
//         // TODO!! - run through all files in folder
//         // TODO!! - add a quality or time cap to bigger files 
//         // Add time limits: Kill Gurobi after 5 minutes, report best solution found
//         // Use MIP gap tolerance: Tell Gurobi to stop at 1% optimality gap
//         "examples_from_mdp/GKD-a/GKD-a_9_n10_m3.txt",
//         "examples_from_mdp/GKD-b/GKD-b_6_n25_m7.txt",
//         "examples_from_mdp/GKD-b/GKD-b_15_n50_m5.txt",
//         "examples_from_mdp/SOM-a/SOM-a_2_n25_m2.txt",
//         "examples_from_mdp/SOM-a/SOM-a_15_n50_m5.txt",
//     ];

//     for path in test_files {
//         // println!("\n{'=':<60}", "");
//         println!("Testing: {}", path);
//         println!("{:=<60}\n", "");
        
//         let data = parser::MdpData::load(path);
//         println!("Problem size: n={}, k={}\n", data.n, data.k);
        
//         run_all_solvers(&data)?;
//     }

//     Ok(())
// }

// fn run_all_solvers(data: &parser::MdpData) -> Result<(), Box<dyn std::error::Error>> {
//     let mut results = Vec::new();

//     // 1. QUBO-based (Gurobi)
//     println!("Running QUBO (Gurobi)...");
//     let start = Instant::now();
//     let (indices, div) = solver_qubo::solve_with_qubo(data, 1000.0)?;
//     let time = start.elapsed();
//     results.push(("QUBO (Gurobi)", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     // 2. GRASP-based methods
//     println!("\nRunning GRASP...");
//     let grasp_config = GraspConfig {
//         iterations: 50,
//         alpha: 0.3,
//         local_search_iters: 500,
//     };
//     let start = Instant::now();
//     let (indices, div) = solver_grasp::solve_grasp(data, &grasp_config);
//     let time = start.elapsed();
//     results.push(("GRASP", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     // 3. Local Search-based methods
//     println!("\nRunning Local Search (First Improvement)...");
//     let ls_config_first = LocalSearchConfig {
//         method: LocalSearchMethod::FirstImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (indices, div) = solver_local_search::solve_local_search(data, &ls_config_first);
//     let time = start.elapsed();
//     results.push(("LS: First Improvement", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     println!("\nRunning Local Search (Best Improvement)...");
//     let ls_config_best = LocalSearchConfig {
//         method: LocalSearchMethod::BestImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (indices, div) = solver_local_search::solve_local_search(data, &ls_config_best);
//     let time = start.elapsed();
//     results.push(("LS: Best Improvement", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     println!("\nRunning Tabu Search...");
//     let ls_config_tabu = LocalSearchConfig {
//         method: LocalSearchMethod::TabuSearch { tabu_tenure: 10 },
//         max_iters: 1000,
//     };
//     let start = Instant::now();
//     let (indices, div) = solver_local_search::solve_local_search(data, &ls_config_tabu);
//     let time = start.elapsed();
//     results.push(("Tabu Search", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     // 4. Population-based methods
//     println!("\nRunning Genetic Algorithm...");
//     let ga_config = GeneticConfig {
//         population_size: 30,
//         generations: 50,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 3,
//     };
//     let start = Instant::now();
//     let (indices, div) = solver_population::solve_genetic(data, &ga_config);
//     let time = start.elapsed();
//     results.push(("Genetic Algorithm", div, time));
//     println!("  Time: {:?}, Diversity: {:.2}", time, div);

//     // Print summary comparison
//     println!("\n{:-<60}", "");
//     println!("SUMMARY");
//     println!("{:-<60}", "");
//     results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
//     for (rank, (name, div, time)) in results.iter().enumerate() {
//         println!("{}. {:25} Diversity: {:12.2}  Time: {:?}", 
//             rank + 1, name, div, time);
//     }

//     Ok(())
// }








mod parser;
mod solver_qubo;
mod solver_grasp;
mod solver_local_search;
mod solver_population;

use std::time::{Instant, Duration};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
use solver_grasp::GraspConfig;
use solver_population::GeneticConfig;

#[derive(Clone)]
struct SolverResult {
    name: String,
    diversity: f64,
    time: Duration,
    success: bool,
}

struct InstanceResults {
    filename: String,
    n: usize,
    k: usize,
    results: Vec<SolverResult>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{:=<80}", "");
    println!("MAXIMUM DIVERSITY PROBLEM - COMPREHENSIVE SOLVER COMPARISON");
    println!("{:=<80}\n", "");

    // Discover all test files
    let base_dir = "examples_from_mdp";
    let subdirs = vec!["GKD-a", "GKD-b", "MDG-a", "MDG-b", "MDG-c", "SOM-a", "SOM-b"];
    
    let mut all_results: HashMap<String, Vec<InstanceResults>> = HashMap::new();
    
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
        all_results.entry(category.clone()).or_insert_with(Vec::new);
        
        for (idx, path) in files.iter().enumerate() {
            println!("[{}/{}] Testing: {}", idx + 1, files.len(), path);
            
            match test_single_file(path) {
                Ok(result) => {
                    all_results.get_mut(&category).unwrap().push(result);
                }
                Err(_e) => {
                    println!("  ERROR: Failed to process file\n");
                }
            }
        }
    }
    
    // Print comprehensive summary
    print_comprehensive_summary(&all_results);
    
    Ok(())
}

fn test_single_file(path: &str) -> Result<InstanceResults, Box<dyn std::error::Error>> {
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
    match solver_qubo::solve_with_qubo(data, 1000.0, 300.0) { // 5 min limit
        Ok((_, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            results.push(SolverResult {
                name: "QUBO (Gurobi)".to_string(),
                diversity: div,
                time,
                success: true,
            });
        }
        Err(e) => {
            println!("✗ Timeout/Error");
            results.push(SolverResult {
                name: "QUBO (Gurobi)".to_string(),
                diversity: 0.0,
                time: start.elapsed(),
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
        time,
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
        name: "LS: First".to_string(),
        diversity: div,
        time,
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
        name: "LS: Best".to_string(),
        diversity: div,
        time,
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
        time,
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
        time,
        success: true,
    });

    println!();
    Ok(results)
}

fn run_medium_solvers(data: &parser::MdpData) -> Result<Vec<SolverResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // QUBO with shorter time limit
    print!("  [1/4] QUBO... ");
    let start = Instant::now();
    match solver_qubo::solve_with_qubo(data, 1000.0, 120.0) { // 2 min limit
        Ok((_, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            results.push(SolverResult {
                name: "QUBO (Gurobi)".to_string(),
                diversity: div,
                time,
                success: true,
            });
        }
        Err(_) => {
            println!("✗ Timeout");
            results.push(SolverResult {
                name: "QUBO (Gurobi)".to_string(),
                diversity: 0.0,
                time: start.elapsed(),
                success: false,
            });
        }
    }

    // Reduced GRASP
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
        time,
        success: true,
    });

    // Best Improvement
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
        name: "LS: Best".to_string(),
        diversity: div,
        time,
        success: true,
    });

    // GA
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
        time,
        success: true,
    });

    println!();
    Ok(results)
}

fn run_fast_solvers(data: &parser::MdpData) -> Vec<SolverResult> {
    let mut results = Vec::new();

    // GRASP only
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
        time,
        success: true,
    });

    // First Improvement
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
        name: "LS: First".to_string(),
        diversity: div,
        time,
        success: true,
    });

    // GA
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
        time,
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

fn print_comprehensive_summary(all_results: &HashMap<String, Vec<InstanceResults>>) {
    println!("\n\n{:=<100}", "");
    println!("COMPREHENSIVE RESULTS SUMMARY");
    println!("{:=<100}\n", "");

    for category in ["GKD", "MDG", "SOM"] {
        if let Some(instances) = all_results.get(category) {
            if instances.is_empty() {
                continue;
            }

            println!("\n{:-<100}", "");
            println!("{} INSTANCES ({} files)", category, instances.len());
            println!("{:-<100}", "");
            
            // Header
            println!("{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
                "File", "n", "k", "QUBO", "GRASP", "LS:First", "LS:Best", "Tabu", "GA");
            println!("{:-<100}", "");

            // Results for each instance
            for inst in instances {
                print!("{:<30} {:>8} {:>6} |", 
                    truncate_filename(&inst.filename, 30),
                    inst.n, 
                    inst.k
                );
                
                // Find diversity for each solver
                let solver_names = ["QUBO (Gurobi)", "GRASP", "LS: First", "LS: Best", "Tabu", "GA"];
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

            // Average times per solver
            println!("\n{:<30} {:>8} {:>6} | {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
                "Average Time", "", "", "QUBO", "GRASP", "LS:First", "LS:Best", "Tabu", "GA");
            println!("{:-<100}", "");
            
            let solver_names = ["QUBO (Gurobi)", "GRASP", "LS: First", "LS: Best", "Tabu", "GA"];
            print!("{:<30} {:>8} {:>6} |", "", "", "");
            
            for solver in &solver_names {
                let times: Vec<Duration> = instances.iter()
                    .flat_map(|inst| inst.results.iter())
                    .filter(|r| r.name == *solver && r.success)
                    .map(|r| r.time)
                    .collect();
                
                if !times.is_empty() {
                    let avg_ms = times.iter().map(|t| t.as_millis()).sum::<u128>() / times.len() as u128;
                    print!(" {:>10}ms", avg_ms);
                } else {
                    print!(" {:>12}", "-");
                }
            }
            println!("\n");
        }
    }

    // Overall statistics
    println!("\n{:=<100}", "");
    println!("OVERALL STATISTICS");
    println!("{:=<100}", "");
    
    let total_instances: usize = all_results.values().map(|v| v.len()).sum();
    println!("Total instances tested: {}", total_instances);
    
    // Best solver by category
    for category in ["GKD", "MDG", "SOM"] {
        if let Some(instances) = all_results.get(category) {
            if instances.is_empty() {
                continue;
            }
            
            let mut solver_wins: HashMap<String, usize> = HashMap::new();
            
            for inst in instances {
                if let Some(best) = inst.results.iter()
                    .filter(|r| r.success)
                    .max_by(|a, b| a.diversity.partial_cmp(&b.diversity).unwrap()) 
                {
                    *solver_wins.entry(best.name.clone()).or_insert(0) += 1;
                }
            }
            
            println!("\n{} - Best solver frequency:", category);
            let mut sorted_wins: Vec<_> = solver_wins.iter().collect();
            sorted_wins.sort_by(|a, b| b.1.cmp(a.1));
            for (solver, wins) in sorted_wins {
                println!("  {:<20} {:>3} wins ({:.1}%)", solver, wins, 
                    (*wins as f64 / instances.len() as f64) * 100.0);
            }
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











// FOR BATCH TESTING ALL TEST FILES

// mod parser;
// mod solver_qubo;
// mod solver_grasp;
// mod solver_local_search;
// mod solver_population;

// use std::time::Instant;
// use std::fs;
// use std::path::Path;
// use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
// use solver_grasp::GraspConfig;
// use solver_population::GeneticConfig;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Print current working directory for debugging
//     println!("Current working directory: {:?}\n", std::env::current_dir()?);

//     // Test single file first to make sure everything works
//     let single_test = "examples_from_mdp/GKD-a/GKD-a_13_n10_m4.txt";
    
//     if Path::new(single_test).exists() {
//         println!("Testing single file: {}\n", single_test);
//         test_single_file(single_test)?;
//     } else {
//         println!("Single test file not found: {}", single_test);
//         println!("Available files in examples_from_mdp/GKD-a/:");
//         list_directory_contents("examples_from_mdp/GKD-a")?;
//         return Ok(());
//     }

//     // Batch test all files in a directory
//     println!("\n\n{:=<80}", "");
//     println!("BATCH TESTING ALL FILES");
//     println!("{:=<80}\n", "");
    
//     // Test all directories
//     let test_dirs = vec![
//         "examples_from_mdp/GKD-a",
//         "examples_from_mdp/MDG-c",
//     ];

//     for dir in test_dirs {
//         if Path::new(dir).exists() {
//             println!("\nTesting directory: {}", dir);
//             let files = discover_test_files(dir)?;
//             println!("Found {} files\n", files.len());
            
//             for path in files {
//                 test_single_file(&path)?;
//             }
//         } else {
//             println!("Directory not found: {}", dir);
//         }
//     }

//     Ok(())
// }

// fn test_single_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
//     println!("\n{:=<60}", "");
//     println!("File: {}", path);
//     println!("{:=<60}", "");
    
//     let data = parser::MdpData::load(path);
//     println!("Problem: n={}, k={}", data.n, data.k);
    
//     // For large instances, use faster settings
//     let is_large = data.n > 500;
    
//     if is_large {
//         println!("(Large instance - using reduced parameters)");
//         run_solvers_fast(&data)?;
//     } else {
//         println!("(Small instance - using full parameters)");
//         run_all_solvers(&data)?;
//     }
    
//     Ok(())
// }

// fn run_solvers_fast(data: &parser::MdpData) -> Result<(), Box<dyn std::error::Error>> {
//     let mut results = Vec::new();

//     // Only run fast methods on large instances
    
//     // GRASP with reduced iterations
//     println!("\n[1/3] Running GRASP...");
//     let grasp_config = GraspConfig {
//         iterations: 20,
//         alpha: 0.3,
//         local_search_iters: 200,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_grasp::solve_grasp(data, &grasp_config);
//     let time = start.elapsed();
//     results.push(("GRASP", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // Best improvement local search
//     println!("\n[2/3] Running Local Search...");
//     let ls_config = LocalSearchConfig {
//         method: LocalSearchMethod::BestImprovement,
//         max_iters: 1000,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_local_search::solve_local_search(data, &ls_config);
//     let time = start.elapsed();
//     results.push(("Local Search", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // Genetic Algorithm with smaller population
//     println!("\n[3/3] Running Genetic Algorithm...");
//     let ga_config = GeneticConfig {
//         population_size: 20,
//         generations: 30,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 2,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_population::solve_genetic(data, &ga_config);
//     let time = start.elapsed();
//     results.push(("Genetic Algorithm", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     print_summary(&results);
//     Ok(())
// }

// fn run_all_solvers(data: &parser::MdpData) -> Result<(), Box<dyn std::error::Error>> {
//     let mut results = Vec::new();

//     // 1. QUBO (skip for very large instances as Gurobi might be slow)
//     if data.n <= 1000 {
//         println!("\n[1/7] Running QUBO (Gurobi)...");
//         let start = Instant::now();
//         match solver_qubo::solve_with_qubo(data, 1000.0) {
//             Ok((_indices, div)) => {
//                 let time = start.elapsed();
//                 results.push(("QUBO (Gurobi)", div, time));
//                 println!("      Diversity: {:.2}, Time: {:?}", div, time);
//             }
//             Err(e) => {
//                 println!("      Failed: {}", e);
//             }
//         }
//     }

//     // 2. GRASP
//     println!("\n[2/7] Running GRASP...");
//     let grasp_config = GraspConfig {
//         iterations: 50,
//         alpha: 0.3,
//         local_search_iters: 500,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_grasp::solve_grasp(data, &grasp_config);
//     let time = start.elapsed();
//     results.push(("GRASP", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // 3. First Improvement LS
//     println!("\n[3/7] Running LS: First Improvement...");
//     let ls_config = LocalSearchConfig {
//         method: LocalSearchMethod::FirstImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_local_search::solve_local_search(data, &ls_config);
//     let time = start.elapsed();
//     results.push(("LS: First Improvement", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // 4. Best Improvement LS
//     println!("\n[4/7] Running LS: Best Improvement...");
//     let ls_config = LocalSearchConfig {
//         method: LocalSearchMethod::BestImprovement,
//         max_iters: 5000,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_local_search::solve_local_search(data, &ls_config);
//     let time = start.elapsed();
//     results.push(("LS: Best Improvement", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // 5. Tabu Search
//     println!("\n[5/7] Running Tabu Search...");
//     let ls_config = LocalSearchConfig {
//         method: LocalSearchMethod::TabuSearch { tabu_tenure: 10 },
//         max_iters: 1000,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_local_search::solve_local_search(data, &ls_config);
//     let time = start.elapsed();
//     results.push(("Tabu Search", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     // 6. Genetic Algorithm
//     println!("\n[6/7] Running Genetic Algorithm...");
//     let ga_config = GeneticConfig {
//         population_size: 30,
//         generations: 50,
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 3,
//     };
//     let start = Instant::now();
//     let (_indices, div) = solver_population::solve_genetic(data, &ga_config);
//     let time = start.elapsed();
//     results.push(("Genetic Algorithm", div, time));
//     println!("      Diversity: {:.2}, Time: {:?}", div, time);

//     print_summary(&results);
//     Ok(())
// }

// fn print_summary(results: &Vec<(&str, f64, std::time::Duration)>) {
//     let mut sorted_results = results.clone();
//     sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
//     println!("\n{:-<60}", "");
//     println!("SUMMARY (sorted by diversity)");
//     println!("{:-<60}", "");
    
//     for (rank, (name, div, time)) in sorted_results.iter().enumerate() {
//         println!("{:2}. {:25} Div: {:12.2}  Time: {:?}", 
//             rank + 1, name, div, time);
//     }
//     println!();
// }

// fn discover_test_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//     let mut files = Vec::new();
    
//     if !Path::new(dir).exists() {
//         return Err(format!("Directory not found: {}", dir).into());
//     }
    
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

// fn list_directory_contents(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
//     if !Path::new(dir).exists() {
//         println!("  Directory does not exist!");
//         return Ok(());
//     }
    
//     for entry in fs::read_dir(dir)? {
//         let entry = entry?;
//         let path = entry.path();
//         println!("  - {:?}", path.file_name().unwrap());
//     }
    
//     Ok(())
// }








// mod parser;
// mod solver_qubo;
// mod solver_direct;

// use std::time::Instant;

// fn main() -> Result<(), Box<dyn std::error::Error>> {

//     // let path = "examples_from_mdp/GKD-a/GKD-a_9_n10_m3.txt";
//     let path = "examples_from_mdp/MDG-c/MDG-c_2_n3000_m300.txt";
//     let data = parser::MdpData::load(path);

//     println!("Loaded: n={}, k={}", data.n, data.k);

//     let start_g = Instant::now();
//     let (indices_g, div_g) = solver_qubo::solve_with_qubo(&data, 1000.0)?;
//     let time_g = start_g.elapsed();

//     let start_d = Instant::now();
//     let (indices_d, div_d) = solver_direct::solve_direct(&data);
//     let time_d = start_d.elapsed();

//     println!("--- Gurobi QUBO ---");
//     println!("Time: {:?}, Diversity: {}, Selected: {:?}", time_g, div_g, indices_g);

//     println!("--- Direct Solution ---");
//     println!("Time: {:?}, Diversity: {}, Selected: {:?}", time_d, div_d, indices_d);

//     Ok(())
// }