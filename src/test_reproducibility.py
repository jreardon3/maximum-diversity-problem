# test_reproducibility.py
"""
Run each solver multiple times on same instances to test stability
"""

def test_solver_stability(mdp_file, solver_name, num_runs=10):
    """Run solver multiple times and check variance"""
    results = []
    
    for run in range(num_runs):
        # Run your solver
        diversity, time_ms = run_solver(mdp_file, solver_name)
        results.append({'diversity': diversity, 'time': time_ms})
    
    diversities = [r['diversity'] for r in results]
    times = [r['time'] for r in results]
    
    print(f"\n{solver_name} Stability Test ({num_runs} runs):")
    print(f"  Diversity: Mean={np.mean(diversities):.2f}, "
          f"Std={np.std(diversities):.2f}, "
          f"CV={np.std(diversities)/np.mean(diversities)*100:.2f}%")
    print(f"  Time: Mean={np.mean(times):.2f}ms, "
          f"Std={np.std(times):.2f}ms")
    
    return results