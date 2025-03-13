import { assert, assertUnreachable } from "@stefnotch/typestef/assert";
import arrayUtils from "@stefnotch/typestef/array-utils";
import type { ContentFile } from "./content-file";

export interface ContentActionLimits {
  /** How many actions do we track.
   * The initial add actions are not counted.
   * Drops the oldest actions.
   */
  max?: number;
  /** How closely spaced together can actions be. */
  maxTimePerFile?: DurationMs;
}

/**
 * An in-memory content store that periodically syncs itself with the filesystem.
 * Comes with undo-redo capabilities.
 */
export class ContentStore {
  /** A sorted list of files (sorted by name) */
  private files: ContentFile[] = [];
  /** Actions that were applied. */
  private actions: CompressedActionsList;
  /** Can be directly applied in the redo function. */
  private redoActions: ContentAction[] = [];

  public constructor(opts: { actionLimits?: ContentActionLimits }) {
    this.actions = new CompressedActionsList(opts.actionLimits ?? {});
  }

  /**
   * Returns a view of the content files.
   * Do note that the underlying data can change.
   */
  getFiles(): ReadonlyArray<ContentFile> {
    return this.files;
  }

  runAction(action: ContentAction) {
    this.actions.push(action, this.files.length);
    this.redoActions.length = 0;
    // LATER enforce that file name are unique
    const needsFileSorting = this.runActionInternal(action);
    if (needsFileSorting) {
      this.sortFiles();
    }
  }

  undo() {
    const lastAction = this.actions.pop();
    if (lastAction === null) return;

    const undoLastAction = undoContentAction(lastAction);
    this.redoActions.push(lastAction);
    const needsFileSorting = this.runActionInternal(undoLastAction);
    if (needsFileSorting) {
      this.sortFiles();
    }
  }

  redo() {
    const redoLastAction = this.redoActions.pop();
    if (redoLastAction === undefined) return;
    this.actions.push(redoLastAction, this.files.length);
    const needsFileSorting = this.runActionInternal(redoLastAction);
    if (needsFileSorting) {
      this.sortFiles();
    }
  }

  /** Runs an action and returns whether the array needs to be re-sorted or not. */
  private runActionInternal(action: ContentAction): boolean {
    if (action.kind === "add") {
      this.files.push(action.file);
      return true;
    } else if (action.kind === "remove") {
      const fileIndex = this.findFileIndex(action.file);
      assert(fileIndex !== null);
      this.files.splice(fileIndex, 1);
      return false;
    } else if (action.kind === "replace") {
      const fileIndex = this.findFileIndex(action.oldFile);
      assert(fileIndex !== null);
      this.files[fileIndex] = action.newFile;
      return true;
    } else {
      assertUnreachable(action);
    }
  }

  private findFileIndex(file: ContentFile): number | null {
    const index = this.files.indexOf(file);
    return index !== -1 ? index : null;
  }

  private sortFiles() {
    this.files.sort((a, b) => a.name.localeCompare(b.name));
  }
}

class CompressedActionsList {
  /** Actions that were applied. */
  private actions: ContentAction[] = [];
  constructor(public limits: ContentActionLimits) {}

  /**
   * Compresses adjacent replace actions on the same file.
   *
   * Could be extended to peek further, and to follow renames, and to deal with combined actions.
   */
  push(action: ContentAction, filesCount: number) {
    if (this.actions.length === 0) {
      this.actions.push(action);
      return;
    }

    const joined = this.tryJoin(action, this.actions[this.actions.length - 1]);
    if (joined === null) {
      this.actions.push(action);
    } else {
      this.actions[this.actions.length - 1] = joined;
    }

    const maxActions = this.limits.max;

    // Good approximation of "number of actions minus add actions per file"
    const currentActions = this.actions.length - filesCount;
  }

  tryJoin(
    newAction: ContentAction,
    previousAction: ContentAction
  ): ContentAction | null {
    const maxTime = this.limits.maxTimePerFile;
    if (maxTime === undefined) {
      return null;
    }
    if (newAction.timestamp - previousAction.timestamp <= maxTime) {
      return null;
    }
    if (newAction.kind === "replace" && previousAction.kind === "add") {
      // Must merge
      return {
        kind: "add",
        file: newAction.newFile,
        timestamp: newAction.timestamp,
      };
    } else if (
      newAction.kind === "replace" &&
      previousAction.kind === "replace"
    ) {
      return {
        kind: "replace",
        newFile: newAction.newFile,
        oldFile: previousAction.oldFile,
        timestamp: newAction.timestamp,
      };
    } else {
      return null;
    }
  }

  pop(): ContentAction | null {
    return this.actions.pop() ?? null;
  }
}

/** A timestamp in milliseconds */
type TimestampMs = number;
/** A duration in milliseconds */
type DurationMs = number;

export type ContentAction =
  | {
      kind: "add";
      file: ContentFile;
      timestamp: TimestampMs;
    }
  | {
      kind: "remove";
      /** We store the entire file so that this action can be undone */
      file: ContentFile;
      timestamp: TimestampMs;
    }
  | {
      kind: "replace";
      oldFile: ContentFile;
      newFile: ContentFile;
      timestamp: TimestampMs;
    };

/** Returns the undo variant of a content action */
export function undoContentAction(action: ContentAction): ContentAction {
  if (action.kind === "add") {
    return {
      kind: "remove",
      file: action.file,
      timestamp: action.timestamp,
    };
  } else if (action.kind === "remove") {
    return {
      kind: "add",
      file: action.file,
      timestamp: action.timestamp,
    };
  } else if (action.kind === "replace") {
    return {
      kind: "replace",
      oldFile: action.newFile,
      newFile: action.oldFile,
      timestamp: action.timestamp,
    };
  } else {
    assertUnreachable(action);
  }
}
