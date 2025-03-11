export class ExporterInstance {
  public vertPositions: { x: number; y: number; z: number }[] = [];
  public tris: number[] = [];
  private colors: string[] = [];
  public useUvs: boolean = false;
  public uvs: { x: number; y: number }[] = [];
  public normalsType: number = 0;
  public exportProgressVar  : any = {value: 0};
  public exportProgressStart: number = 0;
  public exportProgressToDo : number = 0;  

  private patches: any;
  private edges: any;
  private loopMeshX: boolean = false;
  private loopMeshY: boolean = false;
  private mapUv: boolean = false;

  constructor(
    patches: any,
    edges: any,
    loopMeshX = false,
    loopMeshY = false,
    mapUv = false
  ) {
    this.patches = patches;
    this.edges = edges;
    this.loopMeshX = loopMeshX;
    this.loopMeshY = loopMeshY;
    this.mapUv = mapUv;
  }

  public Run(): void {
    this.GenerateMesh();
  }

  private GenerateMesh(): void {
    // Clear arrays
    this.vertPositions = [];
    this.tris = [];
    this.uvs = [];
    let vertsMapping = new Map<string, number>();
    let vertexIndex = 0;
    this.exportProgressVar.value = this.exportProgressStart+this.exportProgressToDo*0.1;
    this.patches.forEach((patch: any[]) => {
      patch.forEach((vertex: any) => {
        let uv = vertex.uv;
        let uvString = uv.x + " " + uv.y;
        if (!vertsMapping.has(uvString)) {
          vertex.globalIndex = vertexIndex;
          vertsMapping.set(uvString, vertexIndex);
          vertexIndex++;
        } else {
          vertex.globalIndex = vertsMapping.get(uvString);
        }
      });
    });
    this.exportProgressVar.value = this.exportProgressStart+this.exportProgressToDo*0.3;
    // Create a boolean array using the vertsMapping count if available,
    // otherwise compute the total number of vertices across all patches.
    const vertCount = vertsMapping
      ? vertsMapping.size
      : this.patches.reduce(
          (acc: number, patch: any[]) => acc + patch.length,
          0
        );
    const bools: boolean[] = new Array(vertCount).fill(false);
    this.exportProgressVar.value = this.exportProgressStart+this.exportProgressToDo*0.5;

    // For each patch, add each vertex’s position and UV (only once per globalIndex)
    for (const patch of this.patches) {
      for (const vert of patch) {
        if (!bools[vert.globalIndex]) {
          this.vertPositions[vert.globalIndex] = {
            x: vert.vert.x,
            y: vert.vert.y,
            z: vert.vert.z,
          };
          this.uvs.push({ x: vert.uv.x, y: vert.uv.y });
          bools[vert.globalIndex] = true;
        }
      }
    }
    this.exportProgressVar.value = this.exportProgressStart+this.exportProgressToDo*0.7;

    let step = 0;
    // Process each patch (the C# version processes one patch at a time)
    for (const patch of this.patches) {
      step = this.processPatch(patch, step);
    }
    this.exportProgressVar.value = this.exportProgressStart+this.exportProgressToDo*0.9;
  }

  private processPatch(patch: any[], step: number): number {
    // Extract the four corner vertices
    const v0 = patch[0],
      v1 = patch[1],
      v2 = patch[2],
      v3 = patch[3];

    // Mark every vertex in the patch as a corner
    for (const v of patch) {
      v.corner = true;
    }

    // Set sides for the four corners
    v0.side = 3;
    v1.side = 2;
    v2.side = 1;
    v3.side = 0;

    // Determine boundary values from the UVs
    let lower = v0.uv.y;
    let upper = v1.uv.y;
    let left = v0.uv.x;
    let right = v2.uv.x;

    let verticesPatch: any[] = [];
    verticesPatch.push(v0);

    // --- Process the "right" edge ---
    let l = left;
    if (this.loopMeshX && l === 0.0) {
      l = 1;
    }
    const rightEdges = this.edges?.vertical?.["right"]?.[l.toString()];
    if (rightEdges) {
      for (const ledge of rightEdges) {
        if (ledge.end < lower) continue;
        if (ledge.end > upper) break;
        const v = this.patches[ledge.ipi][ledge.endVert];
        v.side = 0;
        verticesPatch.push(v);
      }
    }

    verticesPatch.push(v1);

    // --- Process the "bottom" edge ---
    let u = upper;
    if (this.loopMeshY && u === 1) {
      u = 0;
    }
    const bottomEdges = this.edges?.horizontal?.["bottom"]?.[u.toString()];
    if (bottomEdges) {
      for (const ledge of bottomEdges) {
        if (ledge.end <= left) continue;
        if (ledge.end >= right) break;
        const v = this.patches[ledge.ipi][ledge.endVert];
        v.side = 1;
        verticesPatch.push(v);
      }
    }

    verticesPatch.push(v2);

    // --- Process the "left" edge (in reverse order) ---
    let r = right;
    if (r === 1.0 && this.loopMeshX) {
      r = 0;
    }
    const leftEdges = this.edges?.vertical?.["left"]?.[r.toString()];
    if (leftEdges) {
      for (let i = leftEdges.length - 1; i >= 0; i--) {
        const ledge = leftEdges[i];
        if (ledge.end >= upper) continue;
        if (ledge.end <= lower) break;
        const v = this.patches[ledge.ipi][ledge.endVert];
        v.side = 2;
        verticesPatch.push(v);
      }
    }

    verticesPatch.push(v3);

    // --- Remove consecutive duplicate vertices (compare UV.y) ---
    for (let i = 0; i < verticesPatch.length - 1; i++) {
      if (
        verticesPatch[i].uv.y === verticesPatch[i + 1].uv.y &&
        verticesPatch[i].uv.x === verticesPatch[i + 1].uv.x
      ) {
        verticesPatch.splice(i + 1, 1);
        i--; // adjust index after removal
      }
    }

    // --- Process the "top" edge ---
    let lowAdj = lower;
    if (this.loopMeshY && lowAdj === 0.0) {
      lowAdj = 1;
    }
    const topEdges = this.edges?.horizontal?.["top"]?.[lowAdj.toString()];
    if (topEdges) {
      // Process in reverse order
      for (let i = topEdges.length - 1; i >= 0; i--) {
        const ledge = topEdges[i];
        if (ledge.end >= right) continue;
        if (ledge.end <= left) break;
        const v = this.patches[ledge.ipi][ledge.endVert];
        v.side = 3;
        verticesPatch.push(v);
      }
      step++; // increment step when top edge exists
    }

    // --- Triangulate the patch and add triangles to the mesh ---
    this.EarClipping(verticesPatch);

    return step;
  }

  private EarClipping(vp: any[]): void {
    let baseIndex = this.vertPositions.length;
    let indexMapping: number[] = new Array(vp.length).fill(-1);
    let j = 0;

    for (let i = 0; i < vp.length; i++) {
      const vert = vp[i];
      if (vert.globalIndex === -1) {
        vert.globalIndex = baseIndex + j++;
        this.vertPositions.push(
          this.mapUv
            ? {
                x: vert.uv.x,
                y: Math.sin(vert.uv.y * Math.PI * 1.9),
                z: Math.cos(vert.uv.y * Math.PI * 1.9),
              }
            : { x: vert.vert.x, y: vert.vert.y, z: vert.vert.z }
        );
        if (this.useUvs) this.uvs.push(vert.uv);
      }
      indexMapping[i] = vert.globalIndex;
    }

    let loopDetection = 0;
    let offset = 0;
    let i = 0;

    while (vp.length > 2) {
      i++;
      const v1 = vp[(i + offset) % vp.length];
      const v2 = vp[(i + 1 + offset) % vp.length];
      const v3 = vp[(i + 2 + offset) % vp.length];

      loopDetection++;
      if (vp.length === 2 && loopDetection > vp.length) {
        vp.length = 0; // Clear array
        break;
      }

      // Collinearity check (skip if all x or all y are the same)
      if (
        (v1.uv.x === v2.uv.x && v2.uv.x === v3.uv.x) ||
        (v1.uv.y === v2.uv.y && v2.uv.y === v3.uv.y)
      ) {
        if (loopDetection > 1000) {
          vp.length = 0; // Clear array to prevent infinite loops
          break;
        }
        continue;
      }

      vp.splice((i + 1 + offset) % vp.length, 1); // Remove ear vertex
      
      switch(this.normalsType)
      {
        case 1:
          this.tris.push(v1.globalIndex, v3.globalIndex, v2.globalIndex);
          break;
        case 0:
          this.tris.push(v1.globalIndex, v2.globalIndex, v3.globalIndex);
          break;
        case 2:
          this.tris.push(v1.globalIndex, v3.globalIndex, v2.globalIndex);
          this.tris.push(v1.globalIndex, v2.globalIndex, v3.globalIndex);
          break;
      }
      loopDetection = 0;

      i++;
    }
  }
}
