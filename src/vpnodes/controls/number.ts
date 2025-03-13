import { ClassicPreset } from "rete";

export class NumberControl extends ClassicPreset.Control {
  constructor(
    public value?: number,
    public step?: number,
    public label?: string,
    public change?: () => void,
    public updateValue?: (value: number) => void
  ) {
    super();
    if (!this.value) this.value = 0;
    if (!this.step) this.step = 1;
    if (!this.label) this.label = "";
    if (!this.change)
      this.change = () => {
        return;
      };
    if (!this.updateValue)
      this.updateValue = (v) => {
        return;
      };
  }
}
