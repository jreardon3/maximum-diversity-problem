# test_statistical_validation.py
import json
import scipy.stats as stats
import pandas as pd
import numpy as np
from pathlib import Path

def load_results(json_file):
    """Load results from JSON"""
    with open(json_file, 'r') as f:
        data = json.load(f)
    return data['instances']

def statistical_analysis(results_df):
    """Perform statistical tests on solver performance"""
    
    print("=" * 80)
    print("STATISTICAL VALIDATION ANALYSIS")
    print("=" * 80)
    
    solvers = results_df['solver'].unique()
    
    # 1. Normality test (Shapiro-Wilk)
    print("\n1. NORMALITY TESTS (Shapiro-Wilk)")
    print("-" * 80)
    for solver in solvers:
        solver_data = results_df[results_df['solver'] == solver]['diversity']
        statistic, p_value = stats.shapiro(solver_data)
        normal = "Yes" if p_value > 0.05 else "No"
        print(f"{solver:20s}: p-value={p_value:.4f}, Normal={normal}")
    
    # 2. Friedman test (non-parametric repeated measures)
    print("\n2. FRIEDMAN TEST (Overall Difference)")
    print("-" * 80)
    pivot = results_df.pivot_table(values='diversity', index='filename', columns='solver')
    statistic, p_value = stats.friedmanchisquare(*[pivot[col].values for col in pivot.columns])
    print(f"Chi-square statistic: {statistic:.4f}")
    print(f"P-value: {p_value:.6f}")
    print(f"Significant difference: {'Yes' if p_value < 0.05 else 'No'}")
    
    # 3. Pairwise Wilcoxon signed-rank tests
    print("\n3. PAIRWISE COMPARISONS (Wilcoxon Signed-Rank)")
    print("-" * 80)
    from itertools import combinations
    for solver1, solver2 in combinations(solvers, 2):
        data1 = pivot[solver1].values
        data2 = pivot[solver2].values
        statistic, p_value = stats.wilcoxon(data1, data2)
        winner = solver1 if data1.mean() > data2.mean() else solver2
        print(f"{solver1:15s} vs {solver2:15s}: p={p_value:.4f}, Winner: {winner}")
    
    # 4. Effect size (Cohen's d)
    print("\n4. EFFECT SIZES (Cohen's d)")
    print("-" * 80)
    baseline = solvers[0]  # Use first solver as baseline
    baseline_data = results_df[results_df['solver'] == baseline]['diversity']
    
    for solver in solvers[1:]:
        solver_data = results_df[results_df['solver'] == solver]['diversity']
        
        # Cohen's d
        pooled_std = np.sqrt((baseline_data.std()**2 + solver_data.std()**2) / 2)
        cohens_d = (solver_data.mean() - baseline_data.mean()) / pooled_std
        
        magnitude = "negligible" if abs(cohens_d) < 0.2 else \
                   "small" if abs(cohens_d) < 0.5 else \
                   "medium" if abs(cohens_d) < 0.8 else "large"
        
        print(f"{solver:20s} vs {baseline}: d={cohens_d:+.3f} ({magnitude})")
    
    # 5. Consistency analysis (coefficient of variation)
    print("\n5. SOLVER CONSISTENCY (Coefficient of Variation)")
    print("-" * 80)
    for solver in solvers:
        solver_data = results_df[results_df['solver'] == solver]['diversity']
        cv = solver_data.std() / solver_data.mean() * 100
        print(f"{solver:20s}: CV={cv:.2f}% (lower is more consistent)")

def convergence_analysis(results_df):
    """Analyze convergence behavior"""
    print("\n" + "=" * 80)
    print("CONVERGENCE ANALYSIS")
    print("=" * 80)
    
    solvers = results_df['solver'].unique()
    
    print("\n6. TIME TO BEST SOLUTION")
    print("-" * 80)
    for solver in solvers:
        solver_times = results_df[results_df['solver'] == solver]['time_ms']
        print(f"{solver:20s}: Mean={solver_times.mean():.2f}ms, "
              f"Median={solver_times.median():.2f}ms, "
              f"Std={solver_times.std():.2f}ms")
    
    # Time-quality tradeoff metric
    print("\n7. EFFICIENCY SCORE (Diversity / log(Time))")
    print("-" * 80)
    for solver in solvers:
        solver_data = results_df[results_df['solver'] == solver]
        efficiency = (solver_data['diversity'] / np.log10(solver_data['time_ms'] + 1)).mean()
        print(f"{solver:20s}: Efficiency={efficiency:.2f}")

def robustness_testing(results_df):
    """Test solver robustness across problem sizes"""
    print("\n" + "=" * 80)
    print("ROBUSTNESS ANALYSIS")
    print("=" * 80)
    
    print("\n8. PERFORMANCE BY PROBLEM SIZE")
    print("-" * 80)
    size_groups = results_df.groupby(['solver', pd.cut(results_df['n'], bins=3)])
    for (solver, size_bin), group in size_groups:
        print(f"{solver:15s} | n∈{size_bin}: "
              f"Mean Div={group['diversity'].mean():.2f}, "
              f"Success Rate={group['success'].mean()*100:.1f}%")

if __name__ == '__main__':
    # Load your results
    # change to be dynamic i.e. result_*.json
    import sys
    if len(sys.argv) > 1:
        instances = load_results(sys.argv[1])
    else:
        instances = load_results('results_combined.json')
    
    rows = []
    for instance in instances:
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
    df = df[df['success'] == True]  # Only successful runs
    
    statistical_analysis(df)
    convergence_analysis(df)
    robustness_testing(df)