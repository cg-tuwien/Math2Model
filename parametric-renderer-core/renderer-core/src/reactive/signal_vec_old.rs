// Unused, because Signal<Option<T>> is tricky to use.
// I prefer being able to pass a Signal<T> to my components, and dispose those components when they are no longer needed.

use leptos_reactive::{batch, prelude::*, store_value, StoredValue};
/// A reactive vector that will only trigger the least amount of reactivity possible.
pub struct SignalVec<T>
where
    T: 'static,
{
    inner: StoredValue<SignalVecInner<T>>,
}

impl<T> Clone for SignalVec<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for SignalVec<T> {}

impl<T> Default for SignalVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct SignalVecInner<T: 'static> {
    underlying: Vec<RwSignal<Option<T>>>,
    len: RwSignal<usize>,
    none_signal: ReadSignal<Option<T>>,
}

impl<T> SignalVec<T> {
    pub fn new() -> Self {
        Self {
            inner: store_value(SignalVecInner {
                underlying: Vec::new(),
                len: create_rw_signal(0),
                none_signal: create_signal(None).0,
            }),
        }
    }

    /// Returns the signal at the given index, if it exists.
    ///
    /// Will read the length signal, and trigger reactivity iff our index becomes valid. (memoized)
    pub fn get(&self, index: usize) -> ReadSignal<Option<T>> {
        // This weird construction is so that we
        // 1. Fast case: Immediately return inner signal
        // 2. Slow case: Memoized wait until it's valid. Clear memo as soon as it's valid.
        if index < self.inner.with_value(|inner| inner.underlying.len()) {
            self.inner
                .with_value(|inner| inner.underlying[index].read_only())
        } else {
            let len_signal = self.len();
            let is_valid = create_memo(move |_| index < len_signal.get());
            assert!(
                is_valid.get() == false,
                "Should be false when we're tracking for the first time"
            );
            self.inner.with_value(|inner| inner.none_signal)
        }
    }

    fn get_unchecked(&self, index: usize) -> ReadSignal<Option<T>> {
        self.inner
            .with_value(|inner| inner.underlying[index].read_only())
    }

    fn get_unchecked_mut(&self, index: usize) -> RwSignal<Option<T>> {
        self.inner.with_value(|inner| inner.underlying[index])
    }

    pub fn push(&self, value: T) {
        batch(move || {
            self.inner.update_value(|inner| {
                inner.underlying.push(create_rw_signal(Some(value)));
                inner.len.set(inner.underlying.len());
            })
        })
    }

    pub fn len(&self) -> ReadSignal<usize> {
        self.inner.with_value(|inner| inner.len.read_only())
    }

    pub fn is_empty(&self) -> bool {
        self.len().get() == 0
    }

    pub fn truncate(&self, len: usize) {
        let to_dispose = batch(move || {
            self.inner
                .try_update_value(|inner| {
                    let items = inner.underlying.split_off(len);
                    for item in &items {
                        item.set(None);
                    }
                    inner.len.set(len);
                    items
                })
                .expect("SignalVec is already disposed")
        });
        for item in to_dispose {
            item.dispose();
        }
    }

    pub fn clear(&self) {
        self.truncate(0);
    }

    pub fn pop(&self) -> Option<T> {
        if let Some((value, to_dispose)) = batch(move || {
            self.inner
                .try_update_value(|inner| match inner.underlying.pop() {
                    Some(write_signal) => {
                        let value = write_signal
                            .try_update(|signal_value| std::mem::take(signal_value))
                            .expect("Inner signal is already disposed");
                        inner.len.set(inner.underlying.len());
                        value.map(|v| (v, write_signal))
                    }
                    None => None,
                })
                .expect("SignalVec is already disposed")
        }) {
            to_dispose.dispose();
            Some(value)
        } else {
            None
        }
    }

    pub fn set(&self, index: usize, element: T) {
        batch(move || {
            self.inner.update_value(|inner| {
                inner.underlying[index].set(Some(element));
            })
        })
    }

    pub fn swap_remove(&self, index: usize) -> T {
        let (value, to_dispose) = batch(move || {
            self.inner
                .try_update_value(|inner| {
                    let write_signal = inner.underlying.swap_remove(index);
                    // Update the index of the swapped element
                    inner.underlying.get(index).map(|v| v.update(|_| ()));

                    let value = write_signal
                        .try_update(|signal_value| std::mem::take(signal_value))
                        .expect("Inner signal is already disposed");
                    inner.len.set(inner.underlying.len());
                    (value.expect("Swapped element is None"), write_signal)
                })
                .expect("SignalVec is already disposed")
        });
        to_dispose.dispose();
        value
    }

    pub fn iter(&self) -> SignalVecIter<T> {
        self.len().track();
        SignalVecIter {
            vec: *self,
            index: 0,
        }
    }

    pub fn iter_mut(&self) -> SignalVecIterMut<T> {
        self.len().track();
        SignalVecIterMut {
            vec: *self,
            index: 0,
        }
    }
}

pub struct SignalVecIter<T: 'static> {
    vec: SignalVec<T>,
    index: usize,
}

impl<T> Iterator for SignalVecIter<T> {
    type Item = ReadSignal<Option<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.inner.with_value(|inner| inner.underlying.len()) {
            let signal = self.vec.get_unchecked(self.index);
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}

pub struct SignalVecIterMut<T: 'static> {
    vec: SignalVec<T>,
    index: usize,
}

impl<T> Iterator for SignalVecIterMut<T> {
    type Item = RwSignal<Option<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.inner.with_value(|inner| inner.underlying.len()) {
            let signal = self.vec.get_unchecked_mut(self.index);
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}
