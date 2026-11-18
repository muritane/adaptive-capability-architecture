use adaptive_capability_architecture::{
    compare_observation, Architecture, Edit, Fault, InvocationError, Node, Runtime,
};

fn main() -> Result<(), String> {
    let mut architecture = Architecture::new(5);
    architecture.apply(Edit::Add(Node::new("worker", "site-1", 2, &["uppercase"], &[])))?;
    architecture.apply(Edit::Connect { from: "agent".into(), to: "worker".into() })?;
    assert!(architecture.assess("uppercase", Fault::None).feasible);

    let mut runtime = Runtime::new();
    assert_eq!(
        runtime.invoke(&architecture, "uppercase", "hello", Fault::None),
        Err(InvocationError::NoHandler),
    );
    // Real behavior becomes available only after binding an implementation.
    runtime.register("worker", "uppercase", |input| Ok(input.to_uppercase()));
    let value = runtime.invoke(&architecture, "uppercase", "hello", Fault::None)
        .map_err(|err| format!("{err:?}"))?;
    println!("executed uppercase: {value}");
    assert_eq!(value, "HELLO");

    // The observation disagrees with the claim "no execution would succeed".
    assert!(compare_observation(false, true).is_some());
    Ok(())
}
