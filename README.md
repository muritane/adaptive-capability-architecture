# Adaptive Capability Architecture

**Experimental Rust library for architecture-dependent capability, reconfiguration, and counterfactual evaluation.**

Research question: **How can a bounded agent discover, evaluate, and modify the architecture that determines its present and future capabilities?**

This repository turns selected claims from [capability_systems](https://github.com/muritane/capability_systems) and [constraint_architecture](https://github.com/muritane/constraint_architecture) into small, falsifiable computational models. It is an experiment, **not** a faithful simulator of real-world infrastructure.

## Start

Requires Rust 1.74+; no third-party crates.

```sh
cargo test
cargo run --example redundancy
cargo run --example execution
cargo doc --open
```

## The API, and what it demonstrates

| API | Question it makes inspectable |
| --- | --- |
| `Architecture::apply(Edit::…)` | Which architectural changes are well-formed? |
| `Architecture::assess` | Does an advertised capability have a connected, dependency-satisfied path from the agent? |
| `Architecture::report` | Does the capability remain feasible under particular node/site failures? |
| `Architecture::counterfactual` | What would change if an edit were applied to the *model*? |
| `Runtime::register` and `Runtime::invoke` | Is a reachable capability backed by an actually callable implementation? |
| `compare_observation` | Did a prediction disagree with an observation? |

A minimal example:

```rust
use adaptive_capability_architecture::{Architecture, Edit, Fault, Node};

let mut world = Architecture::new(10);
world.apply(Edit::Add(Node::new("compute", "west", 4, &["compute"], &[])))?;
world.apply(Edit::Connect { from: "agent".into(), to: "compute".into() })?;
assert!(world.assess("compute", Fault::None).feasible);
world.apply(Edit::Remove("compute".into()))?;
assert!(!world.assess("compute", Fault::None).feasible);
# Ok::<(), String>(())
```

See [redundancy](examples/redundancy.rs) for a one-node removal test versus site failures. See [execution](examples/execution.rs) for the distinction between declared feasibility and an executable handler.

## Explicit model boundaries

- Edges are directed. `A -> B` means A can invoke services of B. The special source `agent` represents the caller.
- A node activates when **all** its required capability names are supplied by its *already-active outgoing neighbors*. Activation uses the **least fixed point**, so an unsupported dependency cycle does not become viable just because it is cyclic.
- The agent sees only capabilities provided by active nodes connected directly from `agent`. This is an intentionally simple one-hop interface; dependencies can be multi-level.
- Budgets are installation-cost ceilings, not throughput, scheduling, migration-cost, or monetary optimization models.
- Failures disable one node or one **site** in a given assessment. They do not model stochastic failure probabilities.
- Node replication copies advertised capability, cost, requirements, and logical connections, **not persistent state**, data, credentials, or physical infrastructure.
- A feasible assessment is a **structural prediction**, not an execution guarantee. Only `Runtime::invoke` calls registered handlers; even those may fail.
- Runtime handlers are in-process function adapters, not network RPC, GitHub API connections, or a security boundary. No real external service credentials are used.
- Counterfactuals compare two static models. They do **not** certify a safe transition between them.
- A residual flags prediction/observation disagreement; it does not automatically discover the missing variable.

## Why this is not another universal framework

A network that works with a node has not thereby proved the node necessary. A network that survives its removal has not proved it was useless: it may have provided rare-event recoverability. These claims should be tested under explicit scenarios rather than inferred from a working configuration.

The next steps are to add: (1) migration trajectories and switching costs, (2) unknown dependencies and observation-driven model revision, (3) allocation/capacity constraints, and (4) comparison with established self-adaptive systems, graph reliability, planning, and control techniques.

## Related work

- [Reality-Grounded Agency](https://github.com/muritane/reality-grounded-agency): synthesis and research map; this repository supplies a narrow experimental component.
- [Capability Systems](https://github.com/muritane/capability_systems): feasible composition and locally executable guidance.
- [Constraint Architecture](https://github.com/muritane/constraint_architecture): finite resources, irreversibility, loss, and execution typing.
- [Recoverability Principle](https://github.com/muritane/recoverability-principle): viability margins and disturbance response.

See [foundations](docs/foundations.md) and [experiment protocol](docs/experiment-protocol.md). Research claims are tentative; source and implementation should be inspected independently.
