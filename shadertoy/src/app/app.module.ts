import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { AppComponent } from './app.component';
import {MonacoEditorModule, NgxMonacoEditorConfig} from 'ngx-monaco-editor-v2';

const monacoConfig: NgxMonacoEditorConfig = {
  baseUrl: 'app-name/assets', // configure base path for monaco editor. Starting with version 8.0.0 it defaults to './assets'. Previous releases default to '/assets'
  defaultOptions: { scrollBeyondLastLine: false }, // pass default options to be used
  onMonacoLoad: () => { console.log((<any>window).monaco); }, // here monaco object will be available as window.monaco use this function to extend monaco editor functionalities.
  requireConfig: { preferScriptTags: true }, // allows to oweride configuration passed to monacos loader
  monacoRequire: (<any>window).monacoRequire // pass here monacos require function if you loaded monacos loader (loader.js) yourself
};

@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule,
    FormsModule,
    MonacoEditorModule.forRoot(monacoConfig) // use forRoot() in main app module only.
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule {
}
