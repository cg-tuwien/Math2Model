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

### Run Unit Tests with [Vitest](https://vitest.dev/)

```sh
npm run test:unit
```


## Tooling

[VSCode](https://code.visualstudio.com/) with

- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) for Vue.js support
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) for code formatting
  - Settings &rarr; Format On Save &rarr; Enable (`"editor.formatOnSave": true,`)
  - If you are using autosave: Settings &rarr; Autosave &rarr; On Focus Change (`"files.autoSave": "onFocusChange",`)
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) for slightly better Tailwind CSS suggestions


I also totally recommend trying out the following browser extensions

- [Vue Devtools](https://devtools.vuejs.org/) to get top-notch Vue.js debugging support

### Code Structure

- `public/` contains files that are simply copied to the output. Very rarely used.
- `src/` has all the code.
- `src/assets/` contains assets like pictures or CSS files that do not have a better location. Rarely used.
- `src/components/` contains proper Vue.js components that can be used everywhere.
- `src/filesystem/` deals with saving and loading files. Uses the latest filesystem API together with a sandboxed "origin private filesystem".
- `src/router/` contains the Vue.js router stuff, reponsible for mapping URLs to different pages/views.
- `src/scenes/` contains the existing Babylon.js scenes. 
- `src/stores/` contains Pinia stores. They're globally accessible, reactive objects.
- `src/views/` contains the different pages. Currently the HomeView.vue is the only one that matters.
- `src/App.vue/` is the main Vue.js file.
- `src/main.ts` is the Typescript entrypoint.
- `index.html` is the entrypoint for the website.
- `package.json` is the dependency file. It also includes "scripts" that can be started with `npm run`.
- `tsconfig.json` is required to tell Typescript what to do. It's split up, because there are technically multiple different Typescript configs that apply to our code.
  - `tsconfig.app.json` is for the website.
  - `tsconfig.node.json` is for configuration files, like the `vite.config.ts`.
  - `tsconfig.vitest.json` is for the unit tests. It tries to mimick the website's config.
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

`HomeView.vue` asynchronously loads the filesystem code, and Babylon.js. They're a bit slow to initialize, so we load them very early, and we're avoiding doing hot module reloading with them. Once they've finished loading, we create our `EditorAndOutput.vue` component.

