import { ClassicPreset } from "rete";

export class SliderControl extends ClassicPreset.Control {
  public value: number;
  public hotValue: boolean = false;
  constructor(
    value?: number,
    public max?: number,
    public min?: number,
    public step?: number,
    public label?: string,
    public showInput?: boolean,
    public change?: () => void,
    public updateValue?: (value: number) => void,
    public noDebounceChange?: () => void
  ) {
    super();
    this.value = value ?? 0;
    if (!this.max) this.max = 10;
    if (!this.min && this.min != 0) this.min = -10;
    if (!this.step) this.step = 1;
    if (!this.label) this.label = "";
    if (!this.showInput) this.showInput = false;
    if (!this.change)
      this.change = () => {
        return;
      };
    if (!this.updateValue)
      this.updateValue = (v) => {
        return;
      };
    if (!this.noDebounceChange)
      this.noDebounceChange = () => {
        return;
      };
  }

  get sliderValue(): string {
    return this.hotValue ? "extra.hot_value" : "" + this.value;
  }
}
