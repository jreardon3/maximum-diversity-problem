// src/main.rs

mod parser;
mod solver_qubo;
mod solver_grasp;
mod solver_local_search;
mod solver_population;
mod solver_maxcut;

use std::time::{Instant, Duration};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    solution: Option<Vec<usize>>,
}

#[derive(Clone, Serialize, Deserialize)]
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
    category: String,
    instances: Vec<InstanceResults>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let run_stability_tests = args.contains(&"--stability".to_string());
    let run_analysis = !args.contains(&"--no-analysis".to_string());
    let num_stability_runs = if run_stability_tests {
        args.iter()
            .position(|a| a == "--stability")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10)
    } else {
        1
    };

    println!("\n{:=<80}", "");
    println!("MAXIMUM DIVERSITY PROBLEM - COMPREHENSIVE SOLVER COMPARISON");
    println!("{:=<80}\n", "");

    if run_stability_tests {
        println!("🔄 STABILITY MODE: Running each solver {} times per instance\n", num_stability_runs);
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    
    let base_dir = "examples_from_mdp";
    let categories = vec!["GKD", "MDG", "SOM"];
    
    let mut all_result_files = Vec::new();
    
    for category in categories {
        println!("\n{:=<80}", "");
        println!("PROCESSING CATEGORY: {}", category);
        println!("{:=<80}", "");
        
        let mut category_instances: Vec<InstanceResults> = Vec::new();
        
        let subdirs = find_category_subdirs(&base_dir, category)?;
        
        if subdirs.is_empty() {
            println!("No subdirectories found for category {}", category);
            continue;
        }
        
        for subdir in subdirs {
            let dir_path = format!("{}/{}", base_dir, subdir);
            
            println!("\n{:-<80}", "");
            println!("Processing directory: {}", dir_path);
            println!("{:-<80}", "");
            
            let files = discover_test_files(&dir_path)?;
            println!("Found {} files\n", files.len());
            
            for (idx, path) in files.iter().enumerate() {
                println!("[{}/{}] Testing: {}", idx + 1, files.len(), path);
                
                match test_single_file(path, category, num_stability_runs) {
                    Ok(result) => {
                        category_instances.push(result);
                    }
                    Err(_e) => {
                        println!("  ERROR: Failed to process file\n");
                    }
                }
            }
        }
        
        if !category_instances.is_empty() {
            let results = ExperimentResults {
                timestamp: timestamp.clone(),
                category: category.to_string(),
                instances: category_instances.clone(),
            };
            
            let json_file = format!("results_{}_{}.json", category, timestamp);
            save_results_to_json(&results, &json_file)?;
            println!("\n✓ {} results saved to: {}", category, json_file);
            
            all_result_files.push(json_file);
            
            print_category_summary(&results);
        }
    }
    
    // Ensure scripts directory exists
    fs::create_dir_all("scripts")?;
    
    println!("\n{:=<80}", "");
    println!("EXPERIMENT COMPLETE");
    println!("{:=<80}\n", "");
    
    if !all_result_files.is_empty() {
        let result_pattern = all_result_files.join(" ");
        
        println!("📊 Results saved to {} file(s)", all_result_files.len());
        for file in &all_result_files {
            println!("   • {}", file);
        }
        
        if run_analysis {
            println!("\n{:=<80}", "");
            println!("RUNNING AUTOMATED ANALYSIS");
            println!("{:=<80}\n", "");
            
            run_analysis_scripts(&all_result_files)?;
        } else {
            println!("\n{:=<80}", "");
            println!("ANALYSIS COMMANDS (run manually)");
            println!("{:=<80}", "");
            print_analysis_commands(&all_result_files);
        }
    }
    
    Ok(())
}

fn run_analysis_scripts(result_files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let result_args: Vec<&str> = result_files.iter().map(|s| s.as_str()).collect();
    
    // 1. Basic visualization
    println!("📈 Generating basic visualizations...");
    run_python_script("scripts/visualize_results.py", &result_args)?;
    
    // 2. Enhanced visualization
    println!("\n📊 Generating enhanced visualizations...");
    run_python_script("scripts/visualize_results_enhanced.py", &result_args)?;
    
    // 3. Constraint validation
    println!("\n✅ Running constraint validation...");
    run_python_script("scripts/test_constraints.py", &result_args)?;
    
    // 4. Statistical validation
    println!("\n📉 Running statistical validation...");
    run_python_script("scripts/test_statistical_validation.py", &result_args)?;
    
    // 5. Reproducibility tests (if stability mode was used)
    if Path::new("scripts/test_reproducibility.py").exists() {
        println!("\n🔄 Running reproducibility analysis...");
        run_python_script("scripts/test_reproducibility.py", &result_args)?;
    }
    
    println!("\n{:=<80}", "");
    println!("✅ ALL ANALYSES COMPLETE");
    println!("{:=<80}\n", "");
    
    println!("Results available in:");
    println!("  • visualizations_combined/      (basic plots)");
    println!("  • visualizations_enhanced/      (detailed analysis)");
    println!("  • EXECUTIVE_SUMMARY.txt         (text summary)");
    
    Ok(())
}

fn run_python_script(script_path: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(script_path).exists() {
        println!("⚠️  Script not found: {}", script_path);
        return Ok(());
    }
    
    let output = Command::new("python3")
        .arg(script_path)
        .args(args)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("✓ {}", script_path);
                // Print script output
                if !output.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                println!("⚠️  {} failed", script_path);
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not run {}: {}", script_path, e);
            println!("   Make sure Python 3 is installed and required packages are available");
            println!("   You can run it manually: python3 {} {}", script_path, args.join(" "));
        }
    }
    
    Ok(())
}

fn print_analysis_commands(result_files: &[String]) {
    let result_pattern = result_files.join(" ");
    
    println!("\n1. VISUALIZATIONS:");
    println!("   python3 scripts/visualize_results.py {}", result_pattern);
    println!("   python3 scripts/visualize_results_enhanced.py {}", result_pattern);
    
    println!("\n2. VALIDATION:");
    println!("   python3 scripts/test_constraints.py {}", result_pattern);
    
    println!("\n3. STATISTICAL ANALYSIS:");
    println!("   python3 scripts/test_statistical_validation.py {}", result_pattern);
    
    println!("\n4. REPRODUCIBILITY (if --stability was used):");
    println!("   python3 scripts/test_reproducibility.py {}", result_pattern);
    
    println!("\n5. OPTIMALITY GAP (for small instances with known optimal):");
    println!("   python3 scripts/test_optimality_gap.py {}", result_pattern);
    
    println!("\n6. SCALABILITY ANALYSIS:");
    println!("   python3 scripts/test_scalability.py {}", result_pattern);
    
    println!("\n{:=<80}\n", "");
}

fn find_category_subdirs(base_dir: &str, category: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut subdirs = Vec::new();
    
    if !Path::new(base_dir).exists() {
        return Ok(subdirs);
    }
    
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with(category) {
                    subdirs.push(dir_name.to_string());
                }
            }
        }
    }
    
    subdirs.sort();
    Ok(subdirs)
}

fn test_single_file(path: &str, category: &str, num_runs: usize) -> Result<InstanceResults, Box<dyn std::error::Error>> {
    let data = parser::MdpData::load(path);
    println!("  Size: n={}, k={}", data.n, data.k);
    
    let filename = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();
    
    // Run solvers multiple times if stability testing
    let mut all_run_results = Vec::new();
    
    for run in 0..num_runs {
        if num_runs > 1 {
            println!("  Run {}/{}", run + 1, num_runs);
        }
        let results = run_all_solvers_with_timeouts(&data);
        all_run_results.push(results);
    }
    
    // Aggregate results (take best for presentation, but save all for analysis)
    let aggregated_results = if num_runs == 1 {
        all_run_results.into_iter().next().unwrap()
    } else {
        aggregate_multiple_runs(all_run_results)
    };
    
    Ok(InstanceResults {
        filename,
        category: category.to_string(),
        n: data.n,
        k: data.k,
        results: aggregated_results,
    })
}

fn aggregate_multiple_runs(all_runs: Vec<Vec<SolverResult>>) -> Vec<SolverResult> {
    use std::collections::HashMap;
    
    let mut solver_runs: HashMap<String, Vec<SolverResult>> = HashMap::new();
    
    // Group by solver
    for run in all_runs {
        for result in run {
            solver_runs.entry(result.name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
    }
    
    // Aggregate each solver's results (use best diversity)
    solver_runs.into_iter()
        .map(|(name, results)| {
            let best = results.iter()
                .max_by(|a, b| a.diversity.partial_cmp(&b.diversity).unwrap())
                .unwrap();
            
            SolverResult {
                name,
                diversity: best.diversity,
                time_ms: best.time_ms,
                success: best.success,
                solution: best.solution.clone(),
            }
        })
        .collect()
}

fn run_all_solvers_with_timeouts(data: &parser::MdpData) -> Vec<SolverResult> {
    let mut results = Vec::new();
    
    // Time limits in seconds
    let qubo_timeout = 300.0;      // 5 minutes
    let maxcut_timeout = 300.0;    // 5 minutes
    let grasp_timeout = 180.0;     // 3 minutes
    let ls_timeout = 180.0;        // 3 minutes
    let tabu_timeout = 180.0;      // 3 minutes
    let ga_timeout = 180.0;        // 3 minutes

    // 1. QUBO (Constrained)
    print!("  [1/7] QUBO... ");
    let start = Instant::now();
    match solver_qubo::solve_with_qubo(data, 1000.0, qubo_timeout) {
        Ok((solution, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            
            let constraint_satisfied = solution.len() == data.k;
            if !constraint_satisfied {
                eprintln!("    ⚠️  WARNING: QUBO selected {} items, expected {}", solution.len(), data.k);
            }
            
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: div,
                time_ms: time.as_millis(),
                success: div > 0.0 && constraint_satisfied,
                solution: Some(solution),
            });
        }
        Err(_) => {
            let time = start.elapsed();
            println!("✗ Timeout ({:?})", time);
            results.push(SolverResult {
                name: "QUBO".to_string(),
                diversity: 0.0,
                time_ms: time.as_millis(),
                success: false,
                solution: None,
            });
        }
    }

    // 2. MaxCut (MDP→QUBO→MaxCut)
    print!("  [2/7] MaxCut... ");
    let start = Instant::now();
    match solver_maxcut::solve_mdp_via_maxcut(data, 1000.0, maxcut_timeout) {
        Ok((solution, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            
            let constraint_satisfied = solution.len() == data.k;
            if !constraint_satisfied {
                eprintln!("    ⚠️  WARNING: MaxCut selected {} items, expected {}", solution.len(), data.k);
            }
            
            results.push(SolverResult {
                name: "MaxCut".to_string(),
                diversity: div,
                time_ms: time.as_millis(),
                success: div > 0.0 && constraint_satisfied,
                solution: Some(solution),
            });
        }
        Err(_) => {
            let time = start.elapsed();
            println!("✗ Timeout ({:?})", time);
            results.push(SolverResult {
                name: "MaxCut".to_string(),
                diversity: 0.0,
                time_ms: time.as_millis(),
                success: false,
                solution: None,
            });
        }
    }

    // 3. GRASP with timeout
    print!("  [3/7] GRASP... ");
    let start = Instant::now();
    let (solution, div) = run_grasp_with_timeout(data, grasp_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GRASP".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });

    // 4. LS: First Improvement
    print!("  [4/7] LS: First... ");
    let start = Instant::now();
    let (solution, div) = run_ls_with_timeout(data, LocalSearchMethod::FirstImprovement, ls_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-First".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });

    // 5. LS: Best Improvement
    print!("  [5/7] LS: Best... ");
    let start = Instant::now();
    let (solution, div) = run_ls_with_timeout(data, LocalSearchMethod::BestImprovement, ls_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "LS-Best".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });

    // 6. Tabu Search
    print!("  [6/7] Tabu... ");
    let start = Instant::now();
    let (solution, div) = run_ls_with_timeout(data, LocalSearchMethod::TabuSearch { tabu_tenure: 10 }, tabu_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "Tabu".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });

    // 7. Genetic Algorithm
    print!("  [7/7] GA... ");
    let start = Instant::now();
    let (solution, div) = run_ga_with_timeout(data, ga_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "GA".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });

    println!();
    results
}

fn run_grasp_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    let config = GraspConfig {
        iterations: usize::MAX,
        alpha: 0.3,
        local_search_iters: 500,
    };
    solver_grasp::solve_grasp(data, &config, deadline)
}

fn run_ls_with_timeout(data: &parser::MdpData, method: LocalSearchMethod, timeout_secs: f64) -> (Vec<usize>, f64) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    let config = LocalSearchConfig {
        method,
        max_iters: usize::MAX,
    };
    solver_local_search::solve_local_search(data, &config, deadline)
}

fn run_ga_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    let config = GeneticConfig {
        population_size: 30,
        generations: usize::MAX,
        crossover_rate: 0.8,
        mutation_rate: 0.15,
        elite_size: 3,
    };
    solver_population::solve_genetic(data, &config, deadline)
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

fn print_category_summary(results: &ExperimentResults) {
    println!("\n{:-<100}", "");
    println!("SUMMARY FOR {} ({} instances)", results.category, results.instances.len());
    println!("{:-<100}", "");
    
    if results.instances.is_empty() {
        println!("No instances processed.");
        return;
    }
    
    let mut all_solvers: Vec<String> = results.instances[0].results.iter()
        .map(|r| r.name.clone())
        .collect();
    all_solvers.sort();
    
    print!("{:<30} {:>8} {:>6} |", "File", "n", "k");
    for solver in &all_solvers {
        print!(" {:>12}", solver);
    }
    println!();
    println!("{:-<100}", "");
    
    for inst in &results.instances {
        print!("{:<30} {:>8} {:>6} |", 
            truncate_filename(&inst.filename, 30),
            inst.n, 
            inst.k
        );
        
        for solver in &all_solvers {
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
    println!();
}

fn truncate_filename(filename: &str, max_len: usize) -> String {
    if filename.len() <= max_len {
        filename.to_string()
    } else {
        format!("...{}", &filename[filename.len() - (max_len - 3)..])
    }
}



// // // TODO (Performance Analysis Enhancements)
// // // Add more metrics beyond just diversity score and time:
// //     // Solution quality gap: Compare heuristic solutions to the optimal/best known solution
// //     // Time-to-target: How long to reach X% of best solution
// //     // Convergence plots: Show solution quality over time/iterations
// //     // Memory usage: Track RAM consumption
// //     // Scalability analysis: Plot time/quality vs. instance size
// //     // Success rate: Run multiple times with different seeds, report % of runs finding optimal
// // // Statistical rigor:
// //     // Run each solver multiple times (10-30 runs) with different random seeds
// //     // Report mean, median, std dev, min, max
// //     // Add confidence intervals

// // // TODO!! (Add MaxCut)
// // // It's also NP-hard and has similar algorithmic approaches
// // // You can use the same heuristics (GRASP, Tabu, GA, Local Search)
// // // Gurobi can solve MaxCut via QUBO or ILP formulation
// // // Shows your framework generalizes beyond MDP
// // // ./your_program --problem maxcut --solver grasp --input graph.txt

// // // TODO (Instance Analysis) - Characterize why certain solvers work better on certain instances:
// // // Instance features: Size, density, structure
// // // Clustering analysis: Group similar instances
// // // Performance profiles: Which solver is best on which type?
// // // Hardness prediction: Can you predict which instances will be hard?

// // // TODO!! (Visualization & Reporting)
// // // Performance profiles (like in optimization papers)
// // // Scatter plots: Quality vs Time for each solver
// // // Heatmaps: Solver performance across instance types
// // // Pareto frontier: Time-quality tradeoffs
// // // Solution visualization: Show selected subset graphically

// // // TODO!! (Practical Recommendations Section)
// // // End with guidance like:
// // // "Use Gurobi for instances n < 50 when optimality is critical"
// // // "Use GRASP for instances 50 < n < 500 for best quality/time tradeoff"
// // // "Use First Improvement LS for real-time applications (sub-millisecond)")


// mod parser;
// mod solver_qubo;
// mod solver_grasp;
// mod solver_local_search;
// mod solver_population;
// mod solver_maxcut;

// use std::time::{Instant, Duration};
// use std::fs::{self, File};
// use std::io::Write;
// use std::path::Path;
// use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
// use solver_grasp::GraspConfig;
// use solver_population::GeneticConfig;
// use serde::{Serialize, Deserialize};
// // use std::time::Instant;


// #[derive(Clone, Serialize, Deserialize)]
// struct SolverResult {
//     name: String,
//     diversity: f64,
//     time_ms: u128,
//     success: bool,
// }

// #[derive(Clone, Serialize, Deserialize)]
// struct InstanceResults {
//     filename: String,
//     category: String,
//     n: usize,
//     k: usize,
//     results: Vec<SolverResult>,
// }

// #[derive(Serialize, Deserialize)]
// struct ExperimentResults {
//     timestamp: String,
//     category: String,
//     instances: Vec<InstanceResults>,
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     println!("\n{:=<80}", "");
//     println!("MAXIMUM DIVERSITY PROBLEM - COMPREHENSIVE SOLVER COMPARISON");
//     println!("{:=<80}\n", "");

//     // pub type Deadline = Instant;

//     let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

//     // let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    
//     let base_dir = "examples_from_mdp";
//     let categories = vec!["GKD", "MDG", "SOM"];
    
//     for category in categories {
//         println!("\n{:=<80}", "");
//         println!("PROCESSING CATEGORY: {}", category);
//         println!("{:=<80}", "");
        
//         let mut category_instances: Vec<InstanceResults> = Vec::new();
        
//         let subdirs = find_category_subdirs(&base_dir, category)?;
        
//         if subdirs.is_empty() {
//             println!("No subdirectories found for category {}", category);
//             continue;
//         }
        
//         for subdir in subdirs {
//             let dir_path = format!("{}/{}", base_dir, subdir);
            
//             println!("\n{:-<80}", "");
//             println!("Processing directory: {}", dir_path);
//             println!("{:-<80}", "");
            
//             let files = discover_test_files(&dir_path)?;
//             println!("Found {} files\n", files.len());
            
//             for (idx, path) in files.iter().enumerate() {
//                 println!("[{}/{}] Testing: {}", idx + 1, files.len(), path);
                
//                 match test_single_file(path, category) {
//                     Ok(result) => {
//                         category_instances.push(result);
//                     }
//                     Err(_e) => {
//                         println!("  ERROR: Failed to process file\n");
//                     }
//                 }
//             }
//         }
        
//         if !category_instances.is_empty() {
//             let results = ExperimentResults {
//                 timestamp: timestamp.clone(),
//                 category: category.to_string(),
//                 instances: category_instances.clone(),
//             };
            
//             let json_file = format!("results_{}_{}.json", category, timestamp);
//             save_results_to_json(&results, &json_file)?;
//             println!("\n✓ {} results saved to: {}", category, json_file);
            
//             print_category_summary(&results);
//         }
//     }
    
//     generate_visualization_script()?;
//     println!("\n✓ Visualization script saved to: visualize_results.py");
//     println!("  Run with: python visualize_results.py results_*_{}.json", timestamp);
    
//     Ok(())
// }

// fn find_category_subdirs(base_dir: &str, category: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//     let mut subdirs = Vec::new();
    
//     if !Path::new(base_dir).exists() {
//         return Ok(subdirs);
//     }
    
//     for entry in fs::read_dir(base_dir)? {
//         let entry = entry?;
//         let path = entry.path();
        
//         if path.is_dir() {
//             if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
//                 if dir_name.starts_with(category) {
//                     subdirs.push(dir_name.to_string());
//                 }
//             }
//         }
//     }
    
//     subdirs.sort();
//     Ok(subdirs)
// }

// fn test_single_file(path: &str, category: &str) -> Result<InstanceResults, Box<dyn std::error::Error>> {
//     let data = parser::MdpData::load(path);
//     println!("  Size: n={}, k={}", data.n, data.k);
    
//     let filename = Path::new(path)
//         .file_name()
//         .and_then(|s| s.to_str())
//         .unwrap_or(path)
//         .to_string();
    
//     // Run ALL solvers with time limits
//     let results = run_all_solvers_with_timeouts(&data);
    
//     Ok(InstanceResults {
//         filename,
//         category: category.to_string(),
//         n: data.n,
//         k: data.k,
//         results,
//     })
// }

// fn run_all_solvers_with_timeouts(data: &parser::MdpData) -> Vec<SolverResult> {
//     let mut results = Vec::new();
    
//     // Time limits in seconds
//     let qubo_timeout = 300.0;      // 5 minutes
//     let maxcut_timeout = 300.0;    // 5 minutes
//     let grasp_timeout = 180.0;     // 3 minutes
//     let ls_timeout = 180.0;        // 3 minutes
//     let tabu_timeout = 180.0;      // 3 minutes
//     let ga_timeout = 180.0;        // 3 minutes

//     // 1. QUBO (Constrained)
//     print!("  [1/7] QUBO... ");
//     let start = Instant::now();
//     match solver_qubo::solve_with_qubo(data, 1000.0, qubo_timeout) {
//         Ok((_, div)) => {
//             let time = start.elapsed();
//             println!("✓ {:.2} ({:?})", div, time);
//             results.push(SolverResult {
//                 name: "QUBO".to_string(),
//                 diversity: div,
//                 time_ms: time.as_millis(),
//                 success: div > 0.0,
//             });
//         }
//         Err(_) => {
//             let time = start.elapsed();
//             println!("✗ Timeout ({:?})", time);
//             results.push(SolverResult {
//                 name: "QUBO".to_string(),
//                 diversity: 0.0,
//                 time_ms: time.as_millis(),
//                 success: false,
//             });
//         }
//     }

//     // 2. MaxCut (MDP→QUBO→MaxCut)
//     print!("  [2/7] MaxCut... ");
//     let start = Instant::now();
//     match solver_maxcut::solve_mdp_via_maxcut(data, 1000.0, maxcut_timeout) {
//         Ok((_, div)) => {
//             let time = start.elapsed();
//             println!("✓ {:.2} ({:?})", div, time);
//             results.push(SolverResult {
//                 name: "MaxCut".to_string(),
//                 diversity: div,
//                 time_ms: time.as_millis(),
//                 success: div > 0.0,
//             });
//         }
//         Err(_) => {
//             let time = start.elapsed();
//             println!("✗ Timeout ({:?})", time);
//             results.push(SolverResult {
//                 name: "MaxCut".to_string(),
//                 diversity: 0.0,
//                 time_ms: time.as_millis(),
//                 success: false,
//             });
//         }
//     }

//     // 3. GRASP with timeout
//     print!("  [3/7] GRASP... ");
//     let start = Instant::now();
//     let (_, div) = run_grasp_with_timeout(data, grasp_timeout);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GRASP".to_string(),
//         diversity: div,
//         time_ms: time.as_millis(),
//         success: true,
//     });

//     // 4. LS: First Improvement with timeout
//     print!("  [4/7] LS: First... ");
//     let start = Instant::now();
//     let (_, div) = run_ls_with_timeout(data, LocalSearchMethod::FirstImprovement, ls_timeout);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS-First".to_string(),
//         diversity: div,
//         time_ms: time.as_millis(),
//         success: true,
//     });

//     // 5. LS: Best Improvement with timeout
//     print!("  [5/7] LS: Best... ");
//     let start = Instant::now();
//     let (_, div) = run_ls_with_timeout(data, LocalSearchMethod::BestImprovement, ls_timeout);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "LS-Best".to_string(),
//         diversity: div,
//         time_ms: time.as_millis(),
//         success: true,
//     });

//     // 6. Tabu Search with timeout
//     print!("  [6/7] Tabu... ");
//     let start = Instant::now();
//     let (_, div) = run_ls_with_timeout(data, LocalSearchMethod::TabuSearch { tabu_tenure: 10 }, tabu_timeout);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "Tabu".to_string(),
//         diversity: div,
//         time_ms: time.as_millis(),
//         success: true,
//     });

//     // 7. Genetic Algorithm with timeout
//     print!("  [7/7] GA... ");
//     let start = Instant::now();
//     let (_, div) = run_ga_with_timeout(data, ga_timeout);
//     let time = start.elapsed();
//     println!("✓ {:.2} ({:?})", div, time);
//     results.push(SolverResult {
//         name: "GA".to_string(),
//         diversity: div,
//         time_ms: time.as_millis(),
//         success: true,
//     });

//     println!();
//     results
// }

// // fn run_grasp_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
// //     let start = Instant::now();
// //     let timeout = Duration::from_secs_f64(timeout_secs);
    
// //     let mut best_solution = Vec::new();
// //     let mut best_diversity = 0.0;
    
// //     let config = GraspConfig {
// //         iterations: 1000000, // Very high, will stop on timeout
// //         alpha: 0.3,
// //         local_search_iters: 500,
// //     };
    
// //     for iter in 0..config.iterations {
// //         if start.elapsed() >= timeout {
// //             break;
// //         }
        
// //         // One GRASP iteration
// //         let solution = greedy_randomized_construction(data, config.alpha);
// //         let (improved, diversity) = local_search_iteration(data, solution, config.local_search_iters);
        
// //         if diversity > best_diversity {
// //             best_diversity = diversity;
// //             best_solution = improved;
// //         }
        
// //         // Early stopping if converged
// //         if iter > 20 && iter % 10 == 0 {
// //             // Check if we haven't improved in a while
// //             if best_diversity > 0.0 {
// //                 break;
// //             }
// //         }
// //     }
    
// //     (best_solution, best_diversity)
// // }

// // fn run_ls_with_timeout(data: &parser::MdpData, method: LocalSearchMethod, timeout_secs: f64) -> (Vec<usize>, f64) {
// //     let start = Instant::now();
// //     let timeout = Duration::from_secs_f64(timeout_secs);
    
// //     let config = LocalSearchConfig {
// //         method,
// //         max_iters: 1000000, // Very high, will stop on timeout
// //     };
    
// //     // Start with timeout checking wrapper
// //     let mut iteration = 0;
// //     loop {
// //         if start.elapsed() >= timeout {
// //             break;
// //         }
        
// //         let result = solver_local_search::solve_local_search(data, &config);
        
// //         // Local search usually converges quickly, so just return after one run
// //         return result;
// //     }
    
// //     // Fallback (shouldn't reach here)
// //     (Vec::new(), 0.0)
// // }

// fn run_grasp_with_timeout(
//     data: &parser::MdpData,
//     timeout_secs: f64,
// ) -> (Vec<usize>, f64) {
//     let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);

//     let config = GraspConfig {
//         iterations: usize::MAX, // controlled by deadline
//         alpha: 0.3,
//         local_search_iters: 500,
//     };

//     solver_grasp::solve_grasp(data, &config, deadline)
// }


// fn run_ls_with_timeout(
//     data: &parser::MdpData,
//     method: LocalSearchMethod,
//     timeout_secs: f64,
// ) -> (Vec<usize>, f64) {
//     let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);

//     let config = LocalSearchConfig {
//         method,
//         max_iters: usize::MAX, // controlled by time now
//     };

//     solver_local_search::solve_local_search(data, &config, deadline)
// }

// fn run_ga_with_timeout(
//     data: &parser::MdpData,
//     timeout_secs: f64,
// ) -> (Vec<usize>, f64) {
//     let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);

//     let config = GeneticConfig {
//         population_size: 30,
//         generations: usize::MAX, // time-controlled
//         crossover_rate: 0.8,
//         mutation_rate: 0.15,
//         elite_size: 3,
//     };

//     solver_population::solve_genetic(data, &config, deadline)
// }


// // Helper functions for GRASP timeout implementation
// // fn greedy_randomized_construction(data: &parser::MdpData, alpha: f64) -> Vec<usize> {
// //     use rand::Rng;
// //     let mut rng = rand::thread_rng();
// //     let mut selected = Vec::with_capacity(data.k);
// //     let mut available: Vec<usize> = (0..data.n).collect();

// //     for _ in 0..data.k {
// //         if available.is_empty() {
// //             break;
// //         }

// //         let mut contributions: Vec<(usize, f64)> = available
// //             .iter()
// //             .map(|&idx| {
// //                 let mut contrib = 0.0;
// //                 for &s in &selected {
// //                     contrib += data.get_dist(idx, s);
// //                 }
// //                 (idx, contrib)
// //             })
// //             .collect();

// //         contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

// //         let c_min = contributions.last().unwrap().1;
// //         let c_max = contributions.first().unwrap().1;
// //         let threshold = c_max - alpha * (c_max - c_min);

// //         let rcl: Vec<usize> = contributions
// //             .iter()
// //             .filter(|(_, contrib)| *contrib >= threshold)
// //             .map(|(idx, _)| *idx)
// //             .collect();

// //         let chosen = rcl[rng.gen_range(0..rcl.len())];
// //         selected.push(chosen);
// //         available.retain(|&x| x != chosen);
// //     }

// //     selected
// // }

// // fn local_search_iteration(data: &parser::MdpData, mut selected: Vec<usize>, max_iters: usize) -> (Vec<usize>, f64) {
// //     let mut unselected: Vec<usize> = (0..data.n)
// //         .filter(|&i| !selected.contains(&i))
// //         .collect();

// //     let mut current_diversity = calculate_diversity(&selected, data);

// //     for _ in 0..max_iters {
// //         let mut best_swap = None;
// //         let mut best_gain = 0.0;

// //         for i in 0..selected.len() {
// //             for j in 0..unselected.len() {
// //                 let mut gain = 0.0;
// //                 for &s in &selected {
// //                     if s == selected[i] {
// //                         continue;
// //                     }
// //                     gain += data.get_dist(unselected[j], s) - data.get_dist(selected[i], s);
// //                 }
                
// //                 if gain > best_gain {
// //                     best_gain = gain;
// //                     best_swap = Some((i, j));
// //                 }
// //             }
// //         }

// //         if let Some((i, j)) = best_swap {
// //             let temp = selected[i];
// //             selected[i] = unselected[j];
// //             unselected[j] = temp;
// //             current_diversity += best_gain;
// //         } else {
// //             break;
// //         }
// //     }

// //     (selected, current_diversity)
// // }

// fn calculate_diversity(selected: &[usize], data: &parser::MdpData) -> f64 {
//     let mut sum = 0.0;
//     for i in 0..selected.len() {
//         for j in (i + 1)..selected.len() {
//             sum += data.get_dist(selected[i], selected[j]);
//         }
//     }
//     sum
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

// fn save_results_to_json(results: &ExperimentResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let json = serde_json::to_string_pretty(results)?;
//     let mut file = File::create(filename)?;
//     file.write_all(json.as_bytes())?;
//     Ok(())
// }

// fn print_category_summary(results: &ExperimentResults) {
//     println!("\n{:-<100}", "");
//     println!("SUMMARY FOR {} ({} instances)", results.category, results.instances.len());
//     println!("{:-<100}", "");
    
//     if results.instances.is_empty() {
//         println!("No instances processed.");
//         return;
//     }
    
//     let mut all_solvers: Vec<String> = results.instances[0].results.iter()
//         .map(|r| r.name.clone())
//         .collect();
//     all_solvers.sort();
    
//     print!("{:<30} {:>8} {:>6} |", "File", "n", "k");
//     for solver in &all_solvers {
//         print!(" {:>12}", solver);
//     }
//     println!();
//     println!("{:-<100}", "");
    
//     for inst in &results.instances {
//         print!("{:<30} {:>8} {:>6} |", 
//             truncate_filename(&inst.filename, 30),
//             inst.n, 
//             inst.k
//         );
        
//         for solver in &all_solvers {
//             if let Some(result) = inst.results.iter().find(|r| r.name == *solver) {
//                 if result.success {
//                     print!(" {:>12.2}", result.diversity);
//                 } else {
//                     print!(" {:>12}", "TIMEOUT");
//                 }
//             } else {
//                 print!(" {:>12}", "-");
//             }
//         }
//         println!();
//     }
//     println!();
// }

// fn truncate_filename(filename: &str, max_len: usize) -> String {
//     if filename.len() <= max_len {
//         filename.to_string()
//     } else {
//         format!("...{}", &filename[filename.len() - (max_len - 3)..])
//     }
// }

// fn generate_visualization_script() -> Result<(), Box<dyn std::error::Error>> {
//     let script = r#"#!/usr/bin/env python3
// """
// Visualization script for MDP solver comparison results.
// Can process multiple result JSON files at once.

// Usage: python visualize_results.py results_*.json
// """

// import json
// import sys
// import matplotlib.pyplot as plt
// import seaborn as sns
// import pandas as pd
// import numpy as np
// from pathlib import Path
// import glob

// sns.set_style("whitegrid")
// plt.rcParams['figure.figsize'] = (12, 8)

// def load_all_results(json_files):
//     """Load and combine multiple result files"""
//     all_instances = []
    
//     for json_file in json_files:
//         with open(json_file, 'r') as f:
//             data = json.load(f)
//             all_instances.extend(data['instances'])
    
//     return all_instances

// def create_scatter_plot(df, output_dir):
//     """Quality vs Time scatter plot for each solver"""
//     fig, ax = plt.subplots(figsize=(12, 8))
    
//     solvers = df['solver'].unique()
//     colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
//     for solver, color in zip(solvers, colors):
//         solver_data = df[df['solver'] == solver]
//         ax.scatter(solver_data['time_ms'], solver_data['diversity'], 
//                   label=solver, alpha=0.6, s=100, color=color)
    
//     ax.set_xlabel('Time (ms)', fontsize=12)
//     ax.set_ylabel('Diversity Score', fontsize=12)
//     ax.set_title('Solution Quality vs Computation Time', fontsize=14, fontweight='bold')
//     ax.set_xscale('log')
//     ax.legend()
//     ax.grid(True, alpha=0.3)
    
//     plt.tight_layout()
//     plt.savefig(output_dir / 'scatter_quality_vs_time.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: scatter_quality_vs_time.png")
//     plt.close()

// def create_heatmap(df, output_dir):
//     """Heatmap of solver performance across instance types"""
//     pivot_data = df.groupby(['category', 'solver'])['diversity'].mean().unstack(fill_value=0)
    
//     fig, ax = plt.subplots(figsize=(10, 6))
//     sns.heatmap(pivot_data, annot=True, fmt='.1f', cmap='YlOrRd', 
//                 cbar_kws={'label': 'Avg Diversity'}, ax=ax)
//     ax.set_title('Average Solver Performance by Instance Category', fontsize=14, fontweight='bold')
//     ax.set_xlabel('Solver', fontsize=12)
//     ax.set_ylabel('Instance Category', fontsize=12)
    
//     plt.tight_layout()
//     plt.savefig(output_dir / 'heatmap_performance.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: heatmap_performance.png")
//     plt.close()

// def create_pareto_frontier(df, output_dir):
//     """Pareto frontier showing time-quality tradeoffs"""
//     fig, ax = plt.subplots(figsize=(12, 8))
    
//     categories = df['category'].unique()
//     colors = plt.cm.tab10(np.linspace(0, 1, len(categories)))
    
//     for category, color in zip(categories, colors):
//         cat_data = df[df['category'] == category]
//         solver_avgs = cat_data.groupby('solver').agg({'time_ms': 'mean', 'diversity': 'mean'}).reset_index()
        
//         ax.scatter(solver_avgs['time_ms'], solver_avgs['diversity'], 
//                   label=category, alpha=0.7, s=150, color=color, edgecolors='black', linewidth=1.5)
        
//         for _, row in solver_avgs.iterrows():
//             ax.annotate(row['solver'], 
//                        (row['time_ms'], row['diversity']),
//                        xytext=(5, 5), textcoords='offset points', fontsize=8)
    
//     ax.set_xlabel('Average Time (ms)', fontsize=12)
//     ax.set_ylabel('Average Diversity Score', fontsize=12)
//     ax.set_title('Pareto Frontier: Time-Quality Tradeoffs by Category', fontsize=14, fontweight='bold')
//     ax.set_xscale('log')
//     ax.legend()
//     ax.grid(True, alpha=0.3)
    
//     plt.tight_layout()
//     plt.savefig(output_dir / 'pareto_frontier.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: pareto_frontier.png")
//     plt.close()

// def create_box_plots(df, output_dir):
//     """Box plots showing distribution of results"""
//     fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
//     df_success = df[df['success'] == True]
//     df_success.boxplot(column='diversity', by='solver', ax=ax1)
//     ax1.set_title('Distribution of Diversity Scores by Solver', fontsize=12, fontweight='bold')
//     ax1.set_xlabel('Solver', fontsize=11)
//     ax1.set_ylabel('Diversity Score', fontsize=11)
//     plt.sca(ax1)
//     plt.xticks(rotation=45)
    
//     df_success.boxplot(column='time_ms', by='solver', ax=ax2)
//     ax2.set_title('Distribution of Computation Times by Solver', fontsize=12, fontweight='bold')
//     ax2.set_xlabel('Solver', fontsize=11)
//     ax2.set_ylabel('Time (ms)', fontsize=11)
//     ax2.set_yscale('log')
//     plt.sca(ax2)
//     plt.xticks(rotation=45)
    
//     plt.suptitle('')
//     plt.tight_layout()
//     plt.savefig(output_dir / 'boxplot_distributions.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: boxplot_distributions.png")
//     plt.close()

// def create_scaling_plot(df, output_dir):
//     """How solvers scale with problem size"""
//     fig, ax = plt.subplots(figsize=(12, 8))
    
//     solvers = df['solver'].unique()
//     colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
//     for solver, color in zip(solvers, colors):
//         solver_data = df[df['solver'] == solver].copy()
//         solver_data = solver_data.sort_values('n')
//         grouped = solver_data.groupby('n')['time_ms'].mean().reset_index()
        
//         ax.plot(grouped['n'], grouped['time_ms'], 
//                marker='o', label=solver, color=color, linewidth=2, markersize=8)
    
//     ax.set_xlabel('Problem Size (n)', fontsize=12)
//     ax.set_ylabel('Average Time (ms)', fontsize=12)
//     ax.set_title('Solver Scalability: Time vs Problem Size', fontsize=14, fontweight='bold')
//     ax.set_yscale('log')
//     ax.legend()
//     ax.grid(True, alpha=0.3)
    
//     plt.tight_layout()
//     plt.savefig(output_dir / 'scaling_analysis.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: scaling_analysis.png")
//     plt.close()

// def create_win_rate_chart(df, output_dir):
//     """Bar chart showing which solver wins most often"""
//     df_success = df[df['success'] == True].copy()
//     best_solvers = df_success.loc[df_success.groupby('filename')['diversity'].idxmax(), 'solver']
//     win_counts = best_solvers.value_counts()
    
//     fig, ax = plt.subplots(figsize=(10, 6))
//     win_counts.plot(kind='bar', ax=ax, color='steelblue', edgecolor='black')
//     ax.set_title('Solver Win Frequency (Best Diversity Score)', fontsize=14, fontweight='bold')
//     ax.set_xlabel('Solver', fontsize=12)
//     ax.set_ylabel('Number of Instances Won', fontsize=12)
//     ax.set_xticklabels(ax.get_xticklabels(), rotation=45, ha='right')
    
//     for i, v in enumerate(win_counts):
//         ax.text(i, v + 0.5, str(v), ha='center', va='bottom', fontweight='bold')
    
//     plt.tight_layout()
//     plt.savefig(output_dir / 'win_rate.png', dpi=300, bbox_inches='tight')
//     print(f"✓ Saved: win_rate.png")
//     plt.close()

// def main():
//     if len(sys.argv) < 2:
//         print("Usage: python visualize_results.py results_*.json")
//         print("  or:  python visualize_results.py results_GKD_*.json results_MDG_*.json results_SOM_*.json")
//         sys.exit(1)
    
//     json_files = []
//     for pattern in sys.argv[1:]:
//         json_files.extend(glob.glob(pattern))
    
//     if not json_files:
//         print("No result files found!")
//         sys.exit(1)
    
//     print(f"\nLoading results from {len(json_files)} file(s):")
//     for f in json_files:
//         print(f"  - {f}")
    
//     instances = load_all_results(json_files)
    
//     rows = []
//     for instance in instances:
//         for result in instance['results']:
//             rows.append({
//                 'filename': instance['filename'],
//                 'category': instance['category'],
//                 'n': instance['n'],
//                 'k': instance['k'],
//                 'solver': result['name'],
//                 'diversity': result['diversity'],
//                 'time_ms': result['time_ms'],
//                 'success': result['success']
//             })
    
//     df = pd.DataFrame(rows)
    
//     print(f"\nLoaded {len(df)} result records from {len(instances)} instances")
//     print(f"Solvers: {', '.join(df['solver'].unique())}")
//     print(f"Categories: {', '.join(df['category'].unique())}\n")
    
//     output_dir = Path('visualizations_combined')
//     output_dir.mkdir(exist_ok=True)
    
//     print(f"Generating visualizations in: {output_dir}/\n")
    
//     create_scatter_plot(df, output_dir)
//     create_heatmap(df, output_dir)
//     create_pareto_frontier(df, output_dir)
//     create_box_plots(df, output_dir)
//     create_scaling_plot(df, output_dir)
//     create_win_rate_chart(df, output_dir)
    
//     print(f"\n✓ All visualizations saved to: {output_dir}/")
//     print(f"  Total: 6 plots generated")

// if __name__ == '__main__':
//     main()
// "#;
    
//     let mut file = File::create("visualize_results.py")?;
//     file.write_all(script.as_bytes())?;
//     Ok(())
// }

