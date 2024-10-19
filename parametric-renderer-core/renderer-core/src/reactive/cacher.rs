use std::{collections::HashMap, sync::Arc};

use reactive_graph::{
    owner::{on_cleanup, StoredValue},
    traits::UpdateValue,
};

/*Usage: let shader = Memo::new_computed({
    let model = model.clone();
    move |_| {
        let model = model.read();
        let shader_source = Memo::new(|| shader_sources.read().get(&model.shader_id));

        // Hard part: How do I clear the cache?
        // Answer: Whenever this function is called, it clears this cache entry.
        let entry = cacher.get(ShaderKey(model.shader_id.clone()));
        match entry {
            Occupied(compiled_shader) => compiled_shader,
            Vacant(missing_shader) => {
                let new_shader = compile(shader_source);
                missing_shader.insert(new_shader);
                on_cleanup(|| {
                    // This is called when the shader is no longer needed.
                    // It's a good place to clean up resources.
                    new_shader.delete();
                })
                new_shader
            }
        }
    }
}); */
pub struct Cacher<Keys> {
    map: StoredValue<HashMap<Keys, Arc<dyn std::any::Any + Send + Sync>>>,
}

impl<Keys> Default for Cacher<Keys>
where
    Keys: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Keys> Clone for Cacher<Keys> {
    fn clone(&self) -> Self {
        Self { map: self.map }
    }
}

impl<Keys> Copy for Cacher<Keys> {}

impl<Keys> Cacher<Keys>
where
    Keys: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            map: StoredValue::new(Default::default()),
        }
    }

    /// Remember to put this into a `Memo` or `Computed` to avoid recomputing the value.
    /// Usage example
    /// ```raw
    /// TODO: Add usage example
    ///
    /// ```
    pub fn get_or<Value: Send + Sync + 'static>(
        &self,
        key: Keys,
        make_value: impl Fn() -> Value + 'static,
    ) -> Arc<Value> {
        let value = self
            .map
            .try_update_value(|map| match map.get(&key) {
                Some(value) => value.clone().downcast::<Value>().expect("Downcast failed"),
                None => {
                    let value = Arc::new(make_value());
                    map.insert(key.clone(), value.clone());

                    value
                }
            })
            .expect("Failed to update map");

        on_cleanup({
            let map = self.map;
            move || {
                map.update_value(move |map| {
                    map.remove(&key);
                });
            }
        });
        value
    }
}
