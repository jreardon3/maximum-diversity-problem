# test_optimality_gap.py
"""
For small instances where you can compute exact optimal solution
"""

def compute_exact_optimal(mdp_data, timeout=600):
    """Use Gurobi with no time limit to find optimal"""
    # Your exact MDP solver
    optimal_diversity = solve_mdp_exact(mdp_data, timeout)
    return optimal_diversity

def gap_analysis(results_df, optimal_values):
    """Compute optimality gap for each solver"""
    print("\n" + "=" * 80)
    print("OPTIMALITY GAP ANALYSIS")
    print("=" * 80)
    
    for instance, opt_val in optimal_values.items():
        instance_results = results_df[results_df['filename'] == instance]
        
        print(f"\n{instance} (Optimal = {opt_val:.2f}):")
        for _, row in instance_results.iterrows():
            gap = (opt_val - row['diversity']) / opt_val * 100
            print(f"  {row['solver']:15s}: {row['diversity']:7.2f} "
                  f"(gap: {gap:5.2f}%)")