import {
  AfterViewInit,
  Component,
  ElementRef,
  EventEmitter,
  Output,
  ViewChild,
} from '@angular/core';
import * as monaco from 'monaco-editor';
// @ts-ignore
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
// @ts-ignore
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
// @ts-ignore
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

@Component({
  selector: 'app-code-editor',
  standalone: true,
  imports: [],
  templateUrl: './code-editor.component.html',
  styleUrl: './code-editor.component.css',
})
export class CodeEditorComponent implements AfterViewInit {
  @ViewChild('monacoMount', { static: true })
  private monacoMount?: ElementRef<HTMLCanvasElement>;

  @Output() codeChanged = new EventEmitter<() => string>();

  codeModel = {
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
}`,
  };

  constructor() {}

  async ngAfterViewInit(): Promise<void> {
    if (self.MonacoEnvironment) {
      console.error(
        "Monaco environment shouldn't exist yet ",
        self.MonacoEnvironment
      );
    }
    self.MonacoEnvironment = {
      getWorker: function (_, label) {
        switch (label) {
          case 'json':
            return new jsonWorker();
          case 'typescript':
          case 'javascript':
            return new tsWorker();
          default:
            return new editorWorker();
        }
      },
    };

    // TODO: Resize observer

    let editor = monaco.editor.create(this.monacoMount!.nativeElement, {
      value: this.codeModel.value,
      language: this.codeModel.language,
      contextmenu: true,
      minimap: {
        enabled: false,
      },
    });

    // editor.layout()

    editor.onDidChangeModelContent((e) => {
      this.codeChanged.emit(() => editor.getValue());
    });
  }
}
