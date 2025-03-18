# Documentation for Math2Model

Math2Model is a tool for creating scenes with parametrically defined 3D objects.

It offers two main modes, targeted at different audiences.

At the end, the objects can be exported into a standard 3D mesh file format.

## Graph based objects

Build your 3D objects using a visual, node-based editor. 
This is the recommended approach for rapid prototyping.

[Read more](./graph-based-shapes.md)

## Programmatic objects

Build your 3D objects using code.
This lower level approach is useful for expressing things 
that are difficult to express in the graph based approach.
It assumes basic GPU shader programming knowledge.

[Read more](./programmatic-shapes.md)

## Exporting scenes

Export your 3D objects to commonly used mesh based 3D file formats for further usage.
[Read more](./exporter.md)


## For Leo and Ferris

1. Check out [some markdown examples](./markdown-examples)
2. Write your stuff in markdown files
3. Add it to the `docs/.vitepress/config.ts` file so that your pages show up in the side bar
