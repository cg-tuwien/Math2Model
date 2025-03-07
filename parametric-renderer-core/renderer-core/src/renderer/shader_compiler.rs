use std::{borrow::Cow, cell::RefCell, collections::HashMap};

use wesl::{
    NoMangler, ResolveError, Resolver, StandardResolver,
    syntax::{ModulePath, TranslationUnit},
};

#[cfg(feature = "desktop")]
use notify_debouncer_full::{Debouncer, FileIdMap, notify::ReadDirectoryChangesWatcher};
use wgpu::ShaderModule;

// TODO: change this to "./shaders"
const SHADERS_PATH: &str = "src/shaders";

/// Compiles shaders with filesystem caching
/// This will probably become partially obsolete as wesl-rs improves
pub struct ShaderCompiler {
    resolver: MyResolver,
    #[cfg(feature = "desktop")]
    _file_watcher: Debouncer<ReadDirectoryChangesWatcher, FileIdMap>,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            resolver: MyResolver {
                overlay: Default::default(),
                tu_cache: Default::default(),
                resolver: StandardResolver::new(SHADERS_PATH),
            },
            #[cfg(feature = "desktop")]
            _file_watcher: make_file_watcher(SHADERS_PATH),
        }
    }

    pub fn compile_shader(
        &mut self,
        module_path: &ModulePath,
        overlay: HashMap<ModulePath, String>,
        device: &wgpu::Device,
    ) -> Result<ShaderModule, wesl::Diagnostic<wesl::Error>> {
        let overlay = overlay
            .into_iter()
            .map(|(key, value)| {
                let translation_unit = self.resolver.resolver.source_to_module(&value, &key)?;
                Ok((key, (value, translation_unit)))
            })
            .collect::<Result<HashMap<_, _>, wesl::Diagnostic<wesl::Error>>>()?;
        let old_overlay = std::mem::replace(&mut self.resolver.overlay, overlay);

        let mangler = NoMangler; // Technically not needed here
        let shader = wesl::compile(
            module_path,
            &self.resolver,
            &mangler,
            &wesl::CompileOptions {
                condcomp: false,
                ..Default::default()
            },
        )
        .map(|syntax| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(syntax.to_string())),
            })
        });

        self.resolver.overlay = old_overlay;
        Ok(shader?)
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "desktop")]
fn make_file_watcher(path: &str) -> Debouncer<ReadDirectoryChangesWatcher, FileIdMap> {
    use notify_debouncer_full::{DebounceEventResult, notify::RecursiveMode};
    use std::time::Duration;

    let mut file_watcher = notify_debouncer_full::new_debouncer(
        Duration::from_millis(1000),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                let any_modification = events
                    .into_iter()
                    .any(|e| e.kind.is_remove() || e.kind.is_modify());
                if any_modification {
                    // TODO: Clear the tu_cache in MyResolver
                }
            }
            Err(err) => log::error!("Error watching shaders: {:?}", err),
        },
    )
    .unwrap();
    file_watcher.watch(path, RecursiveMode::Recursive).unwrap();
    file_watcher
}

struct MyResolver {
    /// Lets me swap out files on the fly
    overlay: HashMap<ModulePath, (String, TranslationUnit)>,
    /// Super duper high perf caching
    tu_cache: RefCell<HashMap<ModulePath, TranslationUnit>>,
    /// Underlying resolver
    resolver: StandardResolver,
}

impl Resolver for MyResolver {
    fn resolve_source<'a>(
        &'a self,
        path: &ModulePath,
    ) -> Result<std::borrow::Cow<'a, str>, ResolveError> {
        // This is only ever called by the `source_to_module` and by the error message getting functions
        // So no point in doing cool caching
        if let Some((contents, _)) = self.overlay.get(path) {
            Ok(Cow::Borrowed(contents))
        } else {
            self.resolver.resolve_source(path)
        }
    }

    fn source_to_module(
        &self,
        source: &str,
        path: &ModulePath,
    ) -> Result<TranslationUnit, ResolveError> {
        // This is also just an internal helper. No caching here
        // This doesn't even do any FS access, it only calls display_name
        self.resolver.source_to_module(source, path)
    }

    fn resolve_module(&self, path: &ModulePath) -> Result<TranslationUnit, ResolveError> {
        if let Some((_, tu)) = self.overlay.get(path) {
            Ok(tu.clone())
        } else if let Some(tu) = self.tu_cache.borrow().get(path) {
            Ok(tu.clone())
        } else {
            let source = self.resolver.resolve_source(path)?;
            let wesl = self.resolver.source_to_module(&source, path)?;
            self.tu_cache
                .borrow_mut()
                .insert(path.clone(), wesl.clone());
            Ok(wesl)
        }
    }

    fn display_name(&self, path: &ModulePath) -> Option<String> {
        self.resolver.display_name(path)
    }
}
