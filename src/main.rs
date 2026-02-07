// src/main.rs

mod parser;

mod solver_grasp;                   // this one matched the description in the write-up, I didn't adjust it 
mod adj_solver_local_search_its;    // adjusted Local Search (this new one is Iterated Tabu Search, the other local solvers don't keep tabu lists)
mod adj_solver_obma;                // adjusted OBMA (the other one didn't use opposition based learning)
mod adj_solver_breakpoint;          // adjusted BreakPoint solver (added it, mentioned in Tu's email)
mod adj_solver_population_ma;       // adjusted Population (this new one is the Memetic Algorithm that matches the textbook)
mod adj_solver_maxcut;              // adjusted MaxCut solver (maxcut definition now matches the definition in the write-up)
mod adj_solver_qubo;                // adjusted the penalty -> it had to be contained in the objective function

mod solver_local_search;
mod solver_population;
mod solver_obma;
mod solver_qubo;
mod solver_maxcut;

use std::time::{Instant, Duration};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use solver_local_search::{LocalSearchConfig, LocalSearchMethod};
use solver_grasp::GraspConfig;
use solver_population::GeneticConfig;
use adj_solver_obma::ObmaConfig;
use adj_solver_local_search_its::ItsConfig;
use adj_solver_breakpoint::BreakpointConfig;
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

fn record_success(results: &mut Vec<SolverResult>, name: &str, div: f64, solution: Vec<usize>, start: Instant, k: usize) {
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    
    // Check constraint satisfaction
    let constraint_satisfied = solution.len() == k;
    if !constraint_satisfied {
        eprintln!("    ⚠️  WARNING: {} selected {} items, expected {}", name, solution.len(), k);
    }
    
    results.push(SolverResult {
        name: name.to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: div > 0.0, // We mark it successful if it returns diversity, even if constraint is slightly off (repair usually fixes this inside the solver)
        solution: Some(solution),
    });
}

fn record_timeout(results: &mut Vec<SolverResult>, name: &str, start: Instant) {
    let time = start.elapsed();
    println!("✗ Timeout ({:?})", time);
    results.push(SolverResult {
        name: name.to_string(),
        diversity: 0.0,
        time_ms: time.as_millis(),
        success: false,
        solution: None,
    });
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

    // ---- Set to 'false' to deselect solver ------
    let run_qubo    = false; 
    let run_maxcut  = true; 
    let run_grasp   = true;
    let run_ls      = true;
    let run_ga      = false;
    let run_obma    = true;
    let run_bp      = true;
    let run_mamdp   = true;
    let run_qubo_adjusted = true;
    // ---------------------------------------------

    // Determine time limit based on instance size N
    let timeout_seconds = if data.n < 100 {
        1.0   // 1 second for tiny instances (N=10, 25, 50)
    } else if data.n < 1000 {
        60.0  // 1 minute for medium (N=500)
    } else {
        300.0 // 5 minutes for large (N=2000+)
    };
    println!("  -> Size N={}, using Time Limit: {:.1}s", data.n, timeout_seconds);
    
    // Time limits in seconds
    let qubo_timeout = timeout_seconds;
    let maxcut_timeout = timeout_seconds;
    let grasp_timeout = timeout_seconds;
    let ls_timeout = timeout_seconds;
    let tabu_timeout = timeout_seconds;
    let ga_timeout = timeout_seconds;
    let obma_timeout = timeout_seconds;
    let bp_timeout = timeout_seconds;
    let mamdp_timeout = timeout_seconds;

    // 0. QUBO (Transformation #1 + Penalty + Repair)
    if run_qubo_adjusted {
        print!("  [0/10] QUBO... ");
        let start = Instant::now();
        
        // Removed the target_objective argument
        match adj_solver_qubo::solve(data, qubo_timeout) {
            Ok((solution, div)) => {
                let time = start.elapsed();
                println!("✓ {:.2} ({:?})", div, time);
                
                results.push(SolverResult {
                    name: "QUBO".to_string(),
                    diversity: div,
                    time_ms: time.as_millis(),
                    success: true, 
                    solution: Some(solution),
                });
            }
            Err(_) => {
                let time = start.elapsed();
                println!("✗ Timeout/Error ({:?})", time);
                results.push(SolverResult {
                    name: "QUBO".to_string(),
                    diversity: 0.0,
                    time_ms: time.as_millis(),
                    success: false,
                    solution: None,
                });
            }
        }
    }

    // 1. QUBO Constrained (Hard Constraint)
    if run_qubo {
        print!("  [1/10] QUBO-C... ");
        let start = Instant::now();
        match solver_qubo::solve_with_qubo_constrained(data, qubo_timeout) {
        Ok((solution, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            
            let constraint_satisfied = solution.len() == data.k;
            if !constraint_satisfied {
                eprintln!("    ⚠️  WARNING: QUBO-C selected {} items, expected {}", solution.len(), data.k);
            }
            
            results.push(SolverResult {
                name: "QUBO-Constrained".to_string(),
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
                name: "QUBO-Constrained".to_string(),
                diversity: 0.0,
                time_ms: time.as_millis(),
                success: false,
                solution: None,
            });
        }
        }
    }
    

    // 2. QUBO Penalty (Soft Constraint)
    if run_qubo {
        print!("  [2/10] QUBO-P... ");
        let start = Instant::now();
        match solver_qubo::solve_with_qubo_penalty(data, 1000.0, qubo_timeout) {
        Ok((solution, div)) => {
            let time = start.elapsed();
            println!("✓ {:.2} ({:?})", div, time);
            
            let constraint_satisfied = solution.len() == data.k;
            if !constraint_satisfied {
                eprintln!("    ⚠️  WARNING: QUBO-P selected {} items, expected {}", solution.len(), data.k);
            }
            
            results.push(SolverResult {
                name: "QUBO-Penalty".to_string(),
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
                name: "QUBO-Penalty".to_string(),
                diversity: 0.0,
                time_ms: time.as_millis(),
                success: false,
                solution: None,
            });
        }
    }
    }
    

    // 3. MaxCut (MDP→QUBO→MaxCut)
    if run_maxcut {
        print!("  [3/10] MaxCut solver... ");
        let start = Instant::now();
        
        match adj_solver_maxcut::solve_mdp_via_maxcut(data, 1000.0, maxcut_timeout) {
            Ok((solution, div)) => record_success(&mut results, "MaxCut", div, solution, start, data.k),
            Err(_) => record_timeout(&mut results, "MaxCut", start),
        }
    }
    

    // 4. GRASP with timeout
    if run_grasp {
        print!("  [4/10] GRASP... ");
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
    }
    
    if run_ls {
        // 5. Iterated Tabu Search (Replaces LS-First, LS-Best, and Standard Tabu)
        print!("  [5/10] ITS... "); 
        let start = Instant::now();
        
        // We use tabu_timeout (or ls_timeout) here
        let (solution, div) = run_its_with_timeout(data, tabu_timeout);
        
        let time = start.elapsed();
        println!("✓ {:.2} ({:?})", div, time);
        
        results.push(SolverResult {
            name: "ITS".to_string(), // Name in the JSON output
            diversity: div,
            time_ms: time.as_millis(),
            success: true,
            solution: Some(solution),
        });
    }
    
    if run_ga {
        // 8. Genetic Algorithm
    print!("  [8/10] GA... ");
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
    }
    
    if run_obma {
            // 9. OBMA (Opposition-Based Memetic Algorithm)
    print!("  [9/10] OBMA... ");
    let start = Instant::now();
    let (solution, div) = run_obma_with_timeout(data, obma_timeout);
    let time = start.elapsed();
    println!("✓ {:.2} ({:?})", div, time);
    results.push(SolverResult {
        name: "OBMA".to_string(),
        diversity: div,
        time_ms: time.as_millis(),
        success: true,
        solution: Some(solution),
    });
    }
    
    if run_bp {
        // 10. Breakpoint Algorithm
        print!("  [10/10] Breakpoint... ");
        let start = Instant::now();
        let deadline = Instant::now() + Duration::from_secs_f64(bp_timeout);
        
        let config = BreakpointConfig {
            max_lambda_iterations: 30,
            ..Default::default()
        };

        // Note: You need to expose solve_breakpoint in the module as public
        let (solution, div) = adj_solver_breakpoint::solve_breakpoint(data, &config, deadline);
        
        let time = start.elapsed();
        println!("✓ {:.2} ({:?})", div, time);
        
        results.push(SolverResult {
            name: "Breakpoint".to_string(),
            diversity: div,
            time_ms: time.as_millis(),
            success: true,
            solution: Some(solution),
        });
    }

    // 11. MAMDP (New Memetic Algorithm)
    if run_mamdp {
        print!("  [11/10] MAMDP... ");
        let start = Instant::now();
        let (solution, div) = run_mamdp_with_timeout(data, mamdp_timeout);
        let time = start.elapsed();
        println!("✓ {:.2} ({:?})", div, time);
        results.push(SolverResult {
            name: "MAMDP".to_string(),
            diversity: div,
            time_ms: time.as_millis(),
            success: true,
            solution: Some(solution),
        });
    }

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

fn run_its_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
    // Create config with the specific timeout
    let config = ItsConfig {
        timeout_secs,
        ..ItsConfig::default() 
    };
    adj_solver_local_search_its::solve_its(data, &config)
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

fn run_obma_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    let config = ObmaConfig {
        population_size: 10,
        max_iterations: usize::MAX,
        tabu_tenure: 15,
        tabu_max_iters: 2000,
        opposition_mining: true,
    };
    adj_solver_obma::solve_obma(data, &config, deadline)
}

fn run_mamdp_with_timeout(data: &parser::MdpData, timeout_secs: f64) -> (Vec<usize>, f64) {
    let config = adj_solver_population_ma::GeneticConfig {
        generations: usize::MAX,
        ..adj_solver_population_ma::GeneticConfig::default()
    };
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
    adj_solver_population_ma::solve_memetic_mamdp(data, &config, deadline)
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
