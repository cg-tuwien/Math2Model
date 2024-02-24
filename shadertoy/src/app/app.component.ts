import {AfterViewInit, Component, ElementRef, NgZone, ViewChild} from '@angular/core';
import {EngineFactory, Effect, Engine, WebGPUEngine} from "@babylonjs/core";
import {MyFirstScene} from "./scenes/MyFirstScene";
import {CodeModel} from "@ngstack/code-editor";
import '@babylonjs/core/Engines/WebGPU/Extensions/';

import * as BABYLON from "@babylonjs/core";
(window as any).BABYLON = BABYLON;

// shadows
import '@babylonjs/core/Lights/Shadows/shadowGeneratorSceneComponent';

// texture loading
import '@babylonjs/core/Materials/Textures/Loaders/envTextureLoader'
// needed for skybox textur'
import '@babylonjs/core/Misc/dds'
// edge'
import '@babylonjs/core/Rendering/edgesRenderer'
// gltf'loadin'
import '@babylonjs/loaders/glTF/2.0'
// anim'tion'
import '@babylonjs/core/Animations/animatable'
// import {WebGPUEngine} from "@babylonjs/core";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})

export class AppComponent implements AfterViewInit {
  @ViewChild('canvas', {static: true}) private canvas?: ElementRef<HTMLCanvasElement>;
  private engine?: WebGPUEngine;
  private scene?: MyFirstScene;

  //private ngZone: NgZone = new NgZone([]);
  constructor(private ngZone: NgZone) {}
  async ngAfterViewInit(): Promise<void> {
    if (this.canvas) {
      const webGPUSupported = await WebGPUEngine.IsSupportedAsync;
      if (!webGPUSupported) {
        alert("WebGPU not supported");
      }

      (globalThis as any).define.amd = undefined;
      EngineFactory.CreateAsync(this.canvas.nativeElement, {}).then((engine) => {
        this.engine = engine as WebGPUEngine;
        this.scene = new MyFirstScene(this.engine);
        this.onCodeChanged(this.codeModel.value);
        this.ngZone?.runOutsideAngular(() => {
          this.engine?.runRenderLoop(() => {
            if (this.scene != undefined) {
              this.scene.frame++;
              if (this.engine)
                this.scene.time += this.engine.getDeltaTime();
            }
            this.scene?.render();
          });
        });
      });
    }
  }

  theme = 'hc-black';

  codeModel: CodeModel = {
    language: 'wgsl',
    uri: 'main.glsl',
    value: `vec4 mainImage() {
    vec3 pos = vec3(uv.x, 0.0, 2. * uv.y) * 3.14159265359;

    float x = sin(pos.x) * cos(pos.z);
    float y = sin(pos.x) * sin(pos.z);
    float z = cos(pos.x);

    float x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    float y2 = 8. * cos(pos.x);
    float z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    vec3 sphere = vec3(x, y, z) * 3.0;
    vec3 heart = vec3(x2, y2, z2) * 0.2;

    vec4 p = vec4(mix(sphere, heart, 0.5) * 1., 1.);
    return worldViewProjection * p;
}`
  };

  options = {
    contextmenu: true,
    minimap: {
      enabled: true
    }
  };

  onCodeChanged(value: string) {
//    console.log(value);
    this.engine?.releaseEffects();
    Effect.ShadersStore['customFragmentShader'] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;
    var code = this.assembleFullVertexShader(value);
    console.log(code);
    Effect.ShadersStore['customVertexShader'] = code;
    this.scene?.resetMaterials();
  }

  assembleFullVertexShader(innerCode: string) {
    var prefix =
      "precision highp float;\n" +
      "attribute vec3 position;\n" +
      "attribute vec2 uv;\n" +
      "uniform float iTime;\n" +
      "uniform float iTimeDelta;\n" +
      "uniform float iFrame;\n" +
      "uniform mat4 worldViewProjection;\n";
    var call = "" +
      "void main() {" +
      "gl_Position = mainImage();" +
      "}";
    return prefix + "\n" + innerCode + "\n" + call;
  }
}
