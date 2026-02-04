"""
Visualization script for MDP solver comparison results.
Can process multiple result JSON files at once.

Usage: python visualize_results.py results_*.json
"""

import json
import sys
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np
from pathlib import Path
import glob

sns.set_style("whitegrid")
plt.rcParams['figure.figsize'] = (12, 8)

def load_all_results(json_files):
    """Load and combine multiple result files"""
    all_instances = []
    
    for json_file in json_files:
        with open(json_file, 'r') as f:
            data = json.load(f)
            all_instances.extend(data['instances'])
    
    return all_instances

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
        print("Usage: python visualize_results.py results_*.json")
        print("  or:  python visualize_results.py results_GKD_*.json results_MDG_*.json results_SOM_*.json")
        sys.exit(1)
    
    json_files = []
    for pattern in sys.argv[1:]:
        json_files.extend(glob.glob(pattern))
    
    if not json_files:
        print("No result files found!")
        sys.exit(1)
    
    print(f"\nLoading results from {len(json_files)} file(s):")
    for f in json_files:
        print(f"  - {f}")
    
    instances = load_all_results(json_files)
    
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
    
    print(f"\nLoaded {len(df)} result records from {len(instances)} instances")
    print(f"Solvers: {', '.join(df['solver'].unique())}")
    print(f"Categories: {', '.join(df['category'].unique())}\n")
    
    output_dir = Path('visualizations_combined')
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