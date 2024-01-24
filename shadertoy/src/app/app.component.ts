import {AfterViewInit, Component, ElementRef, NgZone, ViewChild} from '@angular/core';
import { RouterOutlet } from '@angular/router';
import {Engine} from "babylonjs";
import {MyFirstScene} from "./scenes/MyFirstScene";

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})

export class AppComponent implements AfterViewInit {
  @ViewChild('canvas', {static: true}) private canvas?: ElementRef<HTMLCanvasElement>;
  private engine?: Engine;
  private scene?: MyFirstScene;
  private ngZone?: NgZone;

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
}
