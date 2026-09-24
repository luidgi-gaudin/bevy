---
title: Puffin profiler support
authors: ["@luidgi-gaudin"]
pull_requests: []
---

Bevy can now record its spans with [puffin](https://github.com/EmbarkStudios/puffin), the
instrumentation profiler made by Embark Studios, in addition to Tracy and the Chrome tracing format.

puffin shines when looking for frame spikes: its viewer shows the duration of the last frames at a
glance, and lets you select the slowest ones to inspect their flame graph. It is also very easy to
set up, as the viewer is a Rust application that doesn't need to match a specific version:

```sh
cargo install puffin_viewer
puffin_viewer &
cargo run --release --features bevy/trace_puffin
```

Every span of the frame is recorded as a puffin scope, including the `system` spans that Bevy
creates for each system, with the name of the system shown next to the scope. The app listens for
viewer connections on `127.0.0.1:8585`; use the `PUFFIN_SERVER_ADDR` environment variable to
change that address, for example to profile an app running on another device.

Recording can be paused and resumed at runtime, and the `puffin` crate is re-exported so you can
add your own scopes without going through `tracing`:

```rust
use bevy::log::puffin;

fn toggle_profiling(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F12) {
        puffin::set_scopes_on(!puffin::are_scopes_on());
    }
}

fn expensive_system() {
    puffin::profile_scope!("expensive_part");
    // ...
}
```
