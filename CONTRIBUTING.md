# Contributing Guide

Features are developed in another branch or fork. After the feature is ready, a pull request to the master branch should be opened.

## Prerequisites
- Node 20 or greater. Don't install Chocolatey.
- A code editor, like VSCode or Webstorm.

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

Remember to use a browser that supports WebGPU.

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

## Tooling

[VSCode](https://code.visualstudio.com/) with

- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) for Vue.js support
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) for code formatting
  - Settings &rarr; Format On Save &rarr; Enable (`"editor.formatOnSave": true,`)
  - If you are using autosave: Settings &rarr; Autosave &rarr; On Focus Change (`"files.autoSave": "onFocusChange",`)
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) for slightly better Tailwind CSS suggestions


I also recommend trying out the following browser extension

- [Vue Devtools](https://devtools.vuejs.org/) to get top-notch Vue.js debugging support

### Code Structure

- `parametric-renderer-core/` - Rust implementation of the renderer. [See Readme there](./parametric-renderer-core/README.md)
- `public/` - files that are directly copied to the generated website
- `src/` - frontend code
- `src/assets/` - assets like pictures or CSS files
- `src/components/` - Vue.js components
- `src/components/exporter/` - code for exporting, turns a vertex buffer into a 3D model (.obj or .glb)
- `src/components/input/` - reusable components for dealing with user input
- `src/components/node-tree/` - used by the side bar for displaying the scene tree and files
- `src/components/visual-programming/` - node graph rendering components
- `src/components/CodeEditor.vue` - Monaco code editor integraton
- `src/components/EditorAndOutput.vue` - Main component, manages the renderer and code editor (either Monaco or visual programming)
- `src/components/FileBrowser.vue` - Displays the filesystem
- `src/components/FocusObject.vue` - Dropdown for selecting the focused object
- `src/components/SceneHierarchy.vue` - Displays the scene, including the object inspector
- `src/components/TopBar.vue` - Navigation Bar, with the `File`, `Export`, etc. menus
- `src/components/WebGpu.vue` - Custom WebGPU code that is exclusively used by the exporter
- `src/content-store/` - a reworked filesystem abstraction, unused
- `src/engine/` - glue code for the native renderer (parametric-renderer-core)
- `src/engine/sync-filesystem.ts` - syncs the textures with the native renderer
- `src/engine/wgpu-engine.ts` - glue code, lets users ignore some messy async-await details
- `src/filesystem/` - deals with saving and loading files. Uses the latest filesystem API together with a sandboxed "origin private filesystem".
- `src/filesystem/scene-file.ts` - type definitions for the scene.json, also generates a JSON schema that we check against
- `src/scenes/` - the CPU state of the scenes
- `src/scenes/example-scene/` - the example scenes
- `src/scenes/aggregate-model-state.ts` - combines multiple models into one state. Used for editing multiple models simultaneously
- `src/stores/` contains Pinia stores. They're globally accessible, reactive objects.
- `src/views/` contains the different pages. Currently the HomeView.vue is the only one that matters.
- `src/App.vue` is the Vue.js rendering entrypoint, it sets up the theme, top bar and the router
- `src/main.ts` is the Typescript entrypoint.
- `index.html` is the entrypoint for the website.
- `package.json` is the dependency file. It also includes "scripts" that can be started with `npm run`.
- `tsconfig.json` is required to tell Typescript what to do.
- `vite.config.ts` is the configuration file for the Vite bundler.


## Useful Documentation

- Vue.js documentation
- TailwindCSS documentation
- [NaiveUI Components](https://www.naiveui.com/en-US/os-theme/components/button)
- [Material Design Icons](https://icon-sets.iconify.design/mdi/), use them with `mdi-icon-name` (e.g. `<mdi-hamburger-menu />`)

Other libraries where one can look up the documentation as needed are
- Typescript
- Pinia: For the `src/stores/`
- Vitest: For unit tests


## High Level Structure

The `index.html` loads the `main.ts` file, which creates a Vue.js instance.
From there `App.vue` gets loaded, which is responsible for the overall layout of the site. It embeds a `<RouterView>`, which is where all the views/pages go. (`HomeView.vue`)

`HomeView.vue` asynchronously loads the filesystem code, and WebGPU engine. They're a bit slow to initialize, so we load them very early, and we're avoiding doing hot module reloading with them. Once they've finished loading, we create our `EditorAndOutput.vue` component.

We then store the code on the user's machine, with the origin private filesystem. Editing the code is done with the Monaco code editor, and with the Rete.js graph editing library.

The scene state is made reactive and editable with typical Vue.js frontend code.

Rendering happens via the WebGPU API, which is used by [parametric-renderer-core](./parametric-renderer-core/), which is implemented in Rust and compiled to WebAssembly.

Exporting then happens in Typescript land, where we use the same WebGPU device from Typescript.

## Deployment

The website is hosted on GitHub Pages. It is deployed using a GitHub action called `deploy`. [The action needs to be manually triggered](https://github.com/cg-tuwien/Math2Model/actions/workflows/deploy.yml).
