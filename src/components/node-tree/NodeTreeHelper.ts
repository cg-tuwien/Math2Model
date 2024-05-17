export type SelectionGeneration = number & { __selectionGeneration: true };
export type NodeKey = string;
export type NodePath = NodeKey[];
export interface TreeNode {
  key: NodeKey;
  label: string;
  isVisible?: boolean;
  isExpanded?: boolean;
  isSelected?: [SelectionGeneration, boolean];
  children?: TreeNode[];
  disabled?: boolean;
}
export const NodeTreeHelper = {
  getNode(root: TreeNode, path: NodePath): TreeNode | null {
    if (path.length === 0) {
      return root;
    }

    const impl = (node: TreeNode, pathIndex: number): TreeNode | null => {
      if (pathIndex >= path.length) {
        return null;
      }
      if (pathIndex === path.length - 1) {
        if (node.key === path[pathIndex]) {
          return node;
        }
      }
      for (const child of node.children ?? []) {
        const result = impl(child, pathIndex + 1);
        if (result !== null) {
          return result;
        }
      }
      return null;
    };
    for (const child of root.children ?? []) {
      const result = impl(child, 0);
      if (result !== null) {
        return result;
      }
    }
    return null;
  },

  *visibleNodesIter(root: TreeNode): Generator<[TreeNode, NodePath]> {
    function* impl(
      node: TreeNode,
      path: NodePath
    ): Generator<[TreeNode, NodePath]> {
      yield [node, path];
      if (node.isExpanded === true) {
        for (const child of node.children ?? []) {
          yield* impl(child, path.concat([child.key]));
        }
      }
    }
    for (const child of root.children ?? []) {
      yield* impl(child, [child.key]);
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
