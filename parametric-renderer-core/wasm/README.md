# WebAssembly (WASM) Bindings for the Renderer

This crate provides WebAssembly bindings to use the [renderer-core](./renderer-core] in web environments. It serves as the bridge between Rust and the browser, enabling seamless interaction between the frontend code and low-level renderer logic written in Rust.

## Overview

The primary responsibilities of this crate include:

* **Logging and Async Executor Setup**: Initialization of logging infrastructure and async runtime for the browser environment.
* **Winit Setup**: Bootstrapping the `winit` application.
* **Exposing JS-Callable Functions**: Using [`wasm-bindgen`](https://rustwasm.github.io/wasm-bindgen/) and [`tsify`](https://github.com/AmbientRun/tsify-next) to generate TypeScript-compatible bindings for JavaScript to call into Rust.

## Multiple Event Loops

`winit` sets up its own event loop, where actions (keyboard inputs, mouse inputs, redraw events) are handled. The ownership of the renderer is also in the winit event loop.

This, however, means that to call it from the outside, one cannot directly call functions on the renderer. Rusts ownership model does not allow that, since it would be an potential race condition.

Instead, we opt for scheduling function calls onto the winit event loop, and asynchronously finishing them.

```text
JavaScript frontend
   ↓
Generated WASM Bindings
   ↓
Rust Function
   ↓
Push command to an internal queue
   ↓
Rust Event Loop (effectively polls the queue on each frame)
   ↓
Executes the queued command
```

## File Structure

* `lib.rs`: Crate entry point. Initializes the logger and the async runtime.
* `application.rs` Contains an "application" object that can be created from JS land. All public functions can be called from JS, with `wasm-bindgen` generating the glue.
* `wasm_abi.rs`: Defines the types and structs used in JS-Rust interaction. 

## Tooling

This crate uses the following tools:

* [`wasm-bindgen`](https://rustwasm.github.io/wasm-bindgen/): Enables high-level JS bindings for Rust functions.
* [`tsify`](https://github.com/AmbientRun/tsify-next): Automatically generates TypeScript declarations from Rust structs to ensure type safety in JavaScript/TypeScript land.
* [`console_log`](https://docs.rs/console_log): Redirects Rust `log` macros to the browser console.
* [`winit`](https://crates.io/crates/winit): Cross-platform window/event handling.
