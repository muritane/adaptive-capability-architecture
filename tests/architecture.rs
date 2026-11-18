use adaptive_capability_architecture::{
    compare_observation, Architecture, Edit, Fault, InvocationError, Node, Runtime,
};

fn add(graph: &mut Architecture, node: Node) {
    graph.apply(Edit::Add(node)).unwrap();
}
fn connect(graph: &mut Architecture, from: &str, to: &str) {
    graph.apply(Edit::Connect { from: from.into(), to: to.into() }).unwrap();
}

#[test]
fn advertisement_without_agent_access_is_not_feasibility() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("a", "s", 2, &["read"], &[]));
    assert!(!g.assess("read", Fault::None).feasible);
    connect(&mut g, "agent", "a");
    assert!(g.assess("read", Fault::None).feasible);
}

#[test]
fn dependencies_need_connected_providers() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("api", "s1", 2, &["answer"], &["data"]));
    add(&mut g, Node::new("db", "s2", 2, &["data"], &[]));
    connect(&mut g, "agent", "api");
    assert!(!g.assess("answer", Fault::None).feasible);
    connect(&mut g, "api", "db");
    assert!(g.assess("answer", Fault::None).feasible);
}

#[test]
fn unsupported_cycle_does_not_bootstrap() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("a", "s1", 1, &["a"], &["b"]));
    add(&mut g, Node::new("b", "s2", 1, &["b"], &["a"]));
    connect(&mut g, "agent", "a");
    connect(&mut g, "a", "b");
    connect(&mut g, "b", "a");
    let outcome = g.assess("a", Fault::None);
    assert!(!outcome.feasible);
    assert_eq!(outcome.blocked_nodes["a"].len(), 1);
}

#[test]
fn replica_improves_cross_site_fault_feasibility() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("primary", "west", 3, &["query"], &[]));
    connect(&mut g, "agent", "primary");
    let faults = [Fault::None, Fault::Site("west".into()), Fault::Site("east".into())];
    let comparison = g.counterfactual(
        Edit::Replicate { source: "primary".into(), id: "backup".into(), site: "east".into() },
        "query", &faults,
    ).unwrap();
    assert_eq!(comparison.before.satisfied_count(), 2);
    assert_eq!(comparison.after.satisfied_count(), 3);
    assert_eq!(g.nodes().len(), 1); // A counterfactual must not mutate its source.
}

#[test]
fn budget_blocks_even_redundant_configuration() {
    let mut g = Architecture::new(5);
    add(&mut g, Node::new("primary", "west", 3, &["query"], &[]));
    connect(&mut g, "agent", "primary");
    g.apply(Edit::Replicate { source: "primary".into(), id: "backup".into(), site: "east".into() }).unwrap();
    let outcome = g.assess("query", Fault::None);
    assert_eq!(outcome.installed_cost, 6);
    assert!(!outcome.within_budget);
    assert!(!outcome.feasible);
}

#[test]
fn edits_are_atomic_and_removal_cleans_edges() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("primary", "west", 3, &["query"], &[]));
    connect(&mut g, "agent", "primary");
    assert!(g.apply(Edit::Connect { from: "missing".into(), to: "primary".into() }).is_err());
    assert_eq!(g.links().len(), 1);
    g.apply(Edit::Remove("primary".into())).unwrap();
    assert!(g.links().is_empty());
}

#[test]
fn declared_capability_needs_handler_to_execute() {
    let mut g = Architecture::new(10);
    add(&mut g, Node::new("worker", "west", 3, &["echo"], &[]));
    connect(&mut g, "agent", "worker");
    let mut runtime = Runtime::new();
    assert_eq!(runtime.invoke(&g, "echo", "test", Fault::None), Err(InvocationError::NoHandler));
    runtime.register("worker", "echo", |s| Ok(format!("echo:{s}")));
    assert_eq!(runtime.invoke(&g, "echo", "test", Fault::None), Ok("echo:test".to_owned()));
    g.apply(Edit::Disconnect { from: "agent".into(), to: "worker".into() }).unwrap();
    assert_eq!(runtime.invoke(&g, "echo", "test", Fault::None), Err(InvocationError::NotFeasible));
}

#[test]
fn residual_is_only_a_disagreement_signal() {
    assert_eq!(compare_observation(true, true), None);
    assert_eq!(compare_observation(true, false).unwrap().observed, false);
}
