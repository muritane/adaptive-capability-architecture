//! Small, deterministic model of architecture-dependent capabilities.
//!
//! A declared capability is NOT an executed capability. The model checks
//! dependencies, access, failures, and budgets. Runtime handlers are needed
//! for real execution; external effects are not simulated by these labels.

use std::collections::{BTreeMap, BTreeSet};

const AGENT: &str = "agent";

/// A service with advertised capabilities and dependencies.
/// The cost is a provisioned resource cost; sites model correlated failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub site: String,
    pub cost: u32,
    pub provides: BTreeSet<String>,
    pub requires: BTreeSet<String>,
}

impl Node {
    pub fn new(id: &str, site: &str, cost: u32, provides: &[&str], requires: &[&str]) -> Self {
        Self {
            id: id.to_owned(),
            site: site.to_owned(),
            cost,
            provides: provides.iter().map(|s| (*s).to_owned()).collect(),
            requires: requires.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

/// A directed link A -> B means A can invoke capabilities provided by B.
/// The reserved source "agent" is the external caller.
#[derive(Clone, Debug)]
pub struct Architecture {
    budget: u32,
    nodes: BTreeMap<String, Node>,
    links: BTreeSet<(String, String)>,
}

#[derive(Clone, Debug)]
pub enum Edit {
    Add(Node),
    Remove(String),
    Update(Node),
    Relocate { id: String, site: String },
    Replicate { source: String, id: String, site: String },
    Connect { from: String, to: String },
    Disconnect { from: String, to: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    None,
    Node(String),
    Site(String),
}

#[derive(Clone, Debug)]
pub struct Assessment {
    pub target: String,
    pub fault: Fault,
    pub installed_cost: u64,
    pub within_budget: bool,
    pub enabled_nodes: BTreeSet<String>,
    pub accessible_capabilities: BTreeSet<String>,
    /// Node -> requirements that cannot be met through currently enabled neighbors.
    pub blocked_nodes: BTreeMap<String, BTreeSet<String>>,
    /// Feasibility within this explicit model, not a claim of successful execution.
    pub feasible: bool,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub target: String,
    pub outcomes: Vec<Assessment>,
}

impl Report {
    pub fn satisfied_count(&self) -> usize {
        self.outcomes.iter().filter(|a| a.feasible).count()
    }

    pub fn robust_for_all(&self) -> bool {
        !self.outcomes.is_empty() && self.outcomes.iter().all(|a| a.feasible)
    }
}

#[derive(Clone, Debug)]
pub struct Comparison {
    pub before: Report,
    pub after: Report,
}

/// A discrepancy is evidence against a prediction, not identification of its cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Residual {
    pub predicted: bool,
    pub observed: bool,
}

pub fn compare_observation(predicted: bool, observed: bool) -> Option<Residual> {
    (predicted != observed).then_some(Residual { predicted, observed })
}

impl Architecture {
    pub fn new(budget: u32) -> Self {
        Self {
            budget,
            nodes: BTreeMap::new(),
            links: BTreeSet::new(),
        }
    }

    pub fn budget(&self) -> u32 {
        self.budget
    }

    pub fn set_budget(&mut self, budget: u32) {
        self.budget = budget;
    }

    pub fn nodes(&self) -> &BTreeMap<String, Node> {
        &self.nodes
    }

    pub fn links(&self) -> &BTreeSet<(String, String)> {
        &self.links
    }

    /// Each edit is atomic: a failed edit does not change the architecture.
    pub fn apply(&mut self, edit: Edit) -> Result<(), String> {
        let mut next = self.clone();
        match edit {
            Edit::Add(node) => {
                validate_node(&node)?;
                if next.nodes.contains_key(&node.id) {
                    return Err(format!("node already exists: {}", node.id));
                }
                next.nodes.insert(node.id.clone(), node);
            }
            Edit::Remove(id) => {
                if next.nodes.remove(&id).is_none() {
                    return Err(format!("unknown node: {id}"));
                }
                next.links.retain(|(from, to)| from != &id && to != &id);
            }
            Edit::Update(node) => {
                validate_node(&node)?;
                if !next.nodes.contains_key(&node.id) {
                    return Err(format!("unknown node: {}", node.id));
                }
                next.nodes.insert(node.id.clone(), node);
            }
            Edit::Relocate { id, site } => {
                if site.is_empty() {
                    return Err("site cannot be empty".to_owned());
                }
                let node = next.nodes.get_mut(&id).ok_or_else(|| format!("unknown node: {id}"))?;
                node.site = site;
            }
            Edit::Replicate { source, id, site } => {
                if next.nodes.contains_key(&id) {
                    return Err(format!("node already exists: {id}"));
                }
                let mut replica = next.nodes.get(&source)
                    .ok_or_else(|| format!("unknown node: {source}"))?
                    .clone();
                replica.id = id.clone();
                replica.site = site;
                validate_node(&replica)?;
                next.nodes.insert(id.clone(), replica);
                // Preserve equivalent access and dependency edges for the replica.
                // This is a logical clone, not a data/state migration.
                let existing: Vec<_> = next.links.iter().cloned().collect();
                for (from, to) in existing {
                    if from == source || to == source {
                        let copy_from = if from == source { id.clone() } else { from };
                        let copy_to = if to == source { id.clone() } else { to };
                        next.links.insert((copy_from, copy_to));
                    }
                }
            }
            Edit::Connect { from, to } => {
                if from != AGENT && !next.nodes.contains_key(&from) {
                    return Err(format!("unknown link source: {from}"));
                }
                if !next.nodes.contains_key(&to) {
                    return Err(format!("unknown link destination: {to}"));
                }
                next.links.insert((from, to));
            }
            Edit::Disconnect { from, to } => {
                if !next.links.remove(&(from.clone(), to.clone())) {
                    return Err(format!("link does not exist: {from} -> {to}"));
                }
            }
        }
        *self = next;
        Ok(())
    }

    /// Least-fixed-point dependency evaluation prevents circular capabilities
    /// from bootstrapping themselves into availability.
    pub fn assess(&self, target: &str, fault: Fault) -> Assessment {
        let live: BTreeSet<String> = self.nodes.iter()
            .filter(|(id, node)| match &fault {
                Fault::None => true,
                Fault::Node(failed) => *id != failed,
                Fault::Site(failed) => &node.site != failed,
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut enabled = BTreeSet::new();
        loop {
            let mut changed = false;
            for id in &live {
                if enabled.contains(id) {
                    continue;
                }
                let node = &self.nodes[id];
                let ready = node.requires.iter().all(|requirement| {
                    self.links.iter().any(|(from, to)| {
                        from == id
                            && enabled.contains(to)
                            && self.nodes[to].provides.contains(requirement)
                    })
                });
                if ready {
                    enabled.insert(id.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut blocked_nodes = BTreeMap::new();
        for id in live.difference(&enabled) {
            let missing: BTreeSet<String> = self.nodes[id].requires.iter()
                .filter(|requirement| {
                    !self.links.iter().any(|(from, to)| {
                        from == id
                            && enabled.contains(to)
                            && self.nodes[to].provides.contains(*requirement)
                    })
                })
                .cloned()
                .collect();
            blocked_nodes.insert(id.clone(), missing);
        }

        let accessible_capabilities: BTreeSet<String> = self.links.iter()
            .filter(|(from, to)| from == AGENT && enabled.contains(to))
            .flat_map(|(_, to)| self.nodes[to].provides.iter().cloned())
            .collect();
        let installed_cost: u64 = self.nodes.values().map(|node| u64::from(node.cost)).sum();
        let within_budget = installed_cost <= u64::from(self.budget);
        let feasible = within_budget && accessible_capabilities.contains(target);

        Assessment {
            target: target.to_owned(),
            fault,
            installed_cost,
            within_budget,
            enabled_nodes: enabled,
            accessible_capabilities,
            blocked_nodes,
            feasible,
        }
    }

    pub fn report(&self, target: &str, faults: &[Fault]) -> Report {
        Report {
            target: target.to_owned(),
            outcomes: faults.iter().cloned().map(|fault| self.assess(target, fault)).collect(),
        }
    }

    /// Compare static end states; transition costs and migration safety are NOT modeled.
    pub fn counterfactual(
        &self,
        edit: Edit,
        target: &str,
        faults: &[Fault],
    ) -> Result<Comparison, String> {
        let before = self.report(target, faults);
        let mut candidate = self.clone();
        candidate.apply(edit)?;
        Ok(Comparison {
            before,
            after: candidate.report(target, faults),
        })
    }
}

fn validate_node(node: &Node) -> Result<(), String> {
    if node.id.is_empty() || node.id == AGENT {
        return Err("node id cannot be empty or reserved 'agent'".to_owned());
    }
    if node.site.is_empty() {
        return Err("site cannot be empty".to_owned());
    }
    if node.provides.is_empty() {
        return Err("node must advertise at least one capability".to_owned());
    }
    if node.provides.contains("") || node.requires.contains("") {
        return Err("capability names cannot be empty".to_owned());
    }
    Ok(())
}

/// Runtime handlers connect advertised capabilities to actual operations.
/// A missing handler means an advertised capability cannot be invoked here.
#[derive(Default)]
pub struct Runtime {
    handlers: BTreeMap<(String, String), Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvocationError {
    NotFeasible,
    NoHandler,
    ExecutionFailed(String),
}

impl Runtime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, node: &str, capability: &str, handler: F)
    where
        F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    {
        self.handlers.insert((node.to_owned(), capability.to_owned()), Box::new(handler));
    }

    /// Check modeled feasibility, find a reachable enabled provider, then
    /// actually call its bound handler. Handler failure is observable.
    pub fn invoke(
        &self,
        architecture: &Architecture,
        capability: &str,
        input: &str,
        fault: Fault,
    ) -> Result<String, InvocationError> {
        let assessment = architecture.assess(capability, fault);
        if !assessment.feasible {
            return Err(InvocationError::NotFeasible);
        }
        for (from, to) in architecture.links() {
            if from != AGENT || !assessment.enabled_nodes.contains(to) {
                continue;
            }
            if !architecture.nodes[to].provides.contains(capability) {
                continue;
            }
            if let Some(handler) = self.handlers.get(&(to.clone(), capability.to_owned())) {
                return handler(input).map_err(InvocationError::ExecutionFailed);
            }
        }
        Err(InvocationError::NoHandler)
    }
}
