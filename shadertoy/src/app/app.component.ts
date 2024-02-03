import {AfterViewInit, Component, ElementRef, NgZone, ViewChild} from '@angular/core';
import {Engine, Effect} from "babylonjs";
import {MyFirstScene} from "./scenes/MyFirstScene";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})

export class AppComponent implements AfterViewInit {
  @ViewChild('canvas', {static: true}) private canvas?: ElementRef<HTMLCanvasElement>;
  private engine?: Engine;
  private scene?: MyFirstScene;
  editorOptions = {theme: 'vs-dark', language: 'javascript'};
  code: string = 'function x() {\nconsole.log("Hello world!");\n}';

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

  protected readonly Effect = Effect;
}
