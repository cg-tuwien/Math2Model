import {AfterViewInit, Component, ElementRef, NgZone, ViewChild} from '@angular/core';
import {Engine, Effect} from "babylonjs";
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
    value: 'void main() {\n  gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);\n}\n'
  };

  options = {
    contextmenu: true,
    minimap: {
      enabled: true
    }
  };

  onCodeChanged(value: string) {
    console.log('CODE', value);
  }

  protected readonly Effect = Effect;
}
