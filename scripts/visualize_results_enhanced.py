# visualize_results_enhanced.py
"""
Enhanced visualization with clear interpretations
Includes Academic Tables (APD, Time, Hit Rate)
"""

import json
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np
from pathlib import Path
import sys
import glob

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
    
    # Save as CSV
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
    plt.close()

    """
    GENERATES THE REQUESTED ACADEMIC TABLES:
    1. APD (Average Percentage Deviation) & Median Time
    2. Hit Rate (Count of Best Solutions)
    """
    print("\n--- Generating Academic Tables (Paper Ready) ---")
    
    # Work on a copy to avoid affecting other plots
    df_acad = df.copy()
    
    # 1. Rename Solvers to match Paper Terminology
    name_map = {
        'QUBO': 'Hybrid Gurobi \n(QUBO)',
        'MaxCut': 'Hybrid Gurobi \n(MaxCut)',
        'MAMDP': 'MA',
        'Breakpoint': 'BP'
    }
    df_acad['solver'] = df_acad['solver'].replace(name_map)
    
    # 2. Calculate Statistics
    # Find Best Known Value (BKV) per instance
    best_known = df_acad.groupby('filename')['diversity'].transform('max')
    df_acad['bkv'] = best_known
    
    # Calculate Percentage Deviation: (BKV - Val) / BKV * 100
    df_acad['deviation_pct'] = (df_acad['bkv'] - df_acad['diversity']) / df_acad['bkv'] * 100
    
    # Flag Best Solutions (allowing for tiny float errors)
    df_acad['is_best'] = df_acad['deviation_pct'] < 1e-5
    
    # Convert Time to Seconds
    df_acad['time_s'] = df_acad['time_ms'] / 1000.0

    # ---------------------------------------------------------
    # TABLE A: APD & Median Time (Combined)
    # ---------------------------------------------------------
    grouped = df_acad.groupby(['category', 'solver']).agg({
        'deviation_pct': 'mean',
        'time_s': 'median'
    })
    
    # Create text for the cell: "0.12% \n(28.4s)"
    grouped['display'] = grouped.apply(
        lambda x: f"{x['deviation_pct']:.2f}%\n({x['time_s']:.2f}s)", axis=1
    )
    
    display_pivot = grouped['display'].unstack()
    dev_pivot = grouped['deviation_pct'].unstack() # For coloring logic
    
    # Save CSV
    grouped[['deviation_pct', 'time_s']].unstack().to_csv(output_dir / 'academic_table_1_data.csv')
    
    # Render
    fig, ax = plt.subplots(figsize=(14, len(display_pivot) * 1.5 + 2))
    ax.axis('off')
    ax.axis('tight')
    
    table = ax.table(cellText=display_pivot.values,
                     rowLabels=display_pivot.index,
                     colLabels=display_pivot.columns,
                     cellLoc='center',
                     loc='center')
    
    table.auto_set_font_size(False)
    table.set_fontsize(11)
    table.scale(1, 2.8) # Tall cells for 2 lines
    
    # Highlight Best Quality (Lowest Dev) in Green
    for i, cat in enumerate(display_pivot.index):
        best_dev = dev_pivot.loc[cat].min()
        for j, solver in enumerate(display_pivot.columns):
            current_dev = dev_pivot.loc[cat, solver]
            if abs(current_dev - best_dev) < 1e-6:
                table[(i+1, j)].set_facecolor('#90EE90') # Green
                table[(i+1, j)].set_text_props(weight='bold')

    plt.title('Table 1: Average % Deviation & (Median Time s)',
              fontweight='bold', pad=20)
    
    plt.savefig(output_dir / 'academic_table_1_apd_time.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: academic_table_1_apd_time.png")

    # ---------------------------------------------------------
    # TABLE B: Hit Rate (Number of Best Solutions)
    # ---------------------------------------------------------
    hit_counts = df_acad[df_acad['is_best']].groupby(['category', 'solver']).size().unstack(fill_value=0)
    
    # Ensure all solvers exist as columns
    for s in df_acad['solver'].unique():
        if s not in hit_counts.columns:
            hit_counts[s] = 0
            
    # Add Total Row
    hit_counts.loc['Total'] = hit_counts.sum()
    
    # Save CSV
    hit_counts.to_csv(output_dir / 'academic_table_2_hitrate.csv')
    
    # Render
    fig, ax = plt.subplots(figsize=(14, len(hit_counts) + 2))
    ax.axis('off')
    ax.axis('tight')
    
    table = ax.table(cellText=hit_counts.values,
                     rowLabels=hit_counts.index,
                     colLabels=hit_counts.columns,
                     cellLoc='center',
                     loc='center')
    
    table.auto_set_font_size(False)
    table.set_fontsize(11)
    table.scale(1, 2)
    
    # Highlight Winner (Most Hits) in Blue
    for i, cat in enumerate(hit_counts.index):
        row_vals = hit_counts.loc[cat]
        max_hits = row_vals.max()
        for j, solver in enumerate(hit_counts.columns):
            if row_vals[solver] == max_hits:
                table[(i+1, j)].set_facecolor('#ADD8E6') # Blue
                table[(i+1, j)].set_text_props(weight='bold')

    plt.title('Table 2: Number of Best Solutions Found (Hit Rate)\n'
              'Higher count = Better robustness', 
              fontweight='bold', pad=20)
    
    plt.savefig(output_dir / 'academic_table_2_hitrate.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: academic_table_2_hitrate.png")
    plt.close()

def create_academic_tables(df, output_dir):
    """
    GENERATES THE REQUESTED ACADEMIC TABLES:
    1. APD (Average Percentage Deviation) & Median Time
    2. Hit Rate (Count of Best Solutions)
    """
    print("\n--- Generating Academic Tables (Paper Ready) ---")
    
    # Work on a copy to avoid affecting other plots
    df_acad = df.copy()
    
    # 1. Rename Solvers to match Paper Terminology
    name_map = {
        'QUBO': 'Hybrid Gurobi\n(QUBO)',
        'MaxCut': 'Hybrid Gurobi\n(MaxCut)',
        'MAMDP': 'MA',
        'Breakpoint': 'BP'
    }
    df_acad['solver'] = df_acad['solver'].replace(name_map)
    
    # 2. Calculate Statistics
    # Find Best Known Value (BKV) per instance
    best_known = df_acad.groupby('filename')['diversity'].transform('max')
    df_acad['bkv'] = best_known
    
    # Calculate Percentage Deviation: (BKV - Val) / BKV * 100
    df_acad['deviation_pct'] = (df_acad['bkv'] - df_acad['diversity']) / df_acad['bkv'] * 100
    
    # Flag Best Solutions (allowing for tiny float errors)
    df_acad['is_best'] = df_acad['deviation_pct'] < 1e-5
    
    # Convert Time to Seconds
    df_acad['time_s'] = df_acad['time_ms'] / 1000.0

    # ---------------------------------------------------------
    # TABLE A: APD & Median Time (Combined)
    # ---------------------------------------------------------
    grouped = df_acad.groupby(['category', 'solver']).agg({
        'deviation_pct': 'mean',
        'time_s': 'median'
    })
    
    # Create text for the cell: "0.1234% \n(28.4s)"
    # NOW USES 4 DECIMAL PLACES
    grouped['display'] = grouped.apply(
        lambda x: f"{x['deviation_pct']:.4f}%\n({x['time_s']:.2f}s)", axis=1
    )
    
    display_pivot = grouped['display'].unstack()
    dev_pivot = grouped['deviation_pct'].unstack() # For coloring logic
    
    # Save CSV
    grouped[['deviation_pct', 'time_s']].unstack().to_csv(output_dir / 'academic_table_1_data.csv')
    
    # Render
    fig, ax = plt.subplots(figsize=(14, len(display_pivot) * 1.5 + 2))
    ax.axis('off')
    ax.axis('tight')
    
    table = ax.table(cellText=display_pivot.values,
                     rowLabels=display_pivot.index,
                     colLabels=display_pivot.columns,
                     cellLoc='center',
                     loc='center')
    
    table.auto_set_font_size(False)
    table.set_fontsize(11)
    table.scale(1, 2.8) # Tall cells for 2 lines
    
    # Highlight Best Quality (Lowest Dev) in Green
    for i, cat in enumerate(display_pivot.index):
        # Find the mathematical minimum
        min_val_math = dev_pivot.loc[cat].min()
        # Round it to 4 decimals (matching the display format)
        min_val_rounded = round(min_val_math, 4)
        
        for j, solver in enumerate(display_pivot.columns):
            current_val = dev_pivot.loc[cat, solver]
            
            # Highlight if the rounded value matches the best rounded value
            if round(current_val, 4) == min_val_rounded:
                table[(i+1, j)].set_facecolor('#90EE90') # Green
                table[(i+1, j)].set_text_props(weight='bold')

    plt.title('Table 1: Average % Deviation & (Median Time s)\n'
              'Top value = Deviation (Lower is better) | Bottom = Time', 
              fontweight='bold', pad=20)
    
    plt.savefig(output_dir / 'academic_table_1_apd_time.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: academic_table_1_apd_time.png")

    # ---------------------------------------------------------
    # TABLE B: Hit Rate (Number of Best Solutions)
    # ---------------------------------------------------------
    hit_counts = df_acad[df_acad['is_best']].groupby(['category', 'solver']).size().unstack(fill_value=0)
    
    # Ensure all solvers exist as columns
    for s in df_acad['solver'].unique():
        if s not in hit_counts.columns:
            hit_counts[s] = 0
            
    # Add Total Row
    hit_counts.loc['Total'] = hit_counts.sum()
    
    # Save CSV
    hit_counts.to_csv(output_dir / 'academic_table_2_hitrate.csv')
    
    # Render
    fig, ax = plt.subplots(figsize=(14, len(hit_counts) + 2))
    ax.axis('off')
    ax.axis('tight')
    
    table = ax.table(cellText=hit_counts.values,
                     rowLabels=hit_counts.index,
                     colLabels=hit_counts.columns,
                     cellLoc='center',
                     loc='center')
    
    table.auto_set_font_size(False)
    table.set_fontsize(11)
    table.scale(1, 2)
    
    # Highlight Winner (Most Hits) in Blue
    for i, cat in enumerate(hit_counts.index):
        row_vals = hit_counts.loc[cat]
        max_hits = row_vals.max()
        for j, solver in enumerate(hit_counts.columns):
            if row_vals[solver] == max_hits:
                table[(i+1, j)].set_facecolor('#ADD8E6') # Blue
                table[(i+1, j)].set_text_props(weight='bold')

    plt.title('Table 2: Number of Best Solutions Found (Hit Rate)\n'
              'Higher count = Better robustness', 
              fontweight='bold', pad=20)
    
    plt.savefig(output_dir / 'academic_table_2_hitrate.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: academic_table_2_hitrate.png")
    plt.close()

def create_quality_comparison_bar(df, output_dir):
    """
    CHART 1: Quality Comparison
    """
    avg_diversity = df.groupby('solver')['diversity'].mean().sort_values(ascending=False)
    std_diversity = df.groupby('solver')['diversity'].std()
    
    fig, ax = plt.subplots(figsize=(12, 6))
    bars = ax.bar(range(len(avg_diversity)), avg_diversity.values, 
                  yerr=std_diversity[avg_diversity.index],
                  capsize=5, alpha=0.8, edgecolor='black', linewidth=1.5)
    
    bars[0].set_color('gold')
    bars[0].set_edgecolor('darkgoldenrod')
    bars[0].set_linewidth(2)
    
    ax.set_xticks(range(len(avg_diversity)))
    ax.set_xticklabels(avg_diversity.index, rotation=45, ha='right')
    ax.set_ylabel('Average Diversity Score', fontsize=13, fontweight='bold')
    ax.set_title('Solution Quality Comparison', fontsize=14, fontweight='bold', pad=15)
    ax.grid(axis='y', alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart1_quality_comparison.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart1_quality_comparison.png")
    plt.close()

def create_speed_comparison_bar(df, output_dir):
    """
    CHART 2: Speed Comparison
    """
    median_time = df.groupby('solver')['time_ms'].median().sort_values()
    
    fig, ax = plt.subplots(figsize=(12, 6))
    bars = ax.barh(range(len(median_time)), median_time.values,
                   alpha=0.8, edgecolor='black', linewidth=1.5)
    
    bars[0].set_color('lightgreen')
    
    ax.set_yticks(range(len(median_time)))
    ax.set_yticklabels(median_time.index)
    ax.set_xlabel('Median Computation Time (ms)', fontsize=13, fontweight='bold')
    ax.set_xscale('log')
    ax.set_title('Solver Speed Comparison (Log Scale)', fontsize=14, fontweight='bold', pad=15)
    
    for i, (bar, val) in enumerate(zip(bars, median_time.values)):
        ax.text(val, bar.get_y() + bar.get_height()/2, 
                f' {val:.1f}ms', va='center', fontweight='bold')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart2_speed_comparison.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart2_speed_comparison.png")
    plt.close()

def create_quality_vs_speed_scatter_individual(df, output_dir):
    """
    CHART 3B: Quality vs Speed Tradeoff (3 Separate Files)
    """
    categories = sorted(df['category'].unique())
    
    # Consistent colors
    solvers = sorted(df['solver'].unique())
    colors = sns.color_palette("husl", n_colors=len(solvers))
    solver_color_map = dict(zip(solvers, colors))
    
    for cat in categories:
        # CREATE NEW FIGURE FOR EACH CATEGORY
        fig, ax = plt.subplots(figsize=(10, 8))
        
        cat_df = df[df['category'] == cat]
        
        solver_stats = cat_df.groupby('solver').agg({
            'diversity': 'mean',
            'time_ms': 'median'
        }).reset_index()
        
        # Pareto calculation
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
        
        # Plot
        for _, row in solver_stats.iterrows():
            is_pareto = row['pareto']
            solver_name = row['solver']
            
            color = solver_color_map[solver_name]
            size = 300 if is_pareto else 150
            marker = 'D' if is_pareto else 'o'
            edgewidth = 2 if is_pareto else 1
            
            ax.scatter(row['time_ms'], row['diversity'], 
                       s=size, alpha=0.9, color=color, 
                       edgecolors='black', linewidths=edgewidth, marker=marker)
            
            # Smart annotation placement
            ax.annotate(solver_name,
                        (row['time_ms'], row['diversity']),
                        xytext=(8, 8), textcoords='offset points',
                        fontsize=12, fontweight='bold' if is_pareto else 'normal',
                        bbox=dict(boxstyle='round,pad=0.3', facecolor='white', alpha=0.8, edgecolor='gray'))

        ax.set_title(f'Quality vs Speed: {cat} Dataset', fontsize=16, fontweight='bold', pad=15)
        ax.set_xlabel('Median Time (ms) [Log Scale] ← Faster', fontsize=13, fontweight='bold')
        ax.set_ylabel('Average Diversity [Higher is Better]', fontsize=13, fontweight='bold')
        ax.set_xscale('log')
        ax.grid(True, alpha=0.3, which="both")
        
        plt.tight_layout()
        filename = f'chart3_quality_vs_speed_{cat}.png'
        plt.savefig(output_dir / filename, dpi=300)
        print(f"✓ Saved: {filename}")
        plt.close()

def create_quality_vs_speed_scatter(df, output_dir):
    """
    CHART 3: Quality vs Speed Tradeoff
    """
    solver_stats = df.groupby('solver').agg({
        'diversity': 'mean',
        'time_ms': 'median'
    }).reset_index()
    
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Pareto calculation
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
    ax.set_title('Quality vs Speed Tradeoff Analysis', fontsize=14, fontweight='bold', pad=15)
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart3_quality_vs_speed.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart3_quality_vs_speed.png")
    plt.close()

    """
    CHART 3B: Quality vs Speed Tradeoff (Split by Category)
    """
    categories = sorted(df['category'].unique())
    n_cats = len(categories)
    
    fig, axes = plt.subplots(1, n_cats, figsize=(6 * n_cats, 6), constrained_layout=True)
    if n_cats == 1: axes = [axes]
    
    # Consistent colors
    solvers = sorted(df['solver'].unique())
    colors = sns.color_palette("husl", n_colors=len(solvers))
    solver_color_map = dict(zip(solvers, colors))
    
    for ax, cat in zip(axes, categories):
        cat_df = df[df['category'] == cat]
        
        solver_stats = cat_df.groupby('solver').agg({
            'diversity': 'mean',
            'time_ms': 'median'
        }).reset_index()
        
        # Pareto calculation
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
        
        # Plot
        for _, row in solver_stats.iterrows():
            is_pareto = row['pareto']
            solver_name = row['solver']
            
            color = solver_color_map[solver_name]
            size = 200 if is_pareto else 100
            marker = 'D' if is_pareto else 'o'
            edgewidth = 2 if is_pareto else 1
            
            ax.scatter(row['time_ms'], row['diversity'], 
                       s=size, alpha=0.8, color=color, 
                       edgecolors='black', linewidths=edgewidth, marker=marker,
                       label=solver_name if is_pareto else None)
            
            ax.annotate(solver_name,
                        (row['time_ms'], row['diversity']),
                        xytext=(5, 5), textcoords='offset points',
                        fontsize=9, fontweight='bold' if is_pareto else 'normal',
                        bbox=dict(boxstyle='round,pad=0.3', facecolor='white', alpha=0.6))

        ax.set_title(f'Category: {cat}', fontsize=14, fontweight='bold')
        ax.set_xlabel('Median Time (ms) [Log Scale]', fontsize=11)
        if cat == categories[0]:
            ax.set_ylabel('Average Diversity (Higher is Better)', fontsize=11)
        ax.set_xscale('log')
        ax.grid(True, alpha=0.3)
    
    plt.suptitle('Quality vs Speed Tradeoff by Instance Group', fontsize=16, fontweight='bold')
    plt.savefig(output_dir / 'chart3_quality_vs_speed_SPLIT.png', dpi=300)
    print(f"✓ Saved: chart3_quality_vs_speed_SPLIT.png")
    plt.close()

def create_win_rate_analysis(df, output_dir):
    """
    CHART 4: Win Rate Chart
    """
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
    ax.set_title('Solver Win Frequency', fontsize=14, fontweight='bold', pad=15)
    
    # Add medal emojis
    if len(win_counts) >= 1: ax.text(0, win_counts.values[0] * 1.05, '🥇', ha='center', fontsize=20)
    if len(win_counts) >= 2: ax.text(1, win_counts.values[1] * 1.05, '🥈', ha='center', fontsize=20)
    if len(win_counts) >= 3: ax.text(2, win_counts.values[2] * 1.05, '🥉', ha='center', fontsize=20)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart4_win_rate.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart4_win_rate.png")
    plt.close()

def create_consistency_analysis(df, output_dir):
    """
    CHART 5: Consistency (Box Plot)
    """
    fig, ax = plt.subplots(figsize=(14, 7))
    
    solver_order = df.groupby('solver')['diversity'].median().sort_values(ascending=False).index
    
    bp = ax.boxplot([df[df['solver'] == solver]['diversity'].values for solver in solver_order],
                    labels=solver_order,
                    patch_artist=True,
                    showmeans=True)
    
    colors = plt.cm.viridis(np.linspace(0, 1, len(solver_order)))
    for patch, color in zip(bp['boxes'], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.7)
    
    ax.set_ylabel('Diversity Score', fontsize=13, fontweight='bold')
    ax.set_title('Solver Consistency Analysis', fontsize=14, fontweight='bold', pad=15)
    plt.xticks(rotation=45, ha='right')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart5_consistency.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart5_consistency.png")
    plt.close()

def create_scalability_plot(df, output_dir):
    """
    CHART 6: Scalability Analysis
    """
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
    solvers = df['solver'].unique()
    colors = plt.cm.tab10(np.linspace(0, 1, len(solvers)))
    
    for solver, color in zip(solvers, colors):
        solver_data = df[df['solver'] == solver].groupby('n').agg({
            'time_ms': ['mean', 'std'],
            'diversity': ['mean', 'std']
        }).reset_index()
        
        # Time Plot
        ax1.plot(solver_data['n'], solver_data['time_ms']['mean'], 
                 marker='o', label=solver, color=color, linewidth=2)
        ax1.fill_between(solver_data['n'],
                         solver_data['time_ms']['mean'] - solver_data['time_ms']['std'],
                         solver_data['time_ms']['mean'] + solver_data['time_ms']['std'],
                         alpha=0.1, color=color)
                         
        # Quality Plot
        ax2.plot(solver_data['n'], solver_data['diversity']['mean'], 
                 marker='s', label=solver, color=color, linewidth=2)
    
    ax1.set_xlabel('Problem Size (n)')
    ax1.set_ylabel('Time (ms)')
    ax1.set_yscale('log')
    ax1.set_title('Time Scalability')
    ax1.legend()
    
    ax2.set_xlabel('Problem Size (n)')
    ax2.set_ylabel('Diversity')
    ax2.set_title('Quality Scalability')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart6_scalability.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart6_scalability.png")
    plt.close()

def create_performance_by_category(df, output_dir):
    """
    CHART 7: Performance by Instance Category
    """
    pivot = df.groupby(['category', 'solver'])['diversity'].mean().unstack(fill_value=0)
    
    fig, ax = plt.subplots(figsize=(12, 8))
    sns.heatmap(pivot, annot=True, fmt='.1f', cmap='RdYlGn', 
                linewidths=0.5, ax=ax)
    
    ax.set_title('Solver Performance by Instance Category', fontsize=14, fontweight='bold', pad=15)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'chart7_category_performance.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: chart7_category_performance.png")
    plt.close()

def create_executive_summary(df, output_dir):
    """
    Generate text summary
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
    
    summary_str = "\n".join(summary_text)
    
    with open(output_dir / 'EXECUTIVE_SUMMARY.txt', 'w') as f:
        f.write(summary_str)
    
    print("\n" + summary_str)
    print(f"\n✓ Saved: EXECUTIVE_SUMMARY.txt")

def main():
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
    df = df[df['success'] == True]
    
    print(f"Loaded {len(df)} successful results")
    
    output_dir = Path('visualizations_enhanced_test_2')
    output_dir.mkdir(exist_ok=True)
    
    print(f"Generating enhanced visualizations in: {output_dir}/\n")
    
    # --- Generate Basic Plots ---
    create_performance_summary_table(df, output_dir)
    create_quality_comparison_bar(df, output_dir)
    create_speed_comparison_bar(df, output_dir)
    create_quality_vs_speed_scatter(df, output_dir)
    create_quality_vs_speed_scatter_split(df, output_dir)
    create_win_rate_analysis(df, output_dir)
    create_consistency_analysis(df, output_dir)
    create_scalability_plot(df, output_dir)
    create_performance_by_category(df, output_dir)
    
    # --- Generate NEW Academic Tables ---
    create_academic_tables(df, output_dir)
    
    create_executive_summary(df, output_dir)
    
    print(f"\n✅ All visualizations generated!")

if __name__ == '__main__':
    main()