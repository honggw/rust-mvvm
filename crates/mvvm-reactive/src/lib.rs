//! Reactive primitives used by the MVVM layer.
//!
//! This crate deliberately exposes a small, owned-handle API over Leptos'
//! `reactive_graph`. UI adapters should create and dispose owners on the same
//! thread where their reactive effects are run; an `Owner`'s current-context
//! is thread-local and must not be treated as a cross-thread capability.

use reactive_graph::{
    computed::ArcMemo,
    owner::Owner as ReactiveOwner,
    prelude::{Get, Set, Update},
    signal::{ArcReadSignal, ArcRwSignal},
};

/// An owned, cloneable reactive state cell.
#[derive(Clone, Debug)]
pub struct State<T>(ArcRwSignal<T>);

impl<T> State<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(value: T) -> Self {
        Self(ArcRwSignal::new(value))
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.0.get()
    }

    pub fn set(&self, value: T) {
        self.0.set(value);
    }

    pub fn update(&self, update: impl FnOnce(&mut T)) {
        self.0.update(update);
    }

    pub fn read_only(&self) -> ReadOnlyState<T> {
        ReadOnlyState(self.0.read_only())
    }

    /// Access the underlying signal for advanced integrations.
    pub fn signal(&self) -> &ArcRwSignal<T> {
        &self.0
    }
}

/// A read-only view of a state cell. It intentionally has no setter.
#[derive(Clone, Debug)]
pub struct ReadOnlyState<T>(ArcReadSignal<T>);

impl<T> ReadOnlyState<T>
where
    T: Send + Sync + 'static,
{
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.0.get()
    }

    pub fn signal(&self) -> &ArcReadSignal<T> {
        &self.0
    }
}

/// A lazily evaluated, memoized derived value.
#[derive(Clone, Debug)]
pub struct Memo<T>(ArcMemo<T>);

impl<T> Memo<T>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    pub fn new(compute: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Self {
        Self(ArcMemo::new(compute))
    }

    pub fn get(&self) -> T {
        self.0.get()
    }
}

/// Explicit reactive lifetime scope. Dropping or calling `cleanup` cancels
/// effects and runs cleanup callbacks owned by this scope.
#[derive(Clone, Debug)]
pub struct Owner(ReactiveOwner);

impl Owner {
    /// Creates an owner nested under the current reactive owner, if present.
    pub fn new() -> Self {
        Self(ReactiveOwner::new())
    }

    /// Creates a detached root owner.
    pub fn new_root() -> Self {
        Self(ReactiveOwner::new_root(None))
    }

    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        self.0.with(f)
    }

    pub fn cleanup(&self) {
        self.0.cleanup();
    }

    pub fn on_cleanup(&self, f: impl FnOnce() + Send + Sync + 'static) {
        self.0.with(|| ReactiveOwner::on_cleanup(f));
    }

    pub fn raw(&self) -> &ReactiveOwner {
        &self.0
    }
}

impl Default for Owner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

    #[test]
    fn state_can_be_read_written_and_updated() {
        let state = State::new(2_i32);
        assert_eq!(state.get(), 2);
        state.set(5);
        state.update(|value| *value += 3);
        assert_eq!(state.get(), 8);
        assert_eq!(state.read_only().get(), 8);
    }

    #[test]
    fn memo_tracks_its_source() {
        let source = State::new(4_i32);
        let source_for_memo = source.clone();
        let doubled = Memo::new(move |_| source_for_memo.get() * 2);
        assert_eq!(doubled.get(), 8);
        source.set(9);
        assert_eq!(doubled.get(), 18);
    }

    #[test]
    fn owner_runs_cleanup_callbacks() {
        let owner = Owner::new_root();
        let cleaned = Arc::new(AtomicBool::new(false));
        let cleaned_in_callback = cleaned.clone();
        owner.on_cleanup(move || cleaned_in_callback.store(true, Ordering::SeqCst));
        owner.cleanup();
        assert!(cleaned.load(Ordering::SeqCst));
    }
}
