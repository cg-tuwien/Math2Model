use indexmap::{map::Entry, IndexMap};
use leptos_reactive::{create_render_effect, Effect, Signal, SignalWith};

/// Inspired by Leptos's <For> component.
pub struct ForEach<Key, Output>
where
    Key: 'static,
    Output: 'static,
{
    effect: Effect<IndexMap<Key, Output>>,
}

impl<Key, Output> ForEach<Key, Output>
where
    Key: Eq + std::hash::Hash + std::fmt::Debug + 'static,
    Output: 'static,
{
    pub fn new<Collection, T, KeyFn, OutputFn>(
        collection: Signal<Collection>,
        key_fn: KeyFn,
        output_fn: OutputFn,
    ) -> Self
    where
        for<'a> &'a Collection: IntoIterator<Item = &'a T>,
        KeyFn: Fn(&T) -> Key + 'static,
        OutputFn: Fn(&T) -> Output + 'static,
    {
        let effect = create_render_effect(move |old_cache: Option<IndexMap<Key, Output>>| {
            let mut old_cache = old_cache.unwrap_or_default();
            let mut new_cache = IndexMap::new();
            collection.with(|collection| {
                for item in collection.into_iter() {
                    let key = (key_fn)(item);
                    let new_item = match old_cache.swap_remove(&key) {
                        Some(v) => v,
                        None => (output_fn)(item),
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
            });
            new_cache
        });

        Self { effect }
    }

    pub fn for_each(&self, callback: impl Fn(&Output)) {
        self.effect.with_value_mut(|last_run| {
            let last_run = last_run
                .as_ref()
                .expect("Render effect should have been run");
            for item in last_run.values() {
                callback(item);
            }
        });
    }
}
