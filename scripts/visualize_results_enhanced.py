# visualize_results_enhanced.py
"""
Enhanced visualization with clear interpretations
"""

import json
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np
from pathlib import Path

sns.set_style("whitegrid")
sns.set_palette("husl")

def create_performance_summary_table(df, output_dir):
    """
    TABLE 1: Performance Summary
    Shows: Which solver is best on average, most consistent, fastest
    """
    summary = df.groupby('solver').agg({
        'diversity': ['mean', 'std', 'max'],
        'time_ms': ['mean', 'median'],
        'success': 'mean'
    }).round(2)
    
    summary.columns = ['Avg Diversity', 'Std Dev', 'Best Diversity', 
                       'Avg Time', 'Median Time', 'Success Rate']
    
    # Add ranking column
    summary['Quality Rank'] = summary['Avg Diversity'].rank(ascending=False).astype(int)
    summary['Speed Rank'] = summary['Avg Time'].rank(ascending=True).astype(int)
    
    # Save as CSV and image
    summary.to_csv(output_dir / 'performance_summary.csv')
    
    fig, ax = plt.subplots(figsize=(14, 6))
    ax.axis('tight')
    ax.axis('off')
    
    table = ax.table(cellText=summary.values,
                     rowLabels=summary.index,
                     colLabels=summary.columns,
                     cellLoc='center',
                     loc='center')
    
    table.auto_set_font_size(False)
    table.set_fontsize(9)
    table.scale(1, 2)
    
    # Highlight best performers
    for i, solver in enumerate(summary.index):
        if summary.loc[solver, 'Quality Rank'] == 1:
            table[(i+1, 0)].set_facecolor('#90EE90')  # Light green
        if summary.loc[solver, 'Speed Rank'] == 1:
            table[(i+1, 4)].set_facecolor('#ADD8E6')  # Light blue
    
    plt.title('Performance Summary Table\n'
              'Green = Best Quality | Blue = Fastest',
              fontsize=14, fontweight='bold', pad=20)
    plt.savefig(output_dir / 'table_summary.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: table_summary.png")
    print(f"  Interpretation: Higher 'Avg Diversity' = Better quality")
    print(f"                  Lower 'Avg Time' = Faster solver")
    plt.close()

def create_quality_comparison_bar(df, output_dir):
    """
    CHART 1: Quality Comparison (Simple Bar Chart)
    Shows: Which solver finds the best solutions on average
    INTERPRETATION: Taller bar = Better quality
    """
    avg_diversity = df.groupby('solver')['diversity'].mean().sort_values(ascending=False)
    std_diversity = df.groupby('solver')['diversity'].std()
    
    fig, ax = plt.subplots(figsize=(12, 6))
    bars = ax.bar(range(len(avg_diversity)), avg_diversity.values, 
                  yerr=std_diversity[avg_diversity.index],
                  capsize=5, alpha=0.8, edgecolor='black', linewidth=1.5)
    
    # Color best solver differently
    bars[0].set_color('gold')
    bars[0].set_edgecolor('darkgoldenrod')
    bars[0].set_linewidth(2)
    
    ax.set_xticks(range(len(avg_diversity)))
    ax.set_xticklabels(avg_diversity.index, rotation=45, ha='right')
    ax.set_ylabel('Average Diversity Score', fontsize=13, fontweight='bold')
    ax.set_title('Solution Quality Comparison\n'
                 '📊 Higher = Better Quality | Error bars show consistency',
                 fontsize=14, fontweight='bold', pad=15)
    ax.grid(axis='y', alpha=0.3)
    
    # Add value labels on bars
    for i, (bar, val) in enumerate(zip(bars, avg_diversity.values)):
        ax.text(bar.get_x() + bar.get_width()/2, val, 
                f'{val:.1f}', ha='center', va='bottom', fontweight='bold')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart1_quality_comparison.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart1_quality_comparison.png")
    print(f"  INTERPRETATION: {avg_diversity.index[0]} produces highest quality solutions")
    plt.close()

def create_speed_comparison_bar(df, output_dir):
    """
    CHART 2: Speed Comparison
    Shows: Which solver is fastest
    INTERPRETATION: Shorter bar = Faster
    """
    median_time = df.groupby('solver')['time_ms'].median().sort_values()
    
    fig, ax = plt.subplots(figsize=(12, 6))
    bars = ax.barh(range(len(median_time)), median_time.values,
                   alpha=0.8, edgecolor='black', linewidth=1.5)
    
    # Color fastest solver differently
    bars[0].set_color('lightgreen')
    bars[0].set_edgecolor('darkgreen')
    bars[0].set_linewidth(2)
    
    ax.set_yticks(range(len(median_time)))
    ax.set_yticklabels(median_time.index)
    ax.set_xlabel('Median Computation Time (ms)', fontsize=13, fontweight='bold')
    ax.set_xscale('log')
    ax.set_title('Solver Speed Comparison\n'
                 '⚡ Shorter bar = Faster | Log scale for clarity',
                 fontsize=14, fontweight='bold', pad=15)
    ax.grid(axis='x', alpha=0.3)
    
    # Add value labels
    for i, (bar, val) in enumerate(zip(bars, median_time.values)):
        ax.text(val, bar.get_y() + bar.get_height()/2, 
                f' {val:.1f}ms', va='center', fontweight='bold')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart2_speed_comparison.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart2_speed_comparison.png")
    print(f"  INTERPRETATION: {median_time.index[0]} is the fastest solver")
    plt.close()

def create_quality_vs_speed_scatter(df, output_dir):
    """
    CHART 3: Quality vs Speed Tradeoff (CLEARER VERSION)
    Shows: Which solver offers best balance
    INTERPRETATION: Top-left corner = Best (high quality, low time)
    """
    solver_stats = df.groupby('solver').agg({
        'diversity': 'mean',
        'time_ms': 'median'
    }).reset_index()
    
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Find Pareto optimal solvers
    pareto_mask = []
    for i, row in solver_stats.iterrows():
        dominated = False
        for j, other in solver_stats.iterrows():
            if i != j:
                if (other['diversity'] >= row['diversity'] and 
                    other['time_ms'] <= row['time_ms'] and
                    (other['diversity'] > row['diversity'] or other['time_ms'] < row['time_ms'])):
                    dominated = True
                    break
        pareto_mask.append(not dominated)
    
    solver_stats['pareto'] = pareto_mask
    
    # Plot all solvers
    for _, row in solver_stats.iterrows():
        color = 'red' if row['pareto'] else 'gray'
        size = 300 if row['pareto'] else 150
        marker = 's' if row['pareto'] else 'o'
        alpha = 1.0 if row['pareto'] else 0.5
        
        ax.scatter(row['time_ms'], row['diversity'], 
                  s=size, alpha=alpha, color=color, 
                  edgecolors='black', linewidths=2, marker=marker)
        
        ax.annotate(row['solver'],
                   (row['time_ms'], row['diversity']),
                   xytext=(10, 10), textcoords='offset points',
                   fontsize=11, fontweight='bold' if row['pareto'] else 'normal',
                   bbox=dict(boxstyle='round,pad=0.5', facecolor='yellow' if row['pareto'] else 'white', alpha=0.7))
    
    ax.set_xlabel('Median Time (ms) ← FASTER', fontsize=13, fontweight='bold')
    ax.set_ylabel('Average Diversity ↑ BETTER', fontsize=13, fontweight='bold')
    ax.set_xscale('log')
    ax.set_title('Quality vs Speed Tradeoff Analysis\n'
                 '🎯 Red squares = Pareto optimal (not dominated) | Best solvers are top-left',
                 fontsize=14, fontweight='bold', pad=15)
    ax.grid(True, alpha=0.3)
    
    # Add quadrant labels
    ax.axvline(solver_stats['time_ms'].median(), color='gray', linestyle='--', alpha=0.3)
    ax.axhline(solver_stats['diversity'].median(), color='gray', linestyle='--', alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart3_quality_vs_speed.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart3_quality_vs_speed.png")
    print(f"  INTERPRETATION: Solvers in top-left quadrant offer best tradeoff")
    print(f"                  Pareto optimal = Cannot improve one metric without worsening other")
    plt.close()

def create_win_rate_analysis(df, output_dir):
    """
    CHART 4: Win Rate Chart
    Shows: Which solver wins most often
    INTERPRETATION: Taller bar = Wins more instances
    """
    # Find winner for each instance
    winners = df.loc[df.groupby('filename')['diversity'].idxmax(), ['solver', 'filename']]
    win_counts = winners['solver'].value_counts().sort_values(ascending=False)
    
    fig, ax = plt.subplots(figsize=(12, 6))
    colors = ['gold' if i == 0 else 'silver' if i == 1 else 'chocolate' if i == 2 else 'steelblue' 
              for i in range(len(win_counts))]
    bars = ax.bar(range(len(win_counts)), win_counts.values, 
                  color=colors, edgecolor='black', linewidth=1.5, alpha=0.8)
    
    ax.set_xticks(range(len(win_counts)))
    ax.set_xticklabels(win_counts.index, rotation=45, ha='right')
    ax.set_ylabel('Number of Instances Won', fontsize=13, fontweight='bold')
    ax.set_title('Solver Win Frequency\n'
                 '🏆 How often each solver finds the BEST solution',
                 fontsize=14, fontweight='bold', pad=15)
    ax.grid(axis='y', alpha=0.3)
    
    # Add value labels and percentages
    total = win_counts.sum()
    for i, (bar, val) in enumerate(zip(bars, win_counts.values)):
        pct = val / total * 100
        ax.text(bar.get_x() + bar.get_width()/2, val, 
                f'{val}\n({pct:.1f}%)', 
                ha='center', va='bottom', fontweight='bold')
    
    # Add medal emojis for top 3
    if len(win_counts) >= 1:
        ax.text(0, win_counts.values[0] * 1.1, '🥇', ha='center', fontsize=30)
    if len(win_counts) >= 2:
        ax.text(1, win_counts.values[1] * 1.1, '🥈', ha='center', fontsize=30)
    if len(win_counts) >= 3:
        ax.text(2, win_counts.values[2] * 1.1, '🥉', ha='center', fontsize=30)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart4_win_rate.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart4_win_rate.png")
    print(f"  INTERPRETATION: {win_counts.index[0]} finds best solution most frequently ({win_counts.values[0]}/{total} instances)")
    plt.close()

def create_consistency_analysis(df, output_dir):
    """
    CHART 5: Consistency (Box Plot)
    Shows: How consistent/reliable each solver is
    INTERPRETATION: Smaller box = More consistent
    """
    fig, ax = plt.subplots(figsize=(14, 7))
    
    # Sort by median for better readability
    solver_order = df.groupby('solver')['diversity'].median().sort_values(ascending=False).index
    
    bp = ax.boxplot([df[df['solver'] == solver]['diversity'].values for solver in solver_order],
                    labels=solver_order,
                    patch_artist=True,
                    showmeans=True,
                    meanprops=dict(marker='D', markerfacecolor='red', markersize=8))
    
    # Color boxes
    colors = plt.cm.viridis(np.linspace(0, 1, len(solver_order)))
    for patch, color in zip(bp['boxes'], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.7)
    
    ax.set_ylabel('Diversity Score', fontsize=13, fontweight='bold')
    ax.set_xlabel('Solver', fontsize=13, fontweight='bold')
    ax.set_title('Solver Consistency Analysis\n'
                 '📊 Smaller box = More consistent | Diamond = Mean | Line = Median',
                 fontsize=14, fontweight='bold', pad=15)
    plt.xticks(rotation=45, ha='right')
    ax.grid(axis='y', alpha=0.3)
    
    # Add CV (Coefficient of Variation) annotations
    for i, solver in enumerate(solver_order, 1):
        solver_data = df[df['solver'] == solver]['diversity']
        cv = solver_data.std() / solver_data.mean() * 100
        ax.text(i, solver_data.max() * 1.02, f'CV={cv:.1f}%', 
                ha='center', fontsize=8, bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart5_consistency.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart5_consistency.png")
    print(f"  INTERPRETATION: Lower CV% = More consistent/reliable solver")
    plt.close()

def create_scalability_plot(df, output_dir):
    """
    CHART 6: Scalability Analysis
    Shows: How solver performance degrades with problem size
    INTERPRETATION: Flatter line = Better scalability
    """
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
    solvers = df['solver'].unique()
    colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
    # Plot 1: Time vs Size
    for solver, color in zip(solvers, colors):
        solver_data = df[df['solver'] == solver].groupby('n').agg({
            'time_ms': ['mean', 'std']
        }).reset_index()
        
        ax1.plot(solver_data['n'], solver_data['time_ms']['mean'], 
                marker='o', label=solver, color=color, linewidth=2, markersize=8)
        ax1.fill_between(solver_data['n'],
                         solver_data['time_ms']['mean'] - solver_data['time_ms']['std'],
                         solver_data['time_ms']['mean'] + solver_data['time_ms']['std'],
                         alpha=0.2, color=color)
    
    ax1.set_xlabel('Problem Size (n)', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Average Time (ms)', fontsize=12, fontweight='bold')
    ax1.set_title('Time Scalability\n(Log scale)', fontsize=13, fontweight='bold')
    ax1.set_yscale('log')
    ax1.legend()
    ax1.grid(True, alpha=0.3)
    
    # Plot 2: Quality vs Size
    for solver, color in zip(solvers, colors):
        solver_data = df[df['solver'] == solver].groupby('n').agg({
            'diversity': ['mean', 'std']
        }).reset_index()
        
        ax2.plot(solver_data['n'], solver_data['diversity']['mean'], 
                marker='s', label=solver, color=color, linewidth=2, markersize=8)
        ax2.fill_between(solver_data['n'],
                         solver_data['diversity']['mean'] - solver_data['diversity']['std'],
                         solver_data['diversity']['mean'] + solver_data['diversity']['std'],
                         alpha=0.2, color=color)
    
    ax2.set_xlabel('Problem Size (n)', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Average Diversity', fontsize=12, fontweight='bold')
    ax2.set_title('Quality Scalability\n(Higher = Better)', fontsize=13, fontweight='bold')
    ax2.legend()
    ax2.grid(True, alpha=0.3)
    
    plt.suptitle('Scalability Analysis: How Performance Changes with Problem Size\n'
                 '⬆️ Steeper slope = Worse scalability',
                 fontsize=14, fontweight='bold', y=1.02)
    plt.tight_layout()
    plt.savefig(output_dir / 'chart6_scalability.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart6_scalability.png")
    print(f"  INTERPRETATION: Look for solvers with flat lines (scale well to large problems)")
    plt.close()

def create_performance_by_category(df, output_dir):
    """
    CHART 7: Performance by Instance Category
    Shows: Which solver works best for which problem type
    INTERPRETATION: Darker color = Better performance
    """
    pivot = df.groupby(['category', 'solver'])['diversity'].mean().unstack(fill_value=0)
    
    fig, ax = plt.subplots(figsize=(12, 8))
    sns.heatmap(pivot, annot=True, fmt='.1f', cmap='RdYlGn', 
                cbar_kws={'label': 'Average Diversity (Higher = Better)'},
                linewidths=0.5, ax=ax, vmin=pivot.min().min(), vmax=pivot.max().max())
    
    ax.set_title('Solver Performance by Instance Category\n'
                 '🎨 Green = Best | Yellow = Medium | Red = Worst',
                 fontsize=14, fontweight='bold', pad=15)
    ax.set_xlabel('Solver', fontsize=12, fontweight='bold')
    ax.set_ylabel('Instance Category', fontsize=12, fontweight='bold')
    
    # Find best solver per category
    for i, category in enumerate(pivot.index):
        best_solver = pivot.loc[category].idxmax()
        best_value = pivot.loc[category].max()
        ax.text(list(pivot.columns).index(best_solver) + 0.5, i + 0.5, 
                '⭐', ha='center', va='center', fontsize=20)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart7_category_performance.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart7_category_performance.png")
    print(f"  INTERPRETATION: Stars show best solver for each problem type")
    plt.close()

def create_executive_summary(df, output_dir):
    """
    Generate text summary for presentation
    """
    summary_text = []
    summary_text.append("=" * 80)
    summary_text.append("EXECUTIVE SUMMARY - MDP SOLVER COMPARISON")
    summary_text.append("=" * 80)
    summary_text.append("")
    
    # Best quality
    best_quality = df.groupby('solver')['diversity'].mean().idxmax()
    best_quality_val = df.groupby('solver')['diversity'].mean().max()
    summary_text.append(f"🏆 BEST QUALITY: {best_quality} (Avg: {best_quality_val:.2f})")
    
    # Fastest
    fastest = df.groupby('solver')['time_ms'].median().idxmin()
    fastest_time = df.groupby('solver')['time_ms'].median().min()
    summary_text.append(f"⚡ FASTEST: {fastest} (Median: {fastest_time:.2f}ms)")
    
    # Most consistent
    cvs = df.groupby('solver')['diversity'].apply(lambda x: x.std() / x.mean() * 100)
    most_consistent = cvs.idxmin()
    summary_text.append(f"📊 MOST CONSISTENT: {most_consistent} (CV: {cvs.min():.2f}%)")
    
    # Win rate
    winners = df.loc[df.groupby('filename')['diversity'].idxmax(), 'solver']
    win_champion = winners.value_counts().idxmax()
    win_count = winners.value_counts().max()
    win_pct = win_count / len(df['filename'].unique()) * 100
    summary_text.append(f"🎯 MOST WINS: {win_champion} ({win_count} instances, {win_pct:.1f}%)")
    
    summary_text.append("")
    summary_text.append("RECOMMENDATIONS:")
    summary_text.append(f"  • For best quality: Use {best_quality}")
    summary_text.append(f"  • For speed: Use {fastest}")
    summary_text.append(f"  • For reliability: Use {most_consistent}")
    
    summary_text.append("")
    summary_text.append("=" * 80)
    
    summary_str = "\n".join(summary_text)
    
    # Save to file
    with open(output_dir / 'EXECUTIVE_SUMMARY.txt', 'w') as f:
        f.write(summary_str)
    
    print("\n" + summary_str)
    print(f"\n✓ Saved: EXECUTIVE_SUMMARY.txt")

def main():
    import sys
    import glob
    
    if len(sys.argv) < 2:
        print("Usage: python visualize_results_enhanced.py results_*.json")
        sys.exit(1)
    
    json_files = []
    for pattern in sys.argv[1:]:
        json_files.extend(glob.glob(pattern))
    
    if not json_files:
        print("No result files found!")
        sys.exit(1)
    
    print(f"\nLoading results from {len(json_files)} file(s)...")
    
    instances = []
    for json_file in json_files:
        with open(json_file, 'r') as f:
            data = json.load(f)
            instances.extend(data['instances'])
    
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
    
    print(f"Loaded {len(df)} successful results from {len(instances)} instances")
    print(f"Solvers: {', '.join(df['solver'].unique())}\n")
    
    output_dir = Path('visualizations_enhanced')
    output_dir.mkdir(exist_ok=True)
    
    print(f"Generating enhanced visualizations in: {output_dir}/\n")
    
    create_performance_summary_table(df, output_dir)
    create_quality_comparison_bar(df, output_dir)
    create_speed_comparison_bar(df, output_dir)
    create_quality_vs_speed_scatter(df, output_dir)
    create_win_rate_analysis(df, output_dir)
    create_consistency_analysis(df, output_dir)
    create_scalability_plot(df, output_dir)
    create_performance_by_category(df, output_dir)
    create_executive_summary(df, output_dir)
    
    print(f"\n✅ All visualizations and summary generated!")
    print(f"   Check {output_dir}/ for results")

if __name__ == '__main__':
    main()