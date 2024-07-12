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

## Controls

- Right click, and then `W` `A` `S` `D` to move the camera.
- Right click, and then `Space` `Shift` to move the camera up and down.
- `P` to get a benchmark of the current frame. It gets written to a `profile-*.json` file and can be viewed on [ui.perfetto.dev](https://ui.perfetto.dev/).



## To update the WGSL shaders

Whenever you are editing the WGSL shaders, you might want to update their "imports". To do so, run

```bash
cargo run --bin copy-includes
```

## Benchmarking

We have multiple forms of benchmarking. The simplest one is pressing `P` at runtime, which will save a profile of the current frame. 

The proper one uses the `criterion` library. To compare two commits, check out the first one, then run

```bash
cargo bench -p renderer-core -- --save-baseline base
```

Then check out the second commit, and run

```bash
cargo bench -p renderer-core -- --baseline base
```

[Documentation link](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html#baselines).

## Developer Notes

- WGPU Tutorial https://sotrh.github.io/learn-wgpu/#what-is-wgpu
- WGSL to Rust library https://github.com/ScanMountGoat/wgsl_to_wgpu
- Math library https://docs.rs/glamour/latest/glamour/
