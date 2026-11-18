# Experiment protocol: is an apparently redundant node useful?

## Hypotheses

H1: A redundant provider may not change no-fault feasibility but can improve single-site-fault feasibility.

H2: Adding a replica can make the budget infeasible, so increased theoretical redundancy need not improve available capability.

H3: A prediction/observation mismatch identifies a model limitation but does not by itself determine whether the cause is topology, capacity, permissions, or a broken implementation.

## Baselines and measurements

- Baseline: single provider.
- Intervention: add provider replica on a distinct site; compare to replica on the same site.
- Perturbations: no fault, failure of first site, failure of second site.
- Measure: per-scenario feasibility, fraction satisfied, installed cost, budget compliance, and real handler success where tested.
- Report both no-fault results and disturbance robustness, not a single scalar rank.

## Threats to validity

Faults are stipulated scenarios, not empirical probabilities. The graph assumes known dependencies; a real system may have hidden edges. Replicas here do not replicate persistent state. No migration cost, latency, authority, or dynamic workload is represented. The simulated outcome should not be claimed as evidence for real-world superiority.

## Development plan

1. Explicit transition planning: add maintenance windows, action costs, temporary loss of service.
2. Model revision: observe systematic residuals and split incorrectly merged capability classes.
3. Introduce load/capacity and demand; show when a structurally available path is operationally inadequate.
4. Run controlled comparisons with fixed, reactive-rule, and look-ahead architectures.
