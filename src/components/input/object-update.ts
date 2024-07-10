export type ObjectPathPart = string | number;
export type NewValueFunction<Value = any> = (current: any) => Value;

// I am so tempted to add a type like
// type Path<T> = T extends Record<infer K extends string | number, any> ? K extends keyof T ? ([K, Path<T[K]>] | [K]) : never : never;
// to the ObjectUpdate class. But I think it's better to keep it simple.

export class ObjectUpdate<Value = any> {
  constructor(
    /**
     * Describes the path to the value that should be updated.
     */
    public readonly path: readonly ObjectPathPart[],
    /**
     * Takes the current value and returns the new value.
     */
    public readonly newValue: NewValueFunction<Value>,
    /**
     * A high speed update that will be followed by a final non-sliding update.
     */
    public readonly isSliding: boolean = false
  ) {}

  static sliding<Value = any>(
    path: readonly ObjectPathPart[],
    newValue: NewValueFunction<Value>
  ): ObjectUpdate {
    return new ObjectUpdate(path, newValue, true);
  }

  /**
   * Takes ownership of the object and applies the update to it.
   * Then returns the updated object.
   */
  applyTo<U>(obj: U): U {
    if (this.path.length === 0) {
      return this.newValue(obj) as any as U;
    } else {
      let current: any = obj;
      let last = this.path.length - 1;
      for (let i = 0; i < last; i++) {
        current = current[this.path[i]];
      }
      current[this.path[last]] = this.newValue(current[this.path[last]]);
      return obj;
    }
  }

  /**
   * Prepends the given path to the current path.
   */
  addPath(parentPath: ObjectPathPart | ObjectPathPart[]): ObjectUpdate {
    if (Array.isArray(parentPath)) {
      return new ObjectUpdate(
        [...parentPath, ...this.path],
        this.newValue,
        this.isSliding
      );
    } else {
      return new ObjectUpdate(
        [parentPath, ...this.path],
        this.newValue,
        this.isSliding
      );
    }
  }
}
