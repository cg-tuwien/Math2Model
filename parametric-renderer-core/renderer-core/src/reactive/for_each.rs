use indexmap::{IndexMap, map::Entry};
use reactive_graph::{computed::ArcMemo, effect::RenderEffect, owner::Owner};

/// Lazily computes a "iterable" of items.
pub fn new_computed_vec<ItemsFn, Items, T, KeyFn, Key, OutputFn, Output>(
    items_fn: ItemsFn,
    key_fn: KeyFn,
    output_fn: OutputFn,
) -> ArcMemo<IndexMap<Key, (Output, Owner)>>
where
    ItemsFn: Fn() -> Items + Send + Sync + 'static,
    Items: IntoIterator<Item = T>,
    T: 'static,
    KeyFn: Fn(&T) -> Key + Send + Sync + 'static,
    Key: Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
    OutputFn: Fn(T) -> Output + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    use indexmap::{IndexMap, map::Entry};
    let parent = Owner::current().expect("no reactive owner");
    ArcMemo::new_owning(move |old_cache: Option<IndexMap<Key, (Output, Owner)>>| {
        let mut old_cache = old_cache.unwrap_or_default();
        let mut new_cache = IndexMap::new();
        let collection = (items_fn)();
        let mut found_all_cached = true;
        for item in collection.into_iter() {
            let key = (key_fn)(&item);
            let new_item = match old_cache.swap_remove(&key) {
                Some(v) => v,
                None => {
                    found_all_cached = false;
                    let owner = parent.with(Owner::new);
                    (owner.with(|| (output_fn)(item)), owner)
                }
            };
            match new_cache.entry(key) {
                Entry::Occupied(occupied) => {
                    panic!(
                        "Keys must be unique! Expected empty, but found entry at {:#?}",
                        occupied.key()
                    )
                }
                Entry::Vacant(vacant) => vacant.insert(new_item),
            };
        }
        std::mem::drop(old_cache);
        let has_changed = !found_all_cached;
        (new_cache, has_changed)
    })
}

/// Inspired by Leptos's `<For>` component.
/// Will forcibly compute everything, even if nobody depends on it.
pub struct ForEach<Key, Output>
where
    Key: 'static,
    Output: 'static,
{
    effect: RenderEffect<IndexMap<Key, (Output, Owner)>>,
}

impl<Key, Output> ForEach<Key, Output>
where
    Key: Eq + std::hash::Hash + std::fmt::Debug + 'static,
    Output: 'static,
{
    pub fn new<ItemsFn, Items, T, KeyFn, OutputFn>(
        items_fn: ItemsFn,
        key_fn: KeyFn,
        output_fn: OutputFn,
    ) -> Self
    where
        ItemsFn: Fn() -> Items + 'static,
        Items: IntoIterator<Item = T>,
        T: 'static,
        KeyFn: Fn(&T) -> Key + 'static,
        OutputFn: Fn(T) -> Output + 'static,
    {
        let parent = Owner::current().expect("no reactive owner");
        let effect = RenderEffect::new(move |old_cache: Option<IndexMap<Key, (Output, Owner)>>| {
            let mut old_cache = old_cache.unwrap_or_default();
            let mut new_cache = IndexMap::new();
            let collection = (items_fn)();
            for item in collection.into_iter() {
                let key = (key_fn)(&item);
                let new_item = match old_cache.swap_remove(&key) {
                    Some(v) => v,
                    None => {
                        let owner = parent.with(Owner::new);
                        (owner.with(|| (output_fn)(item)), owner)
                    }
                };
                match new_cache.entry(key) {
                    Entry::Occupied(occupied) => {
                        panic!(
                            "Keys must be unique! Expected empty, but found entry at {:#?}",
                            occupied.key()
                        )
                    }
                    Entry::Vacant(vacant) => vacant.insert(new_item),
                };
            }
            std::mem::drop(old_cache);
            new_cache
        });

        Self { effect }
    }

    pub fn for_each(&self, mut callback: impl FnMut(&Output)) {
        self.effect.with_value_mut(|last_run| {
            for (item, _) in last_run.values() {
                callback(item);
            }
        });
    }
}
