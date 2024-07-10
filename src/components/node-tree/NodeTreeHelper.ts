export type SelectionGeneration = number & { __selectionGeneration: true };
export type NodePath = readonly number[];
export interface TreeNode {
  key: string;
  label: string;
  isExpanded?: boolean;
  isSelected?: [SelectionGeneration, boolean];
  children?: TreeNode[];
  disabled?: boolean;
}

export const NodeTreeHelper = {
  getNode(root: TreeNode, path: Readonly<NodePath>): TreeNode | null {
    for (const pathIndex of path) {
      const child = root.children?.at(pathIndex) ?? null;
      if (child === null) {
        return null;
      }
      root = child;
    }
    return root;
  },

  *visibleNodesIter(root: TreeNode): Generator<[TreeNode, NodePath]> {
    function* impl(
      node: TreeNode,
      path: NodePath
    ): Generator<[TreeNode, NodePath]> {
      yield [node, path];
      if (node.isExpanded === true) {
        const children = node.children ?? [];
        for (let i = 0; i < children.length; i++) {
          const child = children[i];
          yield* impl(child, path.concat([i]));
        }
      }
    }
    const children = root.children ?? [];
    for (let i = 0; i < children.length; i++) {
      const child = children[i];
      yield* impl(child, [i]);
    }
  },

  isSelected(node: TreeNode, gen: SelectionGeneration): boolean {
    if (node.isSelected === undefined) {
      return false;
    }
    return node.isSelected[0] === gen && node.isSelected[1];
  },

  pathEquals(a: NodePath, b: NodePath): boolean {
    if (a.length !== b.length) {
      return false;
    }
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) {
        return false;
      }
    }
    return true;
  },

  filterNodes(
    node: TreeNode,
    pattern: (node: TreeNode) => boolean
  ): [TreeNode] | [] {
    const children = (node.children ?? []).flatMap((v) =>
      NodeTreeHelper.filterNodes(v, pattern)
    );
    const matches = pattern(node);
    if (children.length === 0 && matches === null) {
      return [];
    } else {
      return [
        {
          ...node,
          children,
        },
      ];
    }
  },
};
export interface TreeSelection {
  base: NodePath;
  generation: SelectionGeneration;
}
export function makeSelectionGeneration(
  generation: number
): SelectionGeneration {
  return generation as SelectionGeneration;
}
