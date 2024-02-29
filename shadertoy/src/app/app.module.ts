import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { AppComponent } from './app.component';
import { CodeEditorComponent } from './code-editor/code-editor.component';
@NgModule({
  declarations: [AppComponent],
  imports: [BrowserModule, FormsModule, CodeEditorComponent],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule {}
