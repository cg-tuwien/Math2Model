<script setup lang="ts">
import { EngineFactory, Effect, WebGPUEngine } from "@babylonjs/core";
import { MyFirstScene } from "@/scenes/MyFirstScene";
import "@babylonjs/core/Engines/WebGPU/Extensions/";
import CodeEditor from "@/components/CodeEditor.vue";

import { ref, shallowRef, watch } from "vue";
import debounce from "debounce";
const canvasElement = ref<HTMLCanvasElement | null>(null);
const code = ref(`vec4 mainImage() {
    vec3 pos = vec3(uv.x, 0.0, 2. * uv.y) * 3.14159265359;

    float x = sin(pos.x) * cos(pos.z);
    float y = sin(pos.x) * sin(pos.z);
    float z = cos(pos.x);

    float x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    float y2 = 8. * cos(pos.x);
    float z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    vec3 sphere = vec3(x, y, z) * 3.0;
    vec3 heart = vec3(x2, y2, z2) * 0.2;

    vec4 p = vec4(mix(sphere, heart, 0.5) * 1., 1.);
    return worldViewProjection * p;
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
      scene.value = new MyFirstScene(engine.value);
      onCodeChanged();

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
  Effect.ShadersStore["customFragmentShader"] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;
  Effect.ShadersStore["customVertexShader"] = assembleFullVertexShader(
    code.value
  );
  scene.value?.resetMaterials();
}

function assembleFullVertexShader(innerCode: string) {
  let prefix =
    "precision highp float;\n" +
    "attribute vec3 position;\n" +
    "attribute vec2 uv;\n" +
    "uniform float iTime;\n" +
    "uniform float iTimeDelta;\n" +
    "uniform float iFrame;\n" +
    "uniform mat4 worldViewProjection;\n";
  let call = "" + "void main() {" + "gl_Position = mainImage();" + "}";
  return prefix + "\n" + innerCode + "\n" + call;
}

const setNewCode = debounce((newCode: () => string) => {
  code.value = newCode();
}, 500);
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="min-height: 90vh">
      <canvas ref="canvasElement" class="touch-non"></canvas>
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
