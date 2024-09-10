import { ClassicPreset } from "rete";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";

export class DropdownControl extends ClassicPreset.Control {
  constructor(
    public options: SelectMixedOption[],
    public selected?: string,
    public placeholder?: string,
    public label?: string,
    public change?: (select: string) => void,
  ) {
    super();
    if (!this.selected || this.selected === "")
      this.selected = (this.options[0].value as string) ?? "";
    if (!this.placeholder) this.placeholder = "Please select one";
    if (!this.label) this.label = "";
    if (!this.change)
      this.change = (s) => {
        return;
      };
  }

  setOptions(options: SelectMixedOption[]) {
    this.options = options;
  }
}
