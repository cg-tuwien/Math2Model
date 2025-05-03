# Copy Include

Copies code from one shader into another one. Currently, WGSL needs all shader code to be in the same file.

We run the copying step beforehand, such that WGSL language servers can actually understand and analyze the code. The second tool that relies on this is [wgsl_to_wgpu](https://github.com/ScanMountGoat/wgsl_to_wgpu), which we run at build time to generate type-safe Rust bindings for our shader code.

## Usage

```glsl
#include "./file.glsl"
```

## Limitations

It's absolutely shoddy. The idea is to give us *something* until https://github.com/wgsl-analyzer/wgsl-analyzer and https://github.com/PolyMeilex/vscode-wgsl and https://github.com/ScanMountGoat/wgsl_to_wgpu fully support naga_oil includes. See also https://github.com/bevyengine/naga_oil/issues/83 .

It's so shoddy that it can only handle one level of includes.
