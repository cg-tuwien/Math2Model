<script setup lang="ts">
import {
  EngineFactory,
  Effect,
  WebGPUEngine,
  ShaderStore,
} from "@babylonjs/core";
import { MyFirstScene } from "@/scenes/MyFirstScene";
import "@babylonjs/core/Engines/WebGPU/Extensions/";
import CodeEditor from "@/components/CodeEditor.vue";

import { ref, shallowRef, watch } from "vue";
import debounce from "debounce";
import { useElementSize } from "@vueuse/core";
const canvasElement = ref<HTMLCanvasElement | null>(null);
const code = ref(`fn evaluateImage(input2: vec2f) -> vec3f {
    let pos = vec3(input2.x, 0.0, 2. * input2.y) * 3.14159265359;

    let x = sin(pos.x) * cos(pos.z);
    let y = sin(pos.x) * sin(pos.z);
    let z = cos(pos.x);

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    let sphere = vec3(x, y, z) * 3.0;
    let heart = vec3(x2, y2, z2) * 0.2;

    let p = vec3(mix(sphere, heart, 0.) * 1.);

    return p;
}

/*fn evaluateImage(input2: vec2f) -> vec3f {
    let pos = vec3(input2.x, 0.0, input2.y);
    return vec3(input2.xy,0.);
}*/
`);
const engine = shallowRef<WebGPUEngine | null>(null);
const scene = shallowRef<MyFirstScene | null>(null);

const { width, height } = useElementSize(canvasElement);
watch(
  [width, height],
  debounce(() => {
    engine.value?.resize();
  }, 100)
);

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});
watch(
  canvasElement,
  (canvas) => {
    if (!canvas) return;
    engine.value?.dispose();
    EngineFactory.CreateAsync(canvas, {}).then((e) => {
      engine.value = e as WebGPUEngine;
      e.compatibilityMode = false;
      e.getCaps().canUseGLInstanceID = false;
      onCodeChanged();
      scene.value = new MyFirstScene(engine.value);

      engine.value?.runRenderLoop(() => {
        if (scene.value === null) return;
        scene.value.frame++;
        if (engine.value) {
          scene.value.time += engine.value.getDeltaTime();
        }
        scene.value.render();
      });
    });
  },
  { immediate: true }
);

watch(code, onCodeChanged);
function onCodeChanged() {
  if (!engine.value) return;

  engine.value.releaseEffects();
  ShaderStore.ShadersStoreWGSL["customFragmentShader"] = `
        varying vNormal : vec3<f32>;
        varying vUV : vec2<f32>;
        @fragment
        fn main(input : FragmentInputs) -> FragmentOutputs {
            fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
        }
   `;
  ShaderStore.ShadersStoreWGSL["customVertexShader"] = assembleFullVertexShader(
    code.value
  );
  scene.value?.dispose();
  scene.value = new MyFirstScene(engine.value);
}

function assembleFullVertexShader(innerCode: string) {
  // language=HTML
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
  iTime: f32,
  iTimeDelta: f32,
  iFrame: f32,
  width: f32,
};

var<uniform> myUBO: MyUBO;

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

    let pos = evaluateImage(vertexInputs.uv);
    vertexOutputs.position = scene.projection * scene.view * mesh.world * vec4f(pos,1.);
    vertexOutputs.vUV = vertexInputs.uv;
    vertexOutputs.vNormal = vertexInputs.normal;
}`;
}

const setNewCode = debounce((newCode: () => string) => {
  code.value = newCode();
}, 500);
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 90vh">
      <canvas
        ref="canvasElement"
        class="touch-non self-stretch flex-1 overflow-hidden"
      ></canvas>
      <!-- TODO: That's a glsl shader -->
      <CodeEditor
        class="self-stretch flex-1 overflow-hidden"
        :start-code="code"
        @update="setNewCode($event)"
      >
      </CodeEditor>
    </div>
  </main>
</template>

<style scoped></style>
