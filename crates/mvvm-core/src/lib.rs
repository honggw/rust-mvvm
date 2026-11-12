//! UI-framework-agnostic MVVM building blocks.

use std::{fmt, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use thiserror::Error;

pub use mvvm_reactive::{Memo, Owner, ReadOnlyState, State};

/// Marker trait for view models. It intentionally has no GPUI dependency.
pub trait ViewModel: 'static {}

impl<T: 'static> ViewModel for T {}

/// Lifecycle contract for view models that need activation/deactivation hooks.
pub trait Activatable {
    fn activate(&mut self) {}
    fn deactivate(&mut self) {}
}

/// Tracks activation state without prescribing a UI lifecycle.
#[derive(Clone, Debug, Default)]
pub struct ActivationState(Arc<AtomicBool>);

impl ActivationState {
    pub fn is_active(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn activate(&self) -> bool {
        !self.0.swap(true, Ordering::AcqRel)
    }

    pub fn deactivate(&self) -> bool {
        self.0.swap(false, Ordering::AcqRel)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CommandError {
    #[error("command is currently disabled")]
    Disabled,
    #[error("command failed: {0}")]
    Failed(Arc<str>),
}

impl CommandError {
    pub fn failed(message: impl Into<Arc<str>>) -> Self {
        Self::Failed(message.into())
    }
}

/// A synchronous command with a separate availability predicate.
///
/// Both callbacks are `Send + Sync`; commands can therefore be shared between
/// application services and UI adapters without capturing thread-bound views.
#[derive(Clone)]
pub struct Command<P> {
    can_execute: Arc<dyn Fn(&P) -> bool + Send + Sync>,
    execute: Arc<dyn Fn(P) -> Result<(), CommandError> + Send + Sync>,
}

impl<P> fmt::Debug for Command<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command").finish_non_exhaustive()
    }
}

impl<P> Command<P> {
    pub fn new(
        can_execute: impl Fn(&P) -> bool + Send + Sync + 'static,
        execute: impl Fn(P) -> Result<(), CommandError> + Send + Sync + 'static,
    ) -> Self {
        Self { can_execute: Arc::new(can_execute), execute: Arc::new(execute) }
    }

    pub fn always(execute: impl Fn(P) -> Result<(), CommandError> + Send + Sync + 'static) -> Self {
        Self::new(|_| true, execute)
    }

    pub fn can_execute(&self, parameter: &P) -> bool {
        (self.can_execute)(parameter)
    }

    pub fn execute(&self, parameter: P) -> Result<(), CommandError> {
        if !(self.can_execute)(&parameter) {
            return Err(CommandError::Disabled);
        }
        (self.execute)(parameter)
    }
}

/// Small helper for representing a command parameter that carries no data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Unit;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    #[test]
    fn command_checks_availability_before_execution() {
        let total = Arc::new(AtomicUsize::new(0));
        let total_in_command = total.clone();
        let command = Command::new(
            |amount: &usize| *amount > 0,
            move |amount| {
                total_in_command.fetch_add(amount, Ordering::SeqCst);
                Ok(())
            },
        );
        assert_eq!(command.execute(0), Err(CommandError::Disabled));
        command.execute(3).unwrap();
        assert_eq!(total.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn activation_state_reports_transitions() {
        let state = ActivationState::default();
        assert!(!state.is_active());
        assert!(state.activate());
        assert!(!state.activate());
        assert!(state.deactivate());
        assert!(!state.deactivate());
    }
}
