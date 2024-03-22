import { ref, computed, type Ref, customRef } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import mapUtils from "@stefnotch/typestef/map-utils";

/**
 * Utility function to create a hot cache. Call it at the top of your file.
 * @param key Use `import.meta.url` as the key
 */
export function makeHotCache<
  Data extends {
    [K in string]: any;
  }
>(urlKey: string) {
  let _hotCache: HmrCache<Data> | null = null;
  const url = new URL(urlKey);
  url.hash = "";
  url.search = "";

  return (): HmrCache<Data> => {
    if (_hotCache === null) {
      _hotCache = useHotCacheStore().openCache(url + "");
    }
    return _hotCache;
  };
}

type GetKeys<Data> = Data extends { [K in keyof Data]: any }
  ? keyof Data
  : never;

/**
 * A single "type-safe" cache.
 *
 * Type-safety is trivially broken when you change the type of the HmrCache.
 */
export class HmrCache<
  Data extends {
    [K in string]: any;
  }
> {
  private map: Map<string, any> = new Map();
  private lastMap: ReadonlyMap<string, any> = new Map();
  private uuid = crypto.randomUUID();
  constructor() {}

  getOrInsert<Key extends GetKeys<Data>, Default extends Data[Key] | null>(
    key: Key,
    defaultValue: () => Default
  ): Data[Key] | Default {
    return mapUtils.getOrInsert(this.map, key, () => {
      if (this.lastMap.has(key)) {
        return this.lastMap.get(key);
      } else {
        return defaultValue();
      }
    }) as any as Default;
  }

  get<Key extends GetKeys<Data>>(key: Key): Data[Key] | null {
    return this.getOrInsert(key, () => null);
  }

  set<Key extends GetKeys<Data>>(key: Key, value: Data[Key] | null) {
    this.map.set(key, value);
  }

  ref<Key extends GetKeys<Data>, Default extends Data[Key] | null = null>(
    key: Key,
    defaultValue?: () => Default
  ): Ref<Data[Key] | Default> {
    const self = this;
    const selfUuid = this.uuid;
    return customRef<Data[Key] | Default>((track, trigger) => {
      return {
        get() {
          if (self.uuid !== selfUuid) {
            console.warn(
              "Cache was reopened, but the ref is old. This can lead to data corruption."
            );
          }
          track();

          if (defaultValue === undefined) {
            return self.getOrInsert<Key, Default>(key, () => null as Default);
          } else {
            return self.getOrInsert<Key, Default>(key, defaultValue);
          }
        },
        set(newValue: Data[Key] | Default) {
          if (self.uuid !== selfUuid) {
            console.warn(
              "Cache was reopened, but the ref is old. This can lead to data corruption."
            );
          }
          self.set(key, newValue);
          trigger();
        },
      };
    });
  }

  reopen() {
    this.lastMap = this.map;
    this.map = new Map();
    this.uuid = crypto.randomUUID();
  }
}

/**
 * This store survives HMR updates.
 *
 * Which means that we can store data that we want to keep between updates, like the camera's position
 */
export const useHotCacheStore = defineStore("hot-cache", () => {
  /**
   * It has a bunch of caches, one for each key.
   *
   * So we can have one cache per file for example.
   */
  const caches = new Map<string, HmrCache<any>>();

  /**
   * Make sure to only call this *once* per cache.
   * It keeps track of used keys and will clean up old keys when they are no longer used.
   */
  function openCache<
    Data extends {
      [K in string]: any;
    }
  >(key: string): HmrCache<Data> {
    const map = mapUtils.getOrInsert(caches, key, () => new HmrCache());
    map.reopen();
    return map;
  }

  return { openCache };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useHotCacheStore, import.meta.hot));
}
