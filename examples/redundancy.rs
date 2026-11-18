use adaptive_capability_architecture::{Architecture, Edit, Fault, Node};

fn main() -> Result<(), String> {
    let mut graph = Architecture::new(12);
    graph.apply(Edit::Add(Node::new("primary", "site-a", 4, &["query"], &[])))?;
    graph.apply(Edit::Connect { from: "agent".into(), to: "primary".into() })?;

    let faults = [
        Fault::None,
        Fault::Site("site-a".into()),
        Fault::Site("site-b".into()),
    ];
    let before = graph.report("query", &faults);
    let comparison = graph.counterfactual(
        Edit::Replicate { source: "primary".into(), id: "backup".into(), site: "site-b".into() },
        "query",
        &faults,
    )?;

    println!("before: {}/{} scenarios feasible", before.satisfied_count(), before.outcomes.len());
    println!("after:  {}/{} scenarios feasible", comparison.after.satisfied_count(), comparison.after.outcomes.len());
    for (old, new) in comparison.before.outcomes.iter().zip(comparison.after.outcomes.iter()) {
        println!("{:?}: {} -> {}", old.fault, old.feasible, new.feasible);
    }
    assert_eq!(before.satisfied_count(), 2);
    assert_eq!(comparison.after.satisfied_count(), 3);
    Ok(())
}
