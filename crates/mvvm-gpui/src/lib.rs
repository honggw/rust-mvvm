//! GPUI runtime adapter. GPUI-specific types are deliberately confined here.

use std::future::Future;
use gpui_kit::{AsyncApp, Context, Task, WeakEntity};

pub use mvvm_core::{ActivationState, Activatable, Command, CommandError, Memo, Owner, ReadOnlyState, State, Unit, ViewModel};

/// Spawn a future on GPUI's foreground executor. The returned handle must be
/// retained or explicitly detached; dropping it cancels unfinished work.
pub fn spawn_foreground<T, F, R>(cx: &Context<T>, future: F) -> Task<R>
where
    T: 'static,
    F: Future<Output = R> + 'static,
    R: 'static,
{
    cx.spawn(async move |_entity: WeakEntity<T>, _app: &mut AsyncApp| future.await)
}

/// Spawn `Send` work on GPUI's background executor. This future must only
/// capture owned, thread-safe data; it must not access GPUI entities/windows.
pub fn spawn_background<T, F, R>(cx: &Context<T>, future: F) -> Task<R>
where
    T: 'static,
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    cx.background_spawn(future)
}

/// A convenience trait for entities whose rendered output is driven by an
/// external reactive source. Call `reactive_changed` after synchronizing the
/// source into the view model to schedule a GPUI redraw.
pub trait ReactiveGpuiView: Sized + 'static {
    fn reactive_changed(&mut self, cx: &mut Context<Self>) {
        cx.notify();
    }
}
