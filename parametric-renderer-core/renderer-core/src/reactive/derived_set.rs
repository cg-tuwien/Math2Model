use std::{collections::HashMap, sync::Arc};

use reactive_graph::{
    computed::ArcMemo,
    graph::untrack,
    owner::StoredValue,
    traits::{Read, Track, UpdateValue, WithValue},
    wrappers::read::ArcSignal,
};

pub trait GetDerived<Key, Value>: Send + Sync + 'static {
    fn get(&self, key: &Key) -> Option<Value>;
}

impl<Key, Value, T> GetDerived<Key, Value> for T
where
    T: Fn(&Key) -> Option<Value> + Send + Sync + 'static,
{
    fn get(&self, key: &Key) -> Option<Value> {
        self(key)
    }
}

pub struct DerivedMemo<Key, Derive: ?Sized, Value>
where
    Value: Send + Sync + 'static,
{
    derive: Arc<Derive>,
    map: StoredValue<HashMap<Key, (ArcSignal<Option<Arc<Value>>>, ArcMemo<Option<Arc<Value>>>)>>,
}
impl<Key, Derive, Value> DerivedMemo<Key, Derive, Value>
where
    Key: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    Derive: GetDerived<Key, Value>,
    Value: Send + Sync + 'static,
{
    pub fn new(derive: Derive) -> Self {
        Self {
            derive: Arc::new(derive),
            map: Default::default(),
        }
    }

    pub fn get_untracked(&self, key: &Key) -> Option<Arc<Value>> {
        untrack(|| self.get(key))
    }

    pub fn get(&self, key: &Key) -> Option<Arc<Value>> {
        if let Some((value_signal, value_memo)) = self.map.with_value(|map| map.get(key).cloned()) {
            let guard = value_memo.read();
            if let Some(value) = &*guard {
                // clone the Arc
                return Some(value.clone());
            } else {
                let _track = value_signal.track();
                // This works with a signal, because the ".read()" from above actually tracks the (derive)(key) function
                // With a memo, we would just be tracking the memo itself. We need to track the (derive)(key) function
                self.map.update_value(|map| {
                    map.remove(key);
                });
                return None;
            }
        } else {
            // the derive function tracks all dependencies. If it's smart, it liberally uses memos to avoid accidentally triggering the recomputation too early
            let value_signal = ArcSignal::derive({
                let derive = self.derive.clone();
                let key = key.clone();
                move || {
                    let value = derive.get(&key);
                    value.map(Arc::new)
                }
            });
            let value_memo = ArcMemo::new_with_compare(
                {
                    let value_signal = value_signal.clone();
                    move |_| {
                        let value: Option<_> = value_signal.read().clone();
                        value
                    }
                },
                |_, _| true,
            );
            self.map.update_value(|map| {
                map.insert(key.clone(), (value_signal, value_memo));
            });
            return self.get(key);
        }
    }
}

impl<Key, Derive, Value> Clone for DerivedMemo<Key, Derive, Value>
where
    Value: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            derive: self.derive.clone(),
            map: self.map.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use any_spawner::Executor;
    use reactive_graph::{
        effect::Effect,
        owner::Owner,
        signal::{ArcRwSignal, RwSignal},
        traits::{Get, Set},
    };

    use super::*;

    #[test]
    fn get_key() {
        let _ = Executor::init_futures_executor();
        let derived = DerivedMemo::new(|key: &i32| if *key == 1 { Some(1) } else { None });

        assert_eq!(derived.get_untracked(&1), Some(Arc::new(1)));
        assert_eq!(derived.get_untracked(&1), Some(Arc::new(1)));
        assert_eq!(derived.get_untracked(&2), None);
        assert_eq!(derived.get_untracked(&2), None);

        Executor::poll_local();
    }

    #[test]
    fn mutate_source() {
        let _ = Executor::init_futures_executor();
        let source = ArcRwSignal::new(2);
        let derived = DerivedMemo::new({
            let source = source.clone();
            move |_key: &i32| {
                // pretend we're accessing a database
                let value = source.get();
                if value == 1 {
                    // pretend that it can have a value or not
                    Some(1)
                } else {
                    None
                }
            }
        });

        assert_eq!(derived.get_untracked(&1), None);
        source.set(1);
        assert_eq!(derived.get_untracked(&1), Some(Arc::new(1)));

        Executor::poll_local();
    }

    #[test]
    fn mutate_source_unset_effect() {
        let _ = Executor::init_futures_executor();
        let _owner = Owner::new();
        let source = RwSignal::new(2);
        let derived = DerivedMemo::new({
            let source = source.clone();
            move |_key: &i32| {
                // pretend we're accessing a database
                let value = source.get();
                if value == 1 {
                    // pretend that it can have a value or not
                    Some(1)
                } else {
                    None
                }
            }
        });

        Effect::new({
            let derived = derived.clone();
            move |is_second_run: Option<()>| {
                let value = derived.get(&1);
                if is_second_run.is_none() {
                    assert_eq!(value, None);
                    ()
                } else {
                    assert_eq!(value, Some(Arc::new(1)));
                    ()
                }
            }
        });

        Effect::new(move |_| {
            source.set(1);
        });

        Executor::poll_local();
    }

    #[test]
    fn mutate_source_set_effect() {
        let _ = Executor::init_futures_executor();
        let _owner = Owner::new();
        let source = RwSignal::new(1);
        let derived = DerivedMemo::new({
            let source = source.clone();
            move |_key: &i32| {
                // pretend we're accessing a database
                let value = source.get();
                if value == 1 {
                    // pretend that it can have a value or not
                    Some(1)
                } else {
                    None
                }
            }
        });

        Effect::new({
            let derived = derived.clone();
            move |is_second_run: Option<()>| {
                let value = derived.get(&1);
                if is_second_run.is_none() {
                    assert_eq!(value, Some(Arc::new(1)));
                    ()
                } else {
                    assert_eq!(value, None);
                    ()
                }
            }
        });

        Effect::new(move |_| {
            source.set(2);
        });

        Executor::poll_local();
    }
}
