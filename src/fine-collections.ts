import {
  onScopeDispose,
  shallowRef,
  triggerRef,
  watch,
  type ShallowRef,
  type WatchOptions,
  type WatchStopHandle,
} from "vue";
import arrayUtils from "@stefnotch/typestef/array-utils";

export type MapChange<Key, Value> =
  | { type: "insert"; key: Key; value: Value }
  | { type: "remove"; key: Key }
  | { type: "update"; key: Key; value: Value };

/** A reactive map that only exposes fine grained reactivity info */
export class FineMap<Key, Value> {
  private changeTrackers: Array<ShallowRef<MapChange<Key, Value>[]>> = [];
  private map: Map<Key, Value>;

  /**
   * Adds a change to every tracker, and also drops changes that are no longer relevant.
   */
  private addChange(change: MapChange<Key, Value>): void {
    // O(n) but n is gonna be tiny
    this.changeTrackers.forEach((tracker) => {
      // Future perf: Drop changes that are no longer relevant
      tracker.value.push(change);
      triggerRef(tracker);
    });
  }

  constructor(entries?: readonly (readonly [Key, Value])[] | null) {
    this.map = new Map(entries);
  }

  watch(
    callback: (change: MapChange<Key, Value>) => void,
    options: WatchOptions<boolean> = {}
  ): WatchStopHandle {
    //Perf: I could technically deduplicate watchers based on the watch options. Not worth it though.
    const changes = shallowRef<MapChange<Key, Value>[]>([]);
    this.changeTrackers.push(changes);

    const stop = watch(
      changes,
      (v) => {
        for (const change of v) {
          callback(change);
        }
        v.length = 0; // Doesn't trigger the shallow ref
      },
      options
    );

    onScopeDispose(() => {
      arrayUtils.remove(this.changeTrackers, changes);
    });
    return () => {
      stop();
      arrayUtils.remove(this.changeTrackers, changes);
    };
  }

  clear(): void {
    const changes = Array.from(this.map.keys()).map(
      (key) => ({ type: "remove", key }) as const
    );
    this.map.clear();
    this.changeTrackers.forEach((tracker) => {
      // All previous changes are now irrelevant
      tracker.value = changes;
      triggerRef(tracker);
    });
  }
  delete(key: Key): boolean {
    const result = this.map.delete(key);
    if (result) {
      this.addChange({ type: "remove", key });
    }
    return result;
  }
  forEach(
    callbackfn: (value: Value, key: Key, map: Map<Key, Value>) => void,
    thisArg?: any
  ): void {
    this.map.forEach(callbackfn, thisArg);
  }
  get(key: Key): Value | undefined {
    return this.map.get(key);
  }
  has(key: Key): boolean {
    return this.map.has(key);
  }
  set(key: Key, value: Value): this {
    const isUpdate = this.map.has(key);
    this.map.set(key, value);
    this.addChange({ type: isUpdate ? "update" : "insert", key, value });
    return this;
  }
  get size(): number {
    return this.map.size;
  }
  /** Warning: Mutations during iteration will be ignored */
  entries(): IterableIterator<[Key, Value]> {
    return this.map.entries();
  }
  /** Warning: Mutations during iteration will be ignored */
  keys(): IterableIterator<Key> {
    return this.map.keys();
  }
  /** Warning: Mutations during iteration will be ignored */
  values(): IterableIterator<Value> {
    return this.map.values();
  }
  /** Warning: Mutations during iteration will be ignored */
  [Symbol.iterator](): IterableIterator<[Key, Value]> {
    return this.map[Symbol.iterator]();
  }
  get [Symbol.toStringTag](): string {
    return this.map[Symbol.toStringTag];
  }
}

export interface ReadonlyFineMap<Key, Value> extends ReadonlyMap<Key, Value> {}
