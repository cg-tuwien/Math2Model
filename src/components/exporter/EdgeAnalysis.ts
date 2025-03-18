import type { VertexRange, Vertex } from "./VertexType";

export function analyzeEdges(patches: Vertex[][]): any {
  let rangesHorizontal: {
    top: Map<number, VertexRange[]>;
    bottom: Map<number, VertexRange[]>;
  } = {
    top: new Map(),
    bottom: new Map(),
  };
  rangesHorizontal.top.clear();
  rangesHorizontal.bottom.clear();

  let rangesVertical: {
    left: Map<number, VertexRange[]>;
    right: Map<number, VertexRange[]>;
  } = {
    left: new Map(),
    right: new Map(),
  };
  rangesVertical.right.clear();
  rangesVertical.left.clear();

  let ranges = { horizontal: rangesHorizontal, vertical: rangesVertical };
  let topRanges = rangesHorizontal.top;
  let botRanges = rangesHorizontal.bottom;
  let leftRanges = rangesVertical.left;
  let rightRanges = rangesVertical.right;

  let inpInd = 0;

  console.log("Fresh ranges",JSON.stringify(rangesHorizontal));
  // Loop through patches
  for (let patch of patches) {
    let [v1, v2, v3, v4] = patch;
    let upper = v2.uv.y;
    let lower = v1.uv.y;
    let left = v1.uv.x;
    let right = v3.uv.x;

    // Create vertex ranges
    let hedgeTop = createVertexRange(left, right, 1, 2, inpInd);
    let hedgeBottom = createVertexRange(left, right, 0, 3, inpInd);
    let vedgeLeft = createVertexRange(lower, upper, 0, 1, inpInd);
    let vedgeRight = createVertexRange(lower, upper, 3, 2, inpInd);

    inpInd++;

    // Insert ranges into horizontal and vertical maps efficiently
    insertRange(topRanges, upper, hedgeTop);
    insertRange(botRanges, lower, hedgeBottom);
    insertRange(leftRanges, left, vedgeLeft);
    insertRange(rightRanges, right, vedgeRight);
  }

  return ranges;
}

// Optimized insertion function
function insertRange(
  rangeMap: Map<number, VertexRange[]>,
  key: number,
  range: VertexRange
): void {
  let rangeList = getRangeList(rangeMap, key);
/*
  // Insert the new range in sorted order (using binary search)
  let low = 0;
  let high = rangeList.length;

  // Binary search to find the correct position
  while (low < high) {
    const mid = Math.floor((low + high) / 2);
    if (rangeList[mid].start < range.start) {
      low = mid + 1;
    } else {
      high = mid;
    }
  }*/
  rangeList.push(range);
//  rangeList.splice(low, 0, range);
  // Insert at the found position (we do not need to do a sort after this)
}

function getRangeList(rangeMap: Map<number, VertexRange[]>, key: number): any {
  if (rangeMap.has(key)) return rangeMap.get(key);
  let rangeList: any = [];
  rangeMap.set(key, rangeList);
  return rangeList;
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
