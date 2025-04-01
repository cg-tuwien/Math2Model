# Graph Based Shapes

This purely graphical shader programming solution has been designed to provide easier access to parametric modelling for people who are not that proficient with programming. Creating a new model and selecting the `New Graph` option will create a `<filename>.graph` and a `<filename>.graph.wgsl` file. The `.wgsl` file is automatically generated from the code graph and always updated when changes occur.

## Code Graph Overview - Heart-Sphere example

When opening the "Math2Model" Web application for the first time you will be greeted with a simple scene rendering a Heart. This heart represents the "MorphHeartSphere" model in the Scene Hierarchy on the left side. Clicking on the model in the Scene Hierarchy will open the code graph of the model on the right side.

![Web Application view after clicking on MorphHeartSphere model](resources/graphbased1.png)

After clicking on the model the application will look similar to the figure above. In the code graph you can drag the view around by holding the left mouse button and dragging the mouse, zoom in and out with the scroll wheel, select nodes by clicking on them, select multiple nodes by holding CTRL while clicking them, open a context menu with various options by pressing the right mouse button and create new nodes by dragging them from the nodes list on the left side into the graph. You can always view the generated code file by pressing on the icon on the top-right corner of the code graph.

Let's go through the code graph of the Heart Sphere Morph Example. It starts by with the `input2` node, which will be automatically provided in each code graph and represents the input coordinates of the plane that is parametrically shaped. This initial node is connected to the `Heart` and `Sphere` node, which can be found under the `Shapes` category in the nodes list. This flow is common for code graphs, the output of the `input2` node is dragged onto the input of the `Heart` and `Sphere` node. The generated code should look something like this (see also under `heart-sphere.graph.wgsl`).

```ts
var ref_831ed = Sphere(input2); // this creates a vec3f reference to the value of Sphere(input2)
var ref_fd7fa = Heart(input2); // this creates a vec3f reference to the value of Heart(input2)
```

The `Shapes` nodes typically transform a two-dimensional input into a three-dimensional output and can be straight up used as the `Return value` on the `Return` node, which is always marked red. In this example the values get transformed a little bit further. The `Heart` output gets multiplied with `0.3` to scale it down a bit, using a `Multiply` node, which can be found under the `Maths` section (`Note: The search function in the nodes list can come in handy, if you have trouble finding the correct node.`). This node comes with two input fields and two input connection points, so users can either provide values by hand or use generated values from other nodes. In this example the left operand uses the generated value from the `Heart` node and the right operand is given by hand. Lastly both the transformed `Heart` output and the `Sphere` output are put into a `Combine` node which can be found under the `Apply` section. This uses the shader's mix function to interpolate between the first input shape and the second input shape. Drag the `combine factor` slider to see the effects. To let the application actually render the resulting shape you have to connect the final output with the `Return` node.
