// A dummy implementation of sampleObject that we use for compiling
import package::scene_globals::{time, screen, mouse};

fn sampleObject(input: vec2f) -> vec3f {
  // Reference those so that wgsl_to_wgpu will include them when compiling
  let a = time;
  let b = screen;
  let c = mouse;
  return vec3(input, 0.0); 
}