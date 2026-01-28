GUROBI SET UP


1) Get a free Gurobi Named-User License (Register with university email while on eduroam wifi or with TU Berlin VPN)
https://www.gurobi.com/academia/academic-program-and-licenses/

2) Run the license activation (This will download a gurobi.lic file to your home directory)
> *grbgetkey YOUR-LICENSE-KEY*

3) Find where Gurobi is stored on your computer:
> *which gurobi_cl | xargs ls -l*

4) Save these lines in shell profile (modify if your path is different from mine in step 1) :

> *nano ~/.zshrc*

Paste below lines:
> export GUROBI_HOME="/Library/gurobi1203/macos_universal2" \
> export GUROBI_LIBNAME="gurobi120" \
> export PATH="${PATH}:${GUROBI_HOME}/bin" \
> export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:${GUROBI_HOME}/lib" 

> *source ~/.zshrc*

5) Verify variables are set

>*echo $GUROBI_HOME*

> *echo $GUROBI_LIBNAME*

6) Run program 

> *cargo clean*

> *cargo build*

> *cargo run* \
(may have to run from /minimum_diversity_problem folder so program can capture input files)

7) May have to run these commands to implement the Python / visualization section

> *cargo add serde --features derive*

> *cargo add serde_json*

> *cargo add chrono*

---------------------------------------------------------------------

How it works:

1. Run your Rust program once - it saves results to results_YYYYMMDD_HHMMSS.json
2. Python script auto-generates - creates visualize_results.py
3. Generate all plots - run python visualize_results.py results_YYYYMMDD_HHMMSS.json

Visualizations:
-> Scatter plot: Quality vs Time
-> Heatmap: Performance by category
-> Pareto frontier: Tradeoffs
-> Box plots: Distribution analysis
-> Scaling: How solvers grow with problem size
-> Win rate: Which solver wins most

For presentation we can:
1. Load old results without re-running: python visualize_results.py old_results.json
2. Compare multiple runs side-by-side
3. JSON is human-readable for quick grep/inspection
