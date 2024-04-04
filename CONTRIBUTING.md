# Contributing Guide

Features are developed in another branch or fork. After the feature is ready, a pull request to the master branch should be opened.

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

- [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) for Vue.js support
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) for code formatting
  - Settings (`Ctrl+j`) &rarr; Format On Save &rarr; Enable (`"editor.formatOnSave": true,`)
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) for slightly better Tailwind CSS suggestions


### Code Structure

- `public/` contains files that are simply copied to the output. Very rarely used.
- `src/` has all the code.
- `src/assets/` contains assets like pictures or CSS files that do not have a better location. Rarely used.
- `src/components/` contains proper Vue.js components that can be used everywhere.
- `src/filesystem/` deals with saving and loading files. Uses the latest filesystem API together with a sandboxed "origin private filesystem".
- `src/router/` contains the Vue.js router stuff, reponsible for mapping URLs to different pages/views.
- `src/scenes/` contains the existing Babylon.js scenes. 
- `src/shaders/` contains a shader transformer.
- `src/stores/` contains Pina stores. They're globally accessible, reactive objects.
- `src/views/` contains the different pages. Currently the HomeView.vue is the only one that matters.
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
- Pinia: `src/stores/`
- Vitest: Unit tests



## High-Level Documentation

### Important Terms

- CAS: Computer Algebra System, this is a piece of magic that takes formulas or equations and evaluates them
- MathJson: [A format](https://cortexjs.io/math-json/) to unambiguously represent mathematical expressions
- Document: A document consists of a number of elements such as
  - Mathematical Expressions
  - Scopes
  - Text Blocks



### Design

- Equation Editor (frontend)
  - We're using mathlive, however other equation editors could also be used
  - Every equation editor has a corresponding element in the document
  - During typing, the backend can optionally be asked to
    - Return the variables (values, functions, units) that are in scope
    - Quickly evaluate the expression
    - Suggest autocompletions
    - Return information about expressions, such as 'variable $a$ is a vector' or 'the $\cdot$ sign here stands for dot product'
  - When the user hits Enter (submit), then
    - The expression gets parsed and turned into MathJson
      - Placeholders get added where the result should go
    - The expression gets sent to the backend
    - Eventually, the result gets returned
- Backend
  - A general backend that doesn't know how to evaluate expressions. Instead, it keeps track of the entire document, including
    - Expressions
    - Scopes
    - Imports
    - Variables
- CAS
  - We're using Sympy, however support for more computer algebra systems is planned
  - The CAS accepts commands
    - id: Every command has an ID to identify it
    - expression: Every command has some expression to evaluate
    - gettersData: When an expression contains some variables, they will be included here
    - callback: A function to call when the result is ready. Can be called multiple times, to return partial results
  - It translates the MathJson expression into something the CAS understands (e.g. Sympy-Python Code)
  - Then evaluates the expression (e.g. in a web worker)
  - And finally, converts the result back to MathJson (e.g. A sympy printer)

