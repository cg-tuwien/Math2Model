import type { VertexRange, Vertex } from "./VertexType";

export function analyzeEdges(patches: Vertex[][]): any {
  const rangesHorizontal: {
    top: Record<number, VertexRange[]>;
    bottom: Record<number, VertexRange[]>;
  } = {
    top: {},
    bottom: {},
  };

  const rangesVertical: {
    left: Record<number, VertexRange[]>;
    right: Record<number, VertexRange[]>;
  } = {
    left: {},
    right: {},
  };

  const ranges = { horizontal: rangesHorizontal, vertical: rangesVertical };
  const topRanges = rangesHorizontal["top"];
  const botRanges = rangesHorizontal["bottom"];
  const leftRanges = rangesVertical["left"];
  const rightRanges = rangesVertical["right"];

  let inpInd = 0;

  for (const patch of patches) {
    const v1 = patch[0],
      v2 = patch[1],
      v3 = patch[2],
      v4 = patch[3];
    const upper = v2.uv.y;
    const lower = v1.uv.y;
    const left = v1.uv.x;
    const right = v3.uv.x;

    const hedgeTop = createVertexRange(left, right, 1, 2, inpInd);
    const hedgeBottom = createVertexRange(left, right, 0, 3, inpInd);
    const vedgeLeft = createVertexRange(lower, upper, 0, 1, inpInd);
    const vedgeRight = createVertexRange(lower, upper, 3, 2, inpInd);

    inpInd++;

    if (!topRanges[upper]) topRanges[upper] = [];
    insertSorted(topRanges[upper], hedgeTop);

    if (!botRanges[lower]) botRanges[lower] = [];
    insertSorted(botRanges[lower], hedgeBottom);

    if (!leftRanges[left]) leftRanges[left] = [];
    insertSorted(leftRanges[left], vedgeLeft);

    if (!rightRanges[right]) rightRanges[right] = [];
    insertSorted(rightRanges[right], vedgeRight);
  }
  for (const key in topRanges)
    topRanges[key].sort((a, b) => a.start - b.start);
  for (const key in botRanges)
    botRanges[key].sort((a, b) => a.start - b.start);
  for (const key in leftRanges)
    leftRanges[key].sort((a, b) => a.start - b.start);
  for (const key in rightRanges)
    rightRanges[key].sort((a, b) => a.start - b.start);

  return ranges;
}
function insertSorted(rangeList: VertexRange[], range: VertexRange): void {
  rangeList.push(range); // No sorting here
}

// Helper function to create a VertexRange object
function createVertexRange(
  start: number,
  end: number,
  startVert: number,
  endVert: number,
  ipi: number
): VertexRange {
  return { start, end, startVert, endVert, ipi };
}
