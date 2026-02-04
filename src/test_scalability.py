# test_scalability.py
"""
Generate synthetic instances of increasing size
"""

def generate_synthetic_instance(n, k, density=1.0):
    """Create random MDP instance"""
    distances = np.random.rand(n, n) * 100
    # Make symmetric
    distances = (distances + distances.T) / 2
    np.fill_diagonal(distances, 0)
    
    return {'n': n, 'k': k, 'distances': distances}

def scalability_test():
    """Test how solvers scale"""
    sizes = [10, 20, 50, 100, 200, 500]
    results = []
    
    for n in sizes:
        k = n // 4
        instance = generate_synthetic_instance(n, k)
        
        for solver in ['QUBO', 'MaxCut', 'GRASP', 'Tabu', 'GA']:
            start = time.time()
            diversity = run_solver(instance, solver)
            elapsed = time.time() - start
            
            results.append({
                'n': n,
                'k': k,
                'solver': solver,
                'diversity': diversity,
                'time': elapsed
            })
    
    return pd.DataFrame(results)