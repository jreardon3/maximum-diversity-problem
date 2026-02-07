// src/adj_solver_breakpoint.rs

use crate::parser::MdpData;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

#[derive(Clone)]
pub struct BreakpointConfig {
    pub max_lambda_iterations: usize,
    // Tolerance for floating point comparison in binary search
    pub epsilon: f64,
}

impl Default for BreakpointConfig {
    fn default() -> Self {
        Self {
            max_lambda_iterations: 60,
            epsilon: 1e-5,
        }
    }
}

/// The main entry point for the Breakpoint Solver
/// Implements the "Compact Formulation" (Lemma 3.1/3.2) and the BP Algorithm.
pub fn solve_breakpoint(
    data: &MdpData,
    config: &BreakpointConfig,
    deadline: Instant,
) -> (Vec<usize>, f64) {
    let n = data.n;
    let k_target = data.k;

    // 1. Pre-calculate weighted degrees (d_i) for Lemma 3.1.
    // d_i = sum of weights of edges incident to node i.
    // This allows us to use the O(n) node graph instead of O(m) nodes.
    // "We denote by d_i the weighted degree of node i"
    let mut degrees = vec![0.0; n];
    let mut max_degree = 0.0;

    // Optimization: Matrix is symmetric, but iterating row-wise is cache-friendly.
    for i in 0..n {
        let mut sum_w = 0.0;
        for j in 0..n {
            if i != j {
                sum_w += data.distances[i * n + j];
            }
        }
        degrees[i] = sum_w;
        if sum_w > max_degree {
            max_degree = sum_w;
        }
    }

    // 2. Search for the relevant breakpoints (Lambda search).
    // We are looking for the "envelope" breakpoints surrounding budget k.
    // Lambda represents the penalty for selecting a node.
    // Range: 0.0 (selects many nodes) to max_degree/2 (selects few/no nodes).
    // Note: Lemma 3.2 formulation maximizes (d_i - 2*lambda).

    let mut low_lambda = 0.0;
    let mut high_lambda = max_degree / 2.0 + 1.0;

    // We store the best "under" (<= k) and "over" (>= k) solutions found.
    // Default s_under is empty, s_over is all nodes (safe fallback).
    let mut s_under: Vec<usize> = Vec::new();
    let mut s_over: Vec<usize> = (0..n).collect();

    let mut iter = 0;
    while iter < config.max_lambda_iterations {
        if Instant::now() >= deadline {
            break;
        }
        iter += 1;

        let mid_lambda = (low_lambda + high_lambda) / 2.0;

        // Solve using the Compact s-excess Formulation (Lemma 3.2)
        // This is significantly faster than the selection graph.
        let solution = solve_compact_sexcess(data, &degrees, mid_lambda);
        let size = solution.len();

        if size == k_target {
            // Exact match found (Optimal breakpoint)
            let div = calculate_diversity(data, &solution);
            return (solution, div);
        } else if size < k_target {
            // Too few nodes -> Penalty (lambda) is too high
            // This solution becomes our new "lower bound" set
            s_under = solution;
            high_lambda = mid_lambda;
        } else {
            // Too many nodes -> Penalty (lambda) is too low
            // This solution becomes our new "upper bound" set
            s_over = solution;
            low_lambda = mid_lambda;
        }

        if (high_lambda - low_lambda).abs() < config.epsilon {
            break;
        }
    }

    // 3. Post-Processing: Greedy Adjustment (The BP Algorithm)
    // "identifies the nearest adjacent breakpoints and applies a greedy heuristic"
    // The "Nestedness Property" ensures s_under is a subset of s_over.

    // Strategy A: Fill S_under up to k (Greedy Add)
    // CRITICAL FIX: We only scan candidates present in S_over.
    // "For each node i in S_{l+1} \ S we calculate the increment..."
    let mut final_a = s_under.clone();
    fill_greedy(data, &mut final_a, k_target, &s_over);
    let div_a = calculate_diversity(data, &final_a);

    // Strategy B: Prune S_over down to k (Greedy Remove)
    let mut final_b = s_over.clone();
    prune_greedy(data, &mut final_b, k_target);
    let div_b = calculate_diversity(data, &final_b);

    // Return the better of the two
    if div_a > div_b {
        (final_a, div_a)
    } else {
        (final_b, div_b)
    }
}

// --- Compact Formulation Solver (Lemma 3.2) ---

/// Constructs and solves the s-excess graph.
/// Nodes: Source(s), Sink(t), and nodes 0..n-1.
/// Size: n + 2 nodes. (Compared to n^2 in the selection formulation).
/// "compact formulation... graph with n+2 nodes and O(m) arcs"
fn solve_compact_sexcess(data: &MdpData, degrees: &[f64], lambda: f64) -> Vec<usize> {
    let n = data.n;

    // Source = n, Sink = n + 1
    let source = n;
    let sink = n + 1;
    let mut graph = FlowGraph::new(n + 2);

    // 1. Edges from Source and to Sink based on weights w_i
    // w_i = d_i - 2*lambda
    // If w_i > 0: Edge s -> i with capacity w_i
    // If w_i < 0: Edge i -> t with capacity -w_i
    for i in 0..n {
        let weight = degrees[i] - 2.0 * lambda;
        if weight > 0.0 {
            graph.add_edge(source, i, weight);
        } else {
            graph.add_edge(i, sink, -weight);
        }
    }

    // 2. Pairwise edges
    // "The set of arcs A consists of a pair of opposing directed arcs... for each edge [i,j]"
    // Capacity is u_ij
    for i in 0..n {
        for j in (i + 1)..n {
            let w = data.distances[i * n + j];
            if w > 1e-9 {
                // Add i -> j and j -> i with capacity w
                graph.add_edge(i, j, w);
                graph.add_edge(j, i, w);
            }
        }
    }

    // 3. Solve Min-Cut
    graph.dinic(source, sink);

    // 4. Extract Solution
    // "Source set of a minimum cut ... is also a maximum s-excess set"
    let reachable = graph.get_reachable(source);
    let mut selected = Vec::new();
    for i in 0..n {
        if reachable.contains(&i) {
            selected.push(i);
        }
    }
    selected
}

// --- Greedy Heuristics ---

/// Greedily fills the solution set until it reaches target_k.
/// STRICT: Only considers candidates found in `upper_bound_set` (s_over).
fn fill_greedy(
    data: &MdpData,
    current_sol: &mut Vec<usize>,
    target_k: usize,
    upper_bound_set: &[usize],
) {
    let n = data.n;
    // Keep track of who is in the set for O(1) lookup
    let mut in_set = vec![false; n];
    for &x in current_sol.iter() {
        in_set[x] = true;
    }

    // If for some reason s_over is smaller than k (edge case), fallback to all nodes.
    // But logically s_over should be >= k based on the binary search.
    let candidates = if upper_bound_set.len() > current_sol.len() {
        upper_bound_set
    } else {
        // Fallback or empty candidates - although this path shouldn't trigger
        // if binary search worked correctly.
        return;
    };

    while current_sol.len() < target_k {
        let mut best_node = None;
        let mut best_gain = -f64::INFINITY;

        // Optimized Search:
        // "For each node i in S_{l+1} \ S..."
        // We iterate ONLY over the candidates provided by the upper breakpoint.
        for &i in candidates {
            if !in_set[i] {
                let mut gain = 0.0;
                for &existing in current_sol.iter() {
                    gain += data.distances[i * n + existing];
                }

                if best_node.is_none() || gain > best_gain {
                    best_gain = gain;
                    best_node = Some(i);
                }
            }
        }

        if let Some(node) = best_node {
            current_sol.push(node);
            in_set[node] = true;
        } else {
            // No valid candidates left to reach k
            break;
        }
    }
}

fn prune_greedy(data: &MdpData, current_sol: &mut Vec<usize>, target_k: usize) {
    let n = data.n;
    // "remove nodes, one at a time, that minimize the loss of utility"
    // We start from s_over and prune down.

    while current_sol.len() > target_k {
        let mut worst_idx = None;
        let mut lowest_contribution = f64::MAX;

        // O(k^2) check per removal.
        for (idx, &node_i) in current_sol.iter().enumerate() {
            let mut contribution = 0.0;
            for (j_idx, &node_j) in current_sol.iter().enumerate() {
                if idx != j_idx {
                    contribution += data.distances[node_i * n + node_j];
                }
            }

            if contribution < lowest_contribution {
                lowest_contribution = contribution;
                worst_idx = Some(idx);
            }
        }

        if let Some(idx) = worst_idx {
            current_sol.swap_remove(idx);
        } else {
            break;
        }
    }
}

fn calculate_diversity(data: &MdpData, solution: &[usize]) -> f64 {
    let mut div = 0.0;
    let n = data.n;
    for i in 0..solution.len() {
        for j in (i + 1)..solution.len() {
            let u = solution[i];
            let v = solution[j];
            div += data.distances[u * n + v];
        }
    }
    div
}

// --- Max-Flow (Dinic) Implementation ---

#[derive(Clone)]
struct Edge {
    to: usize,
    capacity: f64,
    flow: f64,
    rev: usize, // index of the reverse edge in adj[to]
}

struct FlowGraph {
    adj: Vec<Vec<Edge>>,
    level: Vec<i32>,
    ptr: Vec<usize>,
}

impl FlowGraph {
    fn new(nodes: usize) -> Self {
        FlowGraph {
            adj: vec![Vec::new(); nodes],
            level: Vec::new(),
            ptr: Vec::new(),
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, cap: f64) {
        let a_len = self.adj[from].len();
        let b_len = self.adj[to].len();

        self.adj[from].push(Edge {
            to,
            capacity: cap,
            flow: 0.0,
            rev: b_len,
        });

        self.adj[to].push(Edge {
            to: from,
            capacity: 0.0, // Residual only for directed edges
            flow: 0.0,
            rev: a_len,
        });
    }

    fn bfs(&mut self, s: usize, t: usize) -> bool {
        self.level = vec![-1; self.adj.len()];
        self.level[s] = 0;
        let mut q = VecDeque::new();
        q.push_back(s);

        while let Some(u) = q.pop_front() {
            for edge in &self.adj[u] {
                if edge.capacity - edge.flow > 1e-9 && self.level[edge.to] == -1 {
                    self.level[edge.to] = self.level[u] + 1;
                    q.push_back(edge.to);
                }
            }
        }
        self.level[t] != -1
    }

    fn dfs(&mut self, u: usize, t: usize, pushed: f64) -> f64 {
        if pushed < 1e-9 || u == t {
            return pushed;
        }

        for i in self.ptr[u]..self.adj[u].len() {
            self.ptr[u] = i;
            // Access edge fields, avoiding borrowing self.adj twice
            let edge_to = self.adj[u][i].to;
            let edge_cap = self.adj[u][i].capacity;
            let edge_flow = self.adj[u][i].flow;

            if self.level[u] + 1 != self.level[edge_to] || edge_cap - edge_flow < 1e-9 {
                continue;
            }

            let tr = self.dfs(edge_to, t, pushed.min(edge_cap - edge_flow));
            if tr == 0.0 {
                continue;
            }

            self.adj[u][i].flow += tr;
            let rev_idx = self.adj[u][i].rev;
            self.adj[edge_to][rev_idx].flow -= tr;

            return tr;
        }
        0.0
    }

    fn dinic(&mut self, s: usize, t: usize) -> f64 {
        let mut flow = 0.0;
        while self.bfs(s, t) {
            self.ptr = vec![0; self.adj.len()];
            loop {
                let pushed = self.dfs(s, t, f64::INFINITY);
                if pushed < 1e-9 {
                    break;
                }
                flow += pushed;
            }
        }
        flow
    }

    fn get_reachable(&self, s: usize) -> HashSet<usize> {
        let mut visited = HashSet::new();
        let mut q = VecDeque::new();
        q.push_back(s);
        visited.insert(s);

        while let Some(u) = q.pop_front() {
            for edge in &self.adj[u] {
                if edge.capacity - edge.flow > 1e-9 && !visited.contains(&edge.to) {
                    visited.insert(edge.to);
                    q.push_back(edge.to);
                }
            }
        }
        visited
    }
}