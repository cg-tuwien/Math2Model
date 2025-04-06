# WASM Bindings

This crate contains the wasm bindings for the renderer. 
You'll find 
- logging and executor setup
- the winit setup dance
- and functions that are call-able from JS. Wasm-bindgen and Tsify are used for generating the bindings 

One challenge is that all the function must be async or fire-and-forget. We have a winit event loop in Rust land, and functions can only mutate data when called from within the event loop.

So calling something from JS land has a flow of "JS -> generated bindings -> Rust function -> note down *commands* to do on the next event loop iteration -> ... event loop picks up on the commands ... -> done"