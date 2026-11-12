# rust-mvvm

A small, framework-independent MVVM core for Rust, with a GPUI runtime adapter.
The design borrows the fine-grained reactive graph and explicit ownership model
from Leptos, and the ViewModel/Command separation commonly found in ReactiveUI.

## Workspace

```text
crates/
  mvvm-reactive/  # owned State, read-only state, Memo, Owner
  mvvm-core/      # ViewModel marker, lifecycle, Command, activation state
  mvvm-gpui/      # GPUI task scheduling and redraw integration
examples/
  todo-app/       # small GPUI application using the MVVM core
```

## Quick start

```bash
cargo test --workspace
cargo check --workspace
cargo run -p todo-app
```

GPUI Kit 0.9.1 requires a recent Rust toolchain and native platform dependencies.
See the [GPUI Kit installation guide](https://gpui-kit.com/docs/installation/).

## Architecture rules

- `mvvm-reactive` and `mvvm-core` do not depend on GPUI.
- Only `mvvm-gpui` and UI applications may import GPUI types.
- Background work must capture owned `Send + 'static` inputs and return owned
  data. It must not access GPUI `Entity`, `Window`, or `Context` values.
- Foreground GPUI tasks may update entities after awaiting. Keep their `Task`
  handles alive or explicitly detach them: dropping a task cancels it.
- A Leptos `Owner`'s current context is thread-local. Create/use/clean up owners
  in the intended reactive execution context; do not move an owner across
  threads as a substitute for scheduling work on the UI executor.
- `State::read_only()` exposes a read-only handle to consumers. Keep the
  writable `State` inside the owning ViewModel or application service.

## Current scope

This is an initial, deliberately small foundation. It includes synchronous
commands and a minimal GPUI task adapter. It does not yet provide a universal
async-command abstraction, automatic cross-thread reactive notifications,
a DI container, navigation, or code generation. Those should be layered on
after threading, ownership, and cancellation semantics are validated against a
real application.

## License

MIT OR Apache-2.0
