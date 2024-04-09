// TODO: Replace with native `DisposableStack` type when it becomes available
export class DisposableStack implements Disposable {
  private disposables: Disposable[] = [];

  addDisposable<
    T extends { dispose: () => void } | { [Symbol.dispose]: () => void }
  >(disposable: T): T {
    if (Symbol.dispose in disposable) {
      this.disposables.push(disposable);
    } else {
      this.disposables.push({
        [Symbol.dispose]: () => disposable.dispose(),
      });
    }
    return disposable;
  }
  [Symbol.dispose](): void {
    for (let disposable of this.disposables) {
      disposable[Symbol.dispose]();
    }
  }
}
