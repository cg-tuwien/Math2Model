<script setup lang="ts">
import BabylonScene from './components/BabylonScene.vue';
</script>

<script lang="ts">
import loader from "@monaco-editor/loader";

export default {
    name: "Editor",
    async mounted() {

        loader.init().then((monaco) => {
            const editorOptions = {
                language: "wgsl",
                minimap: { enabled: false },
                theme: "hc-black",
            };

            monaco.editor.create(document.getElementById("editor"),   editorOptions);

        });

    },
};
</script>

<template>
  <div id="editor" style="width: 500px; height: 500px"></div>
  <div style="text-align: center">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo" alt="Vite logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
    <a href="https://vitejs.dev" target="_blank">
        <img src="/vite.svg" class="logo" alt="Vite logo" />
    </a>
  </div>
  <BabylonScene/>
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>

