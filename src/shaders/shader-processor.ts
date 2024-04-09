export function assembleFullVertexShader(innerCode: string) {
  return `
#include<sceneUboDeclaration>
#include<meshUboDeclaration>

attribute position : vec3<f32>;
attribute uv: vec2<f32>;
attribute normal : vec3<f32>;

varying vUV : vec2<f32>;
varying vNormal : vec3<f32>;

// TODO: Make them global
struct MyUBO {
  width: f32,
};

struct GlobalUBO {
  iTime: f32,
  iTimeDelta: f32,
  iFrame: f32,
}

var<uniform> myUBO: MyUBO;
var<uniform> globalUBO: GlobalUBO;

${innerCode}

@vertex
fn main(input: VertexInputs) -> FragmentInputs {
    let actualWidth = ceil(sqrt(myUBO.width));
    let ind = f32(input.instanceIndex);

    let cellSize = 1. / actualWidth;

    var xSliced = vertexInputs.uv;
    xSliced.x /= actualWidth;
    xSliced.x += floor(ind/actualWidth)*cellSize;
    xSliced.y /= actualWidth;
    xSliced.y += (ind%actualWidth)*cellSize;
    vertexInputs.uv = xSliced;

    ${
      innerCode !== ""
        ? "let pos = evaluateImage(vertexInputs.uv);"
        : "let pos = vec3f(vertexInputs.uv, 0.0);"
    }
    vertexOutputs.position = scene.projection * scene.view * mesh.world * vec4f(pos,1.);
    vertexOutputs.vUV = vertexInputs.uv;
    vertexOutputs.vNormal = vertexInputs.normal;
}`;
}
