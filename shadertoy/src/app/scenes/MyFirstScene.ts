import { Engine, Scene, Vector3, MeshBuilder, HemisphericLight, ShaderMaterial, FlyCamera } from 'babylonjs';

export class MyFirstScene extends Scene {

  private ground: any;

  constructor(engine: Engine) {
    super(engine);
    // This creates and positions a free camera (non-mesh)
    var camera = new FlyCamera("camera1", new Vector3(0, 5, -10), this);

    // This targets the camera to scene origin
    // camera.setTarget(Vector3.Zero());

    // This attaches the camera to the canvas
    camera.attachControl(true);

    // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
    var light = new HemisphericLight("light1", new Vector3(0, 1, 0), this);

    // Default intensity is 1. Let's dim the light a small amount
    light.intensity = 0.7;

    BABYLON.Effect.ShadersStore['customVertexShader'] = `
precision highp float;
attribute vec3 position;
attribute vec2 uv;
uniform mat4 worldViewProjection;

void main() {

    vec3 pos = vec3(uv.x, 0.0, uv.y) * 3.14159265359 * 2.;

    float x = sin(pos.x) * cos(pos.z);
    float y = sin(pos.x) * sin(pos.z);
    float z = cos(pos.x);

    float x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    float y2 = 8. * cos(pos.x);
    float z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    vec3 sphere = vec3(x, y, z);
    vec3 heart = vec3(x2, y2, z2);

    vec4 p = vec4(mix(sphere, heart, 0.) * 1., 1.);
    gl_Position = worldViewProjection * p;
}
    `;

    BABYLON.Effect.ShadersStore['customFragmentShader'] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;


    var shaderMaterial = new ShaderMaterial('custom', this, 'custom', {attributes: ['uv', 'position']});
    shaderMaterial.allowShaderHotSwapping = true;

    // Create a built-in "box" shape; with 2 segments and a height of 1.
    //this.box = MeshBuilder.CreateBox("box", {size: 2}, this);
    //this.box.material = shaderMaterial;



    this.ground = MeshBuilder.CreateGround("ground", {width: 3.14159265359 * 2, height: 3.14159265359 * 2, subdivisions: 50}, this);
    this.ground.position = new Vector3(3.14159265359, 0.0, 3.14159265359);
    this.ground.material = shaderMaterial;

  }


    // Für Hot reloading
    resetMaterials(): void
    {
      this.ground.material.dispose();
      var shaderMaterial = new ShaderMaterial('custom', this, 'custom', {
      });
      console.log(shaderMaterial);
      //this.box.material = shaderMaterial;
      this.ground.material = shaderMaterial;
    }
}
