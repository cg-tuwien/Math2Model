export type ContentType =
  | {
      kind: "image";
    }
  | {
      kind: "shader";
      subKind?: "graph-generated";
    }
  | {
      kind: "graph";
    }
  | {
      kind: "json";
      subKind?: "scene";
    }
  | {
      kind: "unknown";
    };

export class ContentFile {
  constructor(
    public readonly name: string,
    public readonly data: string | Readonly<Uint8Array>
  ) {}

  getType(): ContentType {
    const extension = getFileExtension(this.name);
    if (extension === null) {
      return {
        kind: "unknown",
      };
    } else if (extension === "wgsl") {
      if (this.name.endsWith(".graph.wgsl")) {
        return {
          kind: "shader",
          subKind: "graph-generated",
        };
      } else {
        return {
          kind: "shader",
        };
      }
    } else if (extension === "graph") {
      return {
        kind: "graph",
      };
    } else if (imageFileTypes.has(extension)) {
      return {
        kind: "image",
      };
    } else if (extension === "json") {
      if (this.name === "scene.json") {
        return {
          kind: "json",
          subKind: "scene",
        };
      } else {
        return {
          kind: "json",
        };
      }
    } else {
      return {
        kind: "unknown",
      };
    }
  }
}

/**
 *  filename CC BY SA-3.0 https://stackoverflow.com/a/190933/3492994
 */
export function getFileExtension(filename: string): string | null {
  var ext = /^.+\.([^.]+)$/.exec(filename);
  return ext == null ? null : ext[1];
}

const imageFileTypes = new Map<string, string>([
  ["png", "image/png"],
  ["avif", "image/avif"],
  ["bmp", "image/bmp"],
  ["gif", "image/gif"],
  ["jpg", "image/jpeg"],
  ["jpeg", "image/jpeg"],
  ["jpe", "image/jpeg"],
  ["jif", "image/jpeg"],
  ["jfif", "image/jpeg"],
  ["tif", "image/tiff"],
  ["tiff", "image/tiff"],
  ["webp", "image/webp"],
]);
