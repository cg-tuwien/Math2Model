import { test, expect } from "vitest";
import {
  ContentStore,
  undoContentAction,
  type ContentAction,
} from "./content-store";
import { ContentFile } from "./content-file";

test("undo(undo(v)) == v", () => {
  const action: ContentAction = {
    kind: "add",
    file: new ContentFile("foo.json", "fooo"),
    timestamp: 30,
  };

  expect(undoContentAction(undoContentAction(action))).toEqual(action);
});

test("list files", () => {
  const store = new ContentStore({});
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: fooFile,
    timestamp: 30,
  });
  store.runAction({
    kind: "add",
    file: new ContentFile("bar.json", "cute"),
    timestamp: 30,
  });
  store.runAction({
    kind: "remove",
    file: fooFile,
    timestamp: 30,
  });
  expect(store.getFiles()).toEqual([new ContentFile("bar.json", "cute")]);
});

test("illegal removal", () => {
  const store = new ContentStore({});
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: new ContentFile("bar.json", "cute"),
    timestamp: 30,
  });
  expect(() =>
    store.runAction({
      kind: "remove",
      file: fooFile,
      timestamp: 30,
    })
  ).toThrow();
});

test("undo-redo", () => {
  const store = new ContentStore({});
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: fooFile,
    timestamp: 30,
  });
  expect(store.getFiles()).toEqual([fooFile]);
  store.undo();
  expect(store.getFiles()).toEqual([]);
  store.redo();
  expect(store.getFiles()).toEqual([fooFile]);
});

test("undo-edit-redo", () => {
  const store = new ContentStore({});
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: fooFile,
    timestamp: 30,
  });
  expect(store.getFiles()).toEqual([fooFile]);
  store.undo();
  const barFile = new ContentFile("bar.json", "baro");
  store.runAction({
    kind: "add",
    file: barFile,
    timestamp: 30,
  });
  expect(store.getFiles()).toEqual([barFile]);
  store.redo(); // Nothing happens
  expect(store.getFiles()).toEqual([barFile]);
});

test("does not compress actions from different files", () => {
  const store = new ContentStore({
    actionLimits: {
      maxTimePerFile: 0,
    },
  });
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: fooFile,
    timestamp: 30,
  });
  store.runAction({
    kind: "add",
    file: new ContentFile("bar.json", "cute"),
    timestamp: 31,
  });
  expect(store.getFiles()).toEqual([
    new ContentFile("bar.json", "cute"),
    fooFile,
  ]);
  store.undo();
  // The undo actions were untouched
  expect(store.getFiles()).toEqual([fooFile]);
});

test("compress actions", () => {
  const store = new ContentStore({
    actionLimits: {
      maxTimePerFile: 0,
    },
  });
  const fooFile = new ContentFile("foo.json", "fooo");
  store.runAction({
    kind: "add",
    file: fooFile,
    timestamp: 30,
  });
  const newFooFile = new ContentFile("foo.json", "new file");
  store.runAction({
    kind: "replace",
    oldFile: fooFile,
    newFile: newFooFile,
    timestamp: 31,
  });
  expect(store.getFiles()).toEqual([newFooFile]);

  // The undo actions were compressed into a single one
  store.undo();
  expect(store.getFiles()).toEqual([]);

  // Does not do anything at all
  store.undo();
  expect(store.getFiles()).toEqual([]);

  // Redoes the entire thing
  store.redo();
  expect(store.getFiles()).toEqual([newFooFile]);
});
