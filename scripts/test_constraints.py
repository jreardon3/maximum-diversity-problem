# test_constraints.py
"""
Verify all solutions satisfy k-item constraint
"""

def verify_constraints(solution, k, n):
    """Check if solution is valid"""
    assert len(solution) == k, f"Wrong size: {len(solution)} != {k}"
    assert all(0 <= idx < n for idx in solution), "Invalid indices"
    assert len(set(solution)) == k, "Duplicate items selected"
    return True

def constraint_violation_report(results_df):
    """Report any constraint violations"""
    violations = []
    
    for _, row in results_df.iterrows():
        if not verify_constraints(row['solution'], row['k'], row['n']):
            violations.append({
                'instance': row['filename'],
                'solver': row['solver'],
                'size': len(row['solution'])
            })
    
    if violations:
        print("⚠️  CONSTRAINT VIOLATIONS FOUND:")
        for v in violations:
            print(f"  {v['solver']} on {v['instance']}: selected {v['size']} items")
    else:
        print("✓ All solutions satisfy constraints")