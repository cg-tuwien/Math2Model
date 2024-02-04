import {AfterViewInit, Component, ElementRef, NgZone, ViewChild} from '@angular/core';
import {Engine} from "babylonjs";
import {MyFirstScene} from "./scenes/MyFirstScene";
import {CodeModel} from "@ngstack/code-editor";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})

export class AppComponent implements AfterViewInit {
  @ViewChild('canvas', {static: true}) private canvas?: ElementRef<HTMLCanvasElement>;
  private engine?: Engine;
  private scene?: MyFirstScene;

  //private ngZone: NgZone = new NgZone([]);
  constructor(private ngZone: NgZone) {}
  ngAfterViewInit(): void {
    if (this.canvas) {
      this.engine = new Engine(this.canvas.nativeElement);
      this.scene = new MyFirstScene(this.engine);

      this.ngZone?.runOutsideAngular(() => {
        this.engine?.runRenderLoop(() => {
          this.scene?.render();
        });
      });
    }
  }

  theme = 'hc-black';

  codeModel: CodeModel = {
    language: 'wgsl',
    uri: 'main.glsl',
    value: `
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
`
  };

  options = {
    contextmenu: true,
    minimap: {
      enabled: true
    }
  };

  onCodeChanged(value: string) {
    console.log(value);
    this.engine?.releaseEffects();
    BABYLON.Effect.ShadersStore['customFragmentShader'] = `
        precision highp float;

        void main() {
            gl_FragColor = vec4(1.,0.,0.,1.);
        }
    `;
    BABYLON.Effect.ShadersStore['customVertexShader'] = value;
    this.scene?.resetMaterials();
  }
}
