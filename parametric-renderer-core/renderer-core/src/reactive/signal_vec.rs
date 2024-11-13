use reactive_graph::{
    prelude::*,
    signal::{ArcReadSignal, ArcRwSignal},
};

const DISPOSED_MSG: &str = "Signal is already disposed";

/// A reactive vector that doesn't trigger reactivity too often.
pub struct SignalVec<T>
where
    T: 'static,
{
    inner: ArcRwSignal<Vec<ArcRwSignal<T>>>,
}

impl<T> Clone for SignalVec<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for SignalVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SignalVec<T> {
    pub fn new() -> Self {
        Self {
            inner: ArcRwSignal::new(Vec::new()),
        }
    }

    /// Returns the signal at the given index, if it exists.
    /// Also tracks the vector's length.
    pub fn get(&self, index: usize) -> Option<ArcReadSignal<T>> {
        if index < self.len() {
            Some(self.inner.with_untracked(|inner| inner[index].read_only()))
        } else {
            None
        }
    }

    fn get_unchecked(&self, index: usize) -> ArcReadSignal<T> {
        self.inner.with_untracked(|inner| inner[index].read_only())
    }

    fn get_unchecked_mut(&self, index: usize) -> ArcRwSignal<T> {
        self.inner.with_untracked(|inner| inner[index].clone())
    }

    pub fn push(&self, value: T) {
        self.inner.write().push(ArcRwSignal::new(value));
    }

    pub fn len(&self) -> usize {
        self.inner.with(|inner| inner.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn truncate(&self, len: usize) {
        self.inner.write().truncate(len);
    }

    pub fn clear(&self) {
        self.inner.write().clear();
    }

    pub fn pop(&self) -> Option<ArcRwSignal<T>> {
        self.inner.write().pop()
    }

    pub fn set(&self, index: usize, element: T) -> T {
        let guard = self.inner.write_untracked();
        guard[index]
            .try_update(|inner_value| std::mem::replace(inner_value, element))
            .expect(DISPOSED_MSG)
    }

    pub fn swap_remove(&self, index: usize) -> ArcRwSignal<T> {
        let mut guard = self.inner.write();
        let signal = guard.swap_remove(index);
        // Update the index of the swapped element
        guard.get(index).map(|v| v.notify());
        signal
    }

    pub fn iter(&self) -> SignalVecIter<T> {
        self.inner.track();
        SignalVecIter {
            vec: self.clone(),
            index: 0,
        }
    }

    pub fn iter_mut(&self) -> SignalVecIterMut<T> {
        self.inner.track();
        SignalVecIterMut {
            vec: self.clone(),
            index: 0,
        }
    }
}

pub struct SignalVecIter<T: 'static> {
    vec: SignalVec<T>,
    index: usize,
}

impl<T> Iterator for SignalVecIter<T> {
    type Item = ArcReadSignal<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.inner.with_untracked(|inner| inner.len()) {
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
    type Item = ArcRwSignal<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.inner.with_untracked(|inner| inner.len()) {
            let signal = self.vec.get_unchecked_mut(self.index);
            self.index += 1;
            Some(signal)
        } else {
            None
        }
    }
}
