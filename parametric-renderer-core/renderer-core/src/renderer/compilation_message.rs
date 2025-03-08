use std::ops::Range;

use wesl::ModulePath;

#[derive(Debug, Clone)]
pub struct WeslCompilationMessage {
    pub message: String,
    pub message_type: wgpu::CompilationMessageType,
    pub module: wesl::ModulePath,
    pub location: Option<Range<usize>>,
}

impl WeslCompilationMessage {
    pub fn from_wesl_error(value: wesl::Diagnostic<wesl::Error>, module: &ModulePath) -> Self {
        Self {
            message: value.error.to_string(),
            message_type: wgpu::CompilationMessageType::Error,
            // TODO: Why is the module_path of the error optional in wesl-rs?
            module: value.module_path.unwrap_or(module.clone()),
            location: value.span.map(|span| span.start..span.end),
        }
    }

    pub fn from_compilation_message(value: wgpu::CompilationMessage, module: ModulePath) -> Self {
        Self {
            message: value.message,
            message_type: value.message_type,
            module,
            location: value.location.map(|location| {
                (location.offset as usize)..((location.offset + location.length) as usize)
            }),
        }
    }
}
