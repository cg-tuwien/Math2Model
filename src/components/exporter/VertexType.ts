// Define a type for a single vertex
export interface Vertex {
  vert: { x: number; y: number; z: number };
  uv: { x: number; y: number };
  corner: boolean;
  globalIndex: number;
  tempIdx: number;
  side: number;
}

// Define a type for a single range
export interface VertexRange {
  start: number;
  end: number;
  startVert: number;
  endVert: number;
  ipi: number;
}
