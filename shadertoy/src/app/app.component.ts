import {
  AfterViewInit,
  Component,
  ElementRef,
  NgZone,
  ViewChild,
} from '@angular/core';
import { EngineFactory, Effect, Engine, WebGPUEngine } from '@babylonjs/core';
import { MyFirstScene } from './scenes/MyFirstScene';
import '@babylonjs/core/Engines/WebGPU/Extensions/';
import { CodeEditorComponent } from './code-editor/code-editor.component';

import * as BABYLON from '@babylonjs/core';
(window as any).BABYLON = BABYLON;

// shadows
import '@babylonjs/core/Lights/Shadows/shadowGeneratorSceneComponent';

// texture loading
import '@babylonjs/core/Materials/Textures/Loaders/envTextureLoader';
// needed for skybox textur'
import '@babylonjs/core/Misc/dds';
// edge'
import '@babylonjs/core/Rendering/edgesRenderer';
// gltf'loadin'
import '@babylonjs/loaders/glTF/2.0';
// anim'tion'
import '@babylonjs/core/Animations/animatable';
// import {WebGPUEngine} from "@babylonjs/core";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
})
export class AppComponent implements AfterViewInit {
  @ViewChild('canvas', { static: true })
  private canvas?: ElementRef<HTMLCanvasElement>;
  private engine?: WebGPUEngine;
  private scene?: MyFirstScene;

  code: string = '';

  //private ngZone: NgZone = new NgZone([]);
  constructor(private ngZone: NgZone) {}
  async ngAfterViewInit(): Promise<void> {
    if (this.canvas) {
      const webGPUSupported = await WebGPUEngine.IsSupportedAsync;
      if (!webGPUSupported) {
        alert('WebGPU not supported');
      }

      EngineFactory.CreateAsync(this.canvas.nativeElement, {}).then(
        (engine) => {
          this.engine = engine as WebGPUEngine;
          this.scene = new MyFirstScene(this.engine);
          this.onCodeChanged();
          this.ngZone?.runOutsideAngular(() => {
            this.engine?.runRenderLoop(() => {
              if (this.scene != undefined) {
                this.scene.frame++;
                if (this.engine) this.scene.time += this.engine.getDeltaTime();
              }
              this.scene?.render();
            });
          });
        }
      );
    }
  }

  theme = 'hc-black';

  onCodeChanged() {
    if (!this.engine) return;
    //    console.log(value);
    this.engine.releaseEffects();
    Effect.ShadersStore['customFragmentShader'] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;
    var code = this.assembleFullVertexShader(this.code);
    console.log(code);
    Effect.ShadersStore['customVertexShader'] = code;
    this.scene?.resetMaterials();
  }

  assembleFullVertexShader(innerCode: string) {
    var prefix =
      'precision highp float;\n' +
      'attribute vec3 position;\n' +
      'attribute vec2 uv;\n' +
      'uniform float iTime;\n' +
      'uniform float iTimeDelta;\n' +
      'uniform float iFrame;\n' +
      'uniform mat4 worldViewProjection;\n';
    var call = '' + 'void main() {' + 'gl_Position = mainImage();' + '}';
    return prefix + '\n' + innerCode + '\n' + call;
  }
}
