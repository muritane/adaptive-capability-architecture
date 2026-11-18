# Foundations and distinctions

## Three different propositions

1. **Advertisement**: node N lists capability C.
2. **Structural feasibility**: the agent can reach an active provider of C, all recursively required capabilities resolve, and the installation budget holds.
3. **Realization**: an actual executable handler is bound, invoked, and returns an observable result.

(1) does not imply (2). (2) does not imply (3). Even (3) need not imply that the resulting world state satisfied an external task.

## Relative feasibility

For architecture G, target c, fault scenario f, and budget B, let the modeled feasibility predicate be:

```text
F(G, c, f, B) =
  installed_cost(G) <= B
  AND exists enabled provider p:
      ("agent", p) is an edge
      AND c is advertised by p
```

Enabled nodes are the **least fixed point** of the rule: a live node is enabled if all its required capabilities are supplied by currently enabled providers along its outgoing edges. This is a conservative bootstrapping assumption. It does not model feedback systems that can establish themselves through staged initialization.

## Edit operators

Add / remove / update / relocate / replicate nodes; connect / disconnect directed dependencies. Each edit is validated atomically. The API does not presume edits are beneficial.

## Counterfactuals

Compare F(G,c,…) to F(T(G,e),c,…) for a hypothetical edit e. Both are predictions from the same simplified model. If observed behavior differs, `compare_observation` records a residual but cannot locate the causal error. A richer approach must differentiate model uncertainty, policy, intervention, observation, and model structure changes.

## Important omissions

No latency, queueing, resource contention, real network transport, live API discovery, stochastic simulation, state transfer, authority management, or physical migration costs. These omissions are **limitations**, not properties of real agents.

## Existing work to engage with

Self-adaptive software / MAPE-K, graph optimization and network reliability, fault tolerance and distributed consensus, hierarchical planning, viable control under uncertainty, Bayesian experimental design, digital twins and causal identification. Specific originality claims should be made only after comparison with those literatures.
