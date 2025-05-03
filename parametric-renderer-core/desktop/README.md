# Desktop Bindings

This sets up the application to run in a desktop environment. It uses [winit](https://docs.rs/winit/latest/winit/) and the [renderer-core](./renderer-core)

With this, the renderer can be debugged with Renderdoc and other standard GPU debuggers. This is the recommended flow for debugging GPU issues.

It is much simpler than the `wasm` version, because it does not expose any functions that can be called from the outside.
