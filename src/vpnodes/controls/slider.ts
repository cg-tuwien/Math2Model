import { ClassicPreset } from "rete";

export class SliderControl extends ClassicPreset.Control {
  constructor(
    public value?: number,
    public max?: number,
    public min?: number,
    public step?: number,
    public label?: string,
    public change?: (value: number) => void,
    public updateValue?: (value: number) => void,
  ) {
    super();
    if (!this.value) this.value = 0;
    if (!this.max) this.max = 10;
    if (!this.min && this.min != 0) this.min = -10;
    if (!this.step) this.step = 1;
    if (!this.label) this.label = "";
    if (!this.change)
      this.change = (v) => {
        return;
      };
    if (!this.updateValue)
      this.updateValue = (v) => {
        return;
      };
  }
}
