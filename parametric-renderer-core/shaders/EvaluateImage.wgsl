struct Time {
  elapsed: f32,
  delta: f32,
  frame: u32,
}
struct Screen {
  resolution: vec2<u32>,
  inv_resolution: vec2<f32>,
}
struct Mouse {
  pos: vec2<f32>,
  buttons: u32,
}
struct Extra {
  hot_value: f32
}
fn mouse_held(button: u32) -> bool {
  return (mouse.buttons & button) != 0u;
}
// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;
@group(0) @binding(3) var<uniform> extra : Extra;
