import { Engine, Scene, FreeCamera, Vector3, MeshBuilder, StandardMaterial, Color3, HemisphericLight } from "babylonjs";
const createScene = (canvas) => {
    const engine = new Engine(canvas);
    const scene = new Scene(engine);

    const camera = new FreeCamera("camera1", new Vector3(0, 5, -10), scene);
    camera.setTarget(Vector3.Zero());
    camera.attachControl(canvas, true);

    new HemisphericLight("light", Vector3.Up(), scene);

    const box = MeshBuilder.CreateBox("box", { size: 2 }, scene);
    const material = new StandardMaterial("box-material", scene);
    material.diffuseColor = Color3.Blue();
    box.material = material;

    // Für Hot reloading
    scene.markAllMaterialsAsDirty(BABYLON.Constants.MATERIAL_AllDirtyFlag)

    engine.runRenderLoop(() => {
        scene.render();
    });
};

export { createScene };