# Parametric Renderer Core

This is the core library for the parametric renderer. It is written in Rust, and uses the WGPU library for rendering.

To start the project

```bash
cargo run
```

To get a release build

```bash
cargo build --release
```

To run the benchmarks

```bash
cargo bench
```


## Documentation of each part

The high level structure is that `renderer-core` implements the actual rendering logic, and is then used by `desktop` and `wasm` for the desktop and wasm backends.

Each crate has its own bit of documentation. Notably

- [copy-includes](./copy-includes) is responsible for copy-pasting code in our shader code
- [desktop](./desktop) has the bindings for running the renderer in a desktop environment (as opposed to running the renderer in a web environment)
- [renderer-core](./renderer-core) is the core implementation of the renderer.
- [wasm](./wasm) has the bindings for running the renderer in a web environment


## To update the WGSL shaders

Whenever you are editing the WGSL shaders, you might want to update their "imports". To do so, run

```bash
cargo run --bin copy-includes
```

## Benchmarking

We have multiple forms of benchmarking. The simplest one is pressing `P` at runtime in the desktop environment, which will save a profile of the current frame. 

The proper one uses the `criterion` library. To compare two commits, check out the first one, then run

```bash
cargo bench -p renderer-core -- --save-baseline base
```

Then check out the second commit, and run

```bash
cargo bench -p renderer-core -- --baseline base
```

[Documentation link](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html#baselines).
