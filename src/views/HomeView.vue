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
const canvasElement = ref<HTMLCanvasElement | null>(null);
const code = ref(`fn mainImage(input: VertexInputs) -> vec4<f32> {
    let pos = vec3(input.uv.x, 0.0, 2. * input.uv.y) * 3.14159265359;

    let x = sin(pos.x) * cos(pos.z);
    let y = sin(pos.x) * sin(pos.z);
    let z = cos(pos.x);

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    let sphere = vec3(x, y, z) * 3.0;
    let heart = vec3(x2, y2, z2) * 0.2;

    let p = vec4(mix(sphere, heart, 0.) * 1., 1.)
    + vec4(f32(input.instanceIndex)* 10.1, 0.0, 0.0, 1.0);

    //let p = vec4(pos.x, pos.y, pos.z, 1.0) + vec4(0.0, f32(input.vertexIndex)* 0.1, 0.0, 0.0);
    return scene.projection * scene.view * mesh.world * p;
}`);
const engine = shallowRef<WebGPUEngine | null>(null);
const scene = shallowRef<MyFirstScene | null>(null);

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
        @fragment
        fn main(input : FragmentInputs) -> FragmentOutputs {
            fragmentOutputs.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
   `;
  ShaderStore.ShadersStoreWGSL["customVertexShader"] = assembleFullVertexShader(
    code.value
  );
  scene.value?.resetMaterials();
}

function assembleFullVertexShader(innerCode: string) {
  return `
#include<sceneUboDeclaration>
#include<meshUboDeclaration>

attribute position : vec3<f32>;
attribute uv: vec2<f32>;

varying vUV : vec2<f32>;

// TODO: Make them global
struct MyUBO {
  iTime: f32,
  iTimeDelta: f32,
  iFrame: f32,
};

var<uniform> myUBO: MyUBO;

${innerCode}

@vertex
fn main(input: VertexInputs) -> FragmentInputs {
    vertexOutputs.position = mainImage(input);
    vertexOutputs.vUV = vertexInputs.uv;
}    
  
  `;
}

const setNewCode = debounce((newCode: () => string) => {
  code.value = newCode();
}, 500);
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="min-height: 90vh">
      <canvas ref="canvasElement" class="touch-non"></canvas>
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

<style scoped>
canvas {
  width: 50%;
  height: 50%;
}
</style>
