# Desktop Bindings

This sets up the application to run in a desktop environment. It uses [winit](https://docs.rs/winit/latest/winit/) and the [renderer-core](./renderer-core).
Upon startup, it'll create or load [a config file with the camera position](https://github.com/cg-tuwien/Math2Model/blob/main/parametric-renderer-core/desktop/src/config.rs). This config file is autosaved on shutdown.

With this, the renderer can be debugged with Renderdoc and other standard GPU debuggers. This is the recommended flow for debugging GPU issues.

It is much simpler than the `wasm` version, because it does not expose any functions that can be called from the outside.


## Controls

- Right click, and then `W` `A` `S` `D` to move the camera.
- Right click, and then `Space` `Shift` to move the camera up and down.
- `P` to get a benchmark of the current frame. It gets written to a `profile-*.json` file and can be viewed on [ui.perfetto.dev](https://ui.perfetto.dev/).

