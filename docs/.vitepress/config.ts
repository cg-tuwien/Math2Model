import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Math2Model Documentation",
  description:
    "Documentation for Math2Model - the shadertoy of parametric modeling",
  base: "/Math2Model/docs/",
  outDir: "../dist/docs/",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [{ text: "Go to Math2Model", link: "../" }],

    sidebar: [
      {
        text: "Documentation",
        link: "/",
        items: [
          { text: "Graph Based Shapes", link: "/graph-based-shapes" },
          { text: "Programmatic Shapes", link: "/programmatic-shapes" },
          { text: "Exporter", link: "/exporter" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/cg-tuwien/Math2Model" },
    ],
    search: {
      provider: "local",
    },
  },
});
