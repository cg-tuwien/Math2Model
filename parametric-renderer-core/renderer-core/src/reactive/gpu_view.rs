// TODO: https://docs.rs/leptos/latest/leptos/trait.IntoView.html

use std::{ops::Range, sync::Arc};

use reactive_graph::{
    computed::{ArcMemo, Memo},
    owner::{ArcStoredValue, StoredValue, SyncStorage},
    signal::{
        ArcReadSignal, ArcRwSignal, ReadSignal, RwSignal,
        guards::{Plain, ReadGuard},
    },
    traits::{Read, ReadValue, With, WithValue},
    wrappers::read::{ArcSignal, Signal, SignalReadGuard},
};

// How do we deal with the lifetimes part?
// As in
// - RenderPass has a lifetime
// - Things that I add to a RenderPass also have a lifetime (like a texture view)
//   But those things can be hidden behind a ValueView
//   So we use with
//        entire "loop { texture.with(|value| commands.add(texture) ) }" block
pub trait IntoValueView<T: Send + Sync + 'static> {
    fn into_value_view(self) -> ValueView<T>;
}

pub enum ValueView<T: Send + Sync + 'static> {
    Raw(T),
    Signal(Signal<T>),
    ArcSignal(ArcSignal<T>),
    StoredValue(StoredValue<T>),
    ArcStored(ArcStoredValue<T>),
}

pub enum ValueGuard<'a, T: Send + Sync + 'static> {
    Raw(&'a T),
    Signal(ReadGuard<T, SignalReadGuard<T, SyncStorage>>),
    StoredValue(ReadGuard<T, Plain<T>>),
}

impl<T: Send + Sync + 'static> ValueView<T> {
    pub fn with<U>(&self, fun: impl FnOnce(&T) -> U) -> U {
        match self {
            ValueView::Raw(value) => fun(value),
            ValueView::Signal(signal) => signal.with(fun),
            ValueView::ArcSignal(signal) => signal.with(fun),
            ValueView::StoredValue(value) => value.with_value(fun),
            ValueView::ArcStored(value) => value.with_value(fun),
        }
    }
    pub fn read<'a>(&'a self) -> ValueGuard<'a, T> {
        match self {
            ValueView::Raw(value) => ValueGuard::Raw(value),
            ValueView::Signal(signal) => ValueGuard::Signal(signal.read()),
            ValueView::ArcSignal(signal) => ValueGuard::Signal(signal.read()),
            ValueView::StoredValue(value) => ValueGuard::StoredValue(value.read_value()),
            ValueView::ArcStored(value) => ValueGuard::StoredValue(value.read_value()),
        }
    }
}

macro_rules! impl_into_value_view {
    ($expr:expr, $($ty:ty),*) => {
        $(
            impl<T: Send + Sync + 'static> IntoValueView<T> for $ty {
                fn into_value_view(self) -> ValueView<T> {
                    $expr(self.into())
                }
            }
        )*
    };
}

impl_into_value_view!(
    ValueView::Signal,
    Signal<T>,
    ReadSignal<T>,
    Memo<T>,
    RwSignal<T>
);
impl_into_value_view!(
    ValueView::ArcSignal,
    ArcSignal<T>,
    ArcReadSignal<T>,
    ArcMemo<T>,
    ArcRwSignal<T>
);

impl<T: Send + Sync + 'static> IntoValueView<T> for T {
    fn into_value_view(self) -> ValueView<T> {
        ValueView::Raw(self)
    }
}
impl<T: Send + Sync + 'static + Clone> IntoValueView<T> for StoredValue<T> {
    fn into_value_view(self) -> ValueView<T> {
        ValueView::StoredValue(self)
    }
}
impl<T: Send + Sync + 'static + Clone> IntoValueView<T> for ArcStoredValue<T> {
    fn into_value_view(self) -> ValueView<T> {
        ValueView::ArcStored(self)
    }
}

pub fn render_pass<DepthStencil: IntoValueView<DepthStencilAttachment>>(
    color_attachments: impl IntoValueView<Vec<ColorAttachment>>,
    depth_stencil_attachment: Option<DepthStencil>,
) -> RenderPass {
    RenderPass {
        label: None,
        color_attachments: color_attachments.into_value_view(),
        depth_stencil_attachment: depth_stencil_attachment.map(IntoValueView::into_value_view),
        commands: ValueView::Raw(Vec::new()),
    }
}

#[must_use]
pub struct RenderPass {
    pub label: Option<ValueView<String>>,
    pub color_attachments: ValueView<Vec<ColorAttachment>>,
    pub depth_stencil_attachment: Option<ValueView<DepthStencilAttachment>>,
    // I don't need to use the For stuff here, because recreating all RenderCommands is very cheap.
    pub commands: ValueView<Vec<RenderCommand>>,
}

impl RenderPass {
    pub fn label(self, label: impl IntoValueView<String>) -> Self {
        Self {
            label: Some(label.into_value_view()),
            ..self
        }
    }
}

pub struct ColorAttachment {
    pub view: ValueView<wgpu::TextureView>,
    pub resolve_target: Option<ValueView<wgpu::TextureView>>,
    pub ops: wgpu::Operations<wgpu::Color>,
}

pub struct DepthStencilAttachment {
    pub view: ValueView<wgpu::TextureView>,
    pub depth_ops: Option<wgpu::Operations<f32>>,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
}

pub enum RenderCommand {
    Scope(String),
    SetPipeline(ValueView<wgpu::RenderPipeline>),
    /// Use this for "set_bind_groups"
    DoAction(ValueView<Arc<dyn Fn(&mut RenderPass) + Send + Sync>>),
    VertexBuffer {
        slot: u32,
        buffer: ValueView<wgpu::Buffer>,
        slice: Range<wgpu::BufferAddress>,
    },
    IndexBuffer {
        buffer: ValueView<wgpu::Buffer>,
        slice: Range<wgpu::BufferAddress>,
        format: wgpu::IndexFormat,
    },
    DrawIndexedIndirect {
        buffer: ValueView<wgpu::Buffer>,
        offset: wgpu::BufferAddress,
    },
}

/*
/*
pub struct ComputePass {
    pub label: Option<ValueView<String>>,
    pub descriptor: ComputePassDescriptor,
    pub commands: Vec<ComputeCommand>,
}

impl ComputePass {
    pub fn label(self, label: impl IntoValueView<String>) -> Self {
        Self {
            label: Some(label.into_value_view()),
            ..self
        }
    }
}

pub struct ComputePassDescriptor {
    pub bind_groups: Vec<BindGroup>,
}

     */

trait IntoGpuView {
    fn into_gpu_view(self) -> GpuView;
}

/// A list of commands that can be executed on the GPU.
/// This data structure is the target of IntoGpuView.
#[must_use]
pub enum GpuView {
    RenderPass { commands: Vec<RenderCommandView> },
    ComputePass { commands: Vec<ComputeCommandView> },
}

pub enum RenderCommandView {
    SetPipeline {
        pipeline: PipelineView,
    },
    SetBindGroup {
        index: u32,
        bind_group: BindGroupView,
    },
    SetVertexBuffer {
        index: u32,
        buffer: BufferView,
    },
    SetIndexBuffer {
        buffer: BufferView,
    },
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
}

pub enum ComputeCommandView {
    SetPipeline {
        pipeline: PipelineView,
    },
    SetBindGroup {
        index: u32,
        bind_group: BindGroupView,
    },
    Dispatch {
        x: u32,
        y: u32,
        z: u32,
    },
}
 */
