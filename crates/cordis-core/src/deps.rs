//! Derived acceleration for exact Service dependency fan-out.
//!
//! Fiber-owned era-local [`DependencyEdge`](crate::fiber::DependencyEdge) rows are
//! authoritative. This index stores only the reverse projection needed to find
//! likely dependents cheaply. If the projection is unavailable or known
//! incomplete, callers fall back to scanning resident Fibers; index order,
//! allocation, and history never participate in diagnostics or target equality.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::context::RealmKey;
use crate::fiber::{Fiber, FiberId};

/// Optional reverse projection beside the authoritative Service store and
/// Fiber-owned exact edges. A missing/incomplete projection changes cost only.
pub(crate) struct DependencyIndex {
    state: Mutex<IndexState>,
}

struct IndexState {
    /// resolved edge → the fibers declaring it. Ordering is nonsemantic.
    reverse: HashMap<(String, RealmKey), Vec<Dependent>>,
    /// Whether this projection is known complete. False means callers must scan authority.
    complete: bool,
}

struct Dependent {
    id: FiberId,
    fiber: Weak<Fiber>,
}

impl Default for DependencyIndex {
    fn default() -> Self {
        Self {
            state: Mutex::new(IndexState {
                reverse: HashMap::new(),
                complete: true,
            }),
        }
    }
}

impl DependencyIndex {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Project a freshly spawned Fiber's authoritative exact edges into the
    /// reverse acceleration map. Called only from the spawn path.
    ///
    /// A fiber that is already dead here (typed group removal that won
    /// the attach-vs-dispose race between registry membership and this
    /// line) is skipped: its edges have no observer left to serve.
    pub(crate) fn register(&self, fiber: &Arc<Fiber>) {
        let id = fiber.id().clone();
        let mut state = self.state.lock();
        if !state.complete {
            return;
        }
        // This check belongs inside the same critical section as insertion.
        // Disposal writes the death mark before taking this lock to unregister:
        // if registration owns the lock first, unregister follows and removes
        // its rows; if disposal owns/completes the ordering first, this check
        // refuses the late insertion. There is no unregister-before-register
        // third outcome that can resurrect a dead Fiber's derived reverse rows.
        if !fiber.is_alive() {
            return;
        }
        for edge in fiber.dependency_edges() {
            state
                .reverse
                .entry((edge.service.clone(), edge.realm))
                .or_default()
                .push(Dependent {
                    id: id.clone(),
                    fiber: Arc::downgrade(fiber),
                });
        }
    }

    /// Remove a Fiber from the derived reverse projection beside its death mark.
    /// Idempotent: an unknown ID
    /// (never registered,
    /// or a race-lost register after dispose) is a no-op.
    pub(crate) fn unregister(&self, fiber: &Fiber) {
        let id = fiber.id();
        let mut state = self.state.lock();
        for edge in fiber.dependency_edges() {
            let key = (edge.service.clone(), edge.realm);
            if let Some(list) = state.reverse.get_mut(&key) {
                list.retain(|d| d.id != *id);
                if list.is_empty() {
                    state.reverse.remove(&key);
                }
            }
        }
    }

    /// The unique dependents with upgradeable handles for any requested
    /// service edge (ADR 0024).
    ///
    /// The implementation returns a stable key-major order for determinism,
    /// but that order is acceleration detail and has no semantic meaning.
    /// Duplicate declarations, duplicate requested edges, and fibers matching
    /// several edges therefore produce one result. All edge lists are read and
    /// dead `Weak`s pruned under one lock, so the union is one atomic index
    /// snapshot rather than a caller-composed sequence of observations.
    pub(crate) fn dependents_of(&self, edges: &[(String, RealmKey)]) -> Option<Vec<Arc<Fiber>>> {
        // Every upgraded Arc leaves the critical section, including
        // duplicates. Dedup happens afterwards so dropping a redundant Arc
        // can never run Fiber (and therefore plugin/config) Drop under the
        // index lock (ADR 0010).
        let mut upgraded = Vec::new();
        {
            let mut state = self.state.lock();
            if !state.complete {
                return None;
            }
            let mut emptied = Vec::new();
            for edge in edges {
                let Some(list) = state.reverse.get_mut(edge) else {
                    continue;
                };
                let mut retained = Vec::with_capacity(list.len());
                for dependent in std::mem::take(list) {
                    if let Some(fiber) = dependent.fiber.upgrade() {
                        upgraded.push((dependent.id.clone(), fiber));
                        retained.push(dependent);
                    }
                    // A rejected `dependent` contains only FiberId + Weak; its
                    // in-section drop cannot execute user code.
                }
                *list = retained;
                if list.is_empty() {
                    emptied.push(edge.clone());
                }
            }
            for edge in emptied {
                // The dropped list contains no entries. Removing the map row
                // cannot execute user code inside the section.
                state.reverse.remove(&edge);
            }
        }

        let mut seen = HashSet::with_capacity(upgraded.len());
        let mut unique = Vec::with_capacity(upgraded.len());
        for (id, fiber) in upgraded {
            if seen.insert(id) {
                unique.push(fiber);
            }
        }
        Some(unique)
    }

    #[cfg(test)]
    pub(crate) fn disable_for_test(&self) {
        let mut state = self.state.lock();
        state.reverse.clear();
        state.complete = false;
    }

    #[cfg(test)]
    pub(crate) fn rebuild_for_test(&self, fibers: &[Arc<Fiber>]) {
        let mut reverse: HashMap<(String, RealmKey), Vec<Dependent>> = HashMap::new();
        for fiber in fibers.iter().filter(|fiber| fiber.is_alive()) {
            for edge in fiber.dependency_edges() {
                reverse
                    .entry((edge.service.clone(), edge.realm))
                    .or_default()
                    .push(Dependent {
                        id: fiber.id().clone(),
                        fiber: Arc::downgrade(fiber),
                    });
            }
        }
        let mut state = self.state.lock();
        state.reverse = reverse;
        state.complete = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Context, RealmKey};
    use crate::fiber::{DependencyEdge, Fiber};

    fn fiber(name: &str, edges: &[(&str, RealmKey)]) -> Arc<Fiber> {
        Fiber::new_with_edges(
            name,
            edges
                .iter()
                .map(|(service, realm)| DependencyEdge::new((*service).to_owned(), *realm))
                .collect(),
        )
    }

    #[test]
    fn reverse_projection_matches_fiber_owned_edges_and_unregisters() {
        let index = DependencyIndex::new();
        let a = fiber("a", &[("svc", RealmKey::DEFAULT)]);
        let b = fiber("b", &[("other", RealmKey::DEFAULT)]);
        index.register(&a);
        index.register(&b);
        let edge = [("svc".to_owned(), RealmKey::DEFAULT)];
        assert_eq!(index.dependents_of(&edge).unwrap().len(), 1);
        index.unregister(&a);
        assert!(index.dependents_of(&edge).unwrap().is_empty());
        assert_eq!(
            a.dependency_edges()[0].service,
            "svc",
            "unregister never erases Fiber authority"
        );
    }

    #[test]
    fn clearing_and_rebuilding_projection_changes_no_authoritative_edges() {
        let ctx = Context::new();
        let a = fiber("a", &[("svc", RealmKey::DEFAULT)]);
        ctx.root.deps.register(&a);
        let edge = [("svc".to_owned(), RealmKey::DEFAULT)];
        assert_eq!(ctx.root.deps.dependents_of(&edge).unwrap().len(), 1);
        ctx.root.deps.disable_for_test();
        assert!(ctx.root.deps.dependents_of(&edge).is_none());
        assert_eq!(a.dependency_edges()[0].service, "svc");
        ctx.root.deps.rebuild_for_test(std::slice::from_ref(&a));
        assert_eq!(ctx.root.deps.dependents_of(&edge).unwrap().len(), 1);
    }

    #[test]
    fn dependent_set_deduplicates_without_defining_semantic_order() {
        let index = DependencyIndex::new();
        let overlap = fiber(
            "overlap",
            &[("alpha", RealmKey::DEFAULT), ("beta", RealmKey::DEFAULT)],
        );
        index.register(&overlap);
        let requested = [
            ("beta".to_owned(), RealmKey::DEFAULT),
            ("alpha".to_owned(), RealmKey::DEFAULT),
            ("beta".to_owned(), RealmKey::DEFAULT),
        ];
        let found = index.dependents_of(&requested).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id(), overlap.id());
    }

    #[tokio::test]
    async fn late_registration_cannot_resurrect_a_dead_fiber() {
        let index = DependencyIndex::new();
        let fiber = fiber("dead", &[("svc", RealmKey::DEFAULT)]);
        fiber.dispose().await;
        index.register(&fiber);
        assert!(
            index
                .dependents_of(&[("svc".to_owned(), RealmKey::DEFAULT)])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn dead_reverse_entries_are_pruned_lazily() {
        let index = DependencyIndex::new();
        let fiber = fiber("gone", &[("svc", RealmKey::DEFAULT)]);
        index.register(&fiber);
        drop(fiber);
        assert!(
            index
                .dependents_of(&[("svc".to_owned(), RealmKey::DEFAULT)])
                .unwrap()
                .is_empty()
        );
        assert!(index.state.lock().reverse.is_empty());
    }
}
