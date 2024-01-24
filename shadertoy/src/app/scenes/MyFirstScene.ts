import { Engine, Scene, FreeCamera, Vector3, MeshBuilder, StandardMaterial, Color3, HemisphericLight, ShaderMaterial } from 'babylonjs';

export class MyFirstScene extends Scene {
  constructor(engine: Engine) {
    super(engine);
    // This creates and positions a free camera (non-mesh)
    var camera = new FreeCamera("camera1", new Vector3(0, 5, -10), this);

    // This targets the camera to scene origin
    camera.setTarget(Vector3.Zero());

    // This attaches the camera to the canvas
    camera.attachControl(this.getEngine().getRenderingCanvas());

    // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
    var light = new HemisphericLight("light1", new Vector3(0, 1, 0), this);

    // Default intensity is 1. Let's dim the light a small amount
    light.intensity = 0.7;

    BABYLON.Effect.ShadersStore['customVertexShader'] = `
        precision highp float;
        attribute vec3 position;
        uniform mat4 worldViewProjection;
        
        void main() {
            vec4 p = vec4(position, 1.);
            gl_Position = worldViewProjection * p;
        }
    `;

    BABYLON.Effect.ShadersStore['customFragmentShader'] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;


    var shaderMaterial = new ShaderMaterial('custom', this, 'custom', {
    });
    shaderMaterial.allowShaderHotSwapping = true;

    // Create a built-in "box" shape; with 2 segments and a height of 1.
    var box = MeshBuilder.CreateBox("box", {size: 2}, this);
    const material = new StandardMaterial("box-material", this);
    material.diffuseColor = Color3.Blue();
    box.material = shaderMaterial;



    var ground = MeshBuilder.CreateGround("ground", {width: 6, height: 6}, this);
    const groundMaterial = new StandardMaterial("ground-material", this);
    groundMaterial.diffuseColor = Color3.Green();
    ground.material = shaderMaterial;

  }

  
    // Für Hot reloading
    resetMaterials(): void
    {
      this.markAllMaterialsAsDirty(BABYLON.Constants.MATERIAL_AllDirtyFlag)
    }
}
