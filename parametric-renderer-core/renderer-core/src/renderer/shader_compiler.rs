use std::{borrow::Cow, cell::RefCell, collections::HashMap};

use wesl::{
    NoMangler, ResolveError, Resolver, StandardResolver, Wesl,
    syntax::{ModulePath, TranslationUnit},
};

pub struct ShaderCompiler {
    resolver: MyResolver,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            resolver: MyResolver {
                overlay: Default::default(),
                tu_cache: Default::default(),
                // // TODO: change this to "./shaders"
                resolver: StandardResolver::new("src/shaders"),
            },
        }
    }

    pub fn compile_shader(
        &mut self,
        module_path: ModulePath,
    ) -> Result<wesl::CompileResult, wesl::Error> {
        // // TODO: change this to "./shaders"
        let mut compiler = Wesl::new("src/shaders").set_custom_resolver(&self.resolver);
        compiler.set_custom_mangler(NoMangler); // Technically not needed here
        compiler.compile(module_path)
    }
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
