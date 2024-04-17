# Copy Include

Copies code from one shader into another one.

## Usage

```glsl
#include "./file.glsl"
```

## Limitations

It's absolutely shoddy. The idea is to give us *something* until https://github.com/wgsl-analyzer/wgsl-analyzer and https://github.com/PolyMeilex/vscode-wgsl and https://github.com/ScanMountGoat/wgsl_to_wgpu fully support naga_oil includes. See also https://github.com/bevyengine/naga_oil/issues/83 .

It's so shoddy that it can only handle one level of includes.