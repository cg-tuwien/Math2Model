use std::marker::PhantomData;
use wgpu::util::DeviceExt;

pub struct TypedBuffer<T>
where
    T: ?Sized,
{
    buffer: wgpu::Buffer,
    variant: TypedBufferVariant,
    _phantom: PhantomData<T>,
}

#[derive(Copy, Clone)]
enum TypedBufferVariant {
    Uniform,
    Storage,
}

impl<T> TypedBuffer<T>
where
    T: ?Sized,
{
    pub fn new_uniform(
        device: &wgpu::Device,
        label: &str,
        data: &T,
        usage: wgpu::BufferUsages,
    ) -> Self
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let usage = usage | wgpu::BufferUsages::UNIFORM;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &write_uniform_buffer(data),
            usage,
        });

        Self {
            buffer,
            variant: TypedBufferVariant::Uniform,
            _phantom: PhantomData,
        }
    }

    pub fn new_storage(
        device: &wgpu::Device,
        label: &str,
        data: &T,
        usage: wgpu::BufferUsages,
    ) -> Self
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let usage = usage | wgpu::BufferUsages::STORAGE;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &write_storage_buffer(data),
            usage,
        });

        Self {
            buffer,
            variant: TypedBufferVariant::Storage,
            _phantom: PhantomData,
        }
    }

    pub fn new_storage_with_runtime_array(
        device: &wgpu::Device,
        label: &str,
        data: &T,
        count: u64,
        usage: wgpu::BufferUsages,
    ) -> Self
    where
        T: encase::ShaderType + encase::internal::WriteInto + encase::CalculateSizeFor,
    {
        let usage = usage | wgpu::BufferUsages::STORAGE;
        let contents = write_storage_buffer(data);
        let size = T::calculate_size_for(count).get();
        assert!(size >= contents.len() as u64);

        // Copied from the "create_buffer_init" function
        // Valid vulkan usage is
        // 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
        // 2. buffer size must be greater than 0.
        // Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
        let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
        let padded_size = ((size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            usage,
            size: padded_size,
            mapped_at_creation: true,
        });
        buffer.slice(..contents.len() as u64).get_mapped_range_mut()[..contents.len()]
            .copy_from_slice(&contents);
        buffer.unmap();

        Self {
            buffer,
            variant: TypedBufferVariant::Storage,
            _phantom: PhantomData,
        }
    }

    pub fn write_buffer(&self, queue: &wgpu::Queue, data: &T)
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let contents = match self.variant {
            TypedBufferVariant::Uniform => write_uniform_buffer(data),
            TypedBufferVariant::Storage => write_storage_buffer(data),
        };
        // TODO: Use https://docs.rs/wgpu/23.0.0/wgpu/struct.Queue.html#method.write_buffer_with instead
        queue.write_buffer(&self.buffer, 0, contents.as_slice());
    }

    pub fn as_entire_buffer_binding(&self) -> wgpu::BufferBinding<'_> {
        self.buffer.as_entire_buffer_binding()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
pub trait CommandEncoderBufferExt<T> {
    fn copy_tbuffer_to_tbuffer(&mut self, buffer: &TypedBuffer<T>, other: &TypedBuffer<T>);
}
impl<T> CommandEncoderBufferExt<T> for wgpu::CommandEncoder {
    fn copy_tbuffer_to_tbuffer(&mut self, buffer: &TypedBuffer<T>, other: &TypedBuffer<T>) {
        self.copy_buffer_to_buffer(&buffer, 0, &other, 0, buffer.size());
    }
}

pub trait DeviceBufferExt<T> {
    fn storage_buffer(&self, label: &str, data: &T, usage: wgpu::BufferUsages) -> TypedBuffer<T>;
    fn storage_buffer_with_array(
        &self,
        label: &str,
        data: &T,
        count: u64,
        usage: wgpu::BufferUsages,
    ) -> TypedBuffer<T>
    where
        T: encase::CalculateSizeFor;
    fn uniform_buffer(&self, label: &str, data: &T, usage: wgpu::BufferUsages) -> TypedBuffer<T>;
}
impl<T> DeviceBufferExt<T> for wgpu::Device
where
    T: encase::ShaderType + encase::internal::WriteInto,
{
    fn storage_buffer(&self, label: &str, data: &T, usage: wgpu::BufferUsages) -> TypedBuffer<T> {
        TypedBuffer::new_storage(self, label, data, usage)
    }
    fn storage_buffer_with_array(
        &self,
        label: &str,
        data: &T,
        count: u64,
        usage: wgpu::BufferUsages,
    ) -> TypedBuffer<T>
    where
        T: encase::CalculateSizeFor,
    {
        TypedBuffer::new_storage_with_runtime_array(self, label, data, count, usage)
    }
    fn uniform_buffer(&self, label: &str, data: &T, usage: wgpu::BufferUsages) -> TypedBuffer<T> {
        TypedBuffer::new_uniform(self, label, data, usage)
    }
}

impl<T> std::ops::Deref for TypedBuffer<T>
where
    T: ?Sized,
{
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T> Clone for TypedBuffer<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            variant: self.variant,
            _phantom: self._phantom.clone(),
        }
    }
}

fn write_uniform_buffer<T>(data: &T) -> Vec<u8>
where
    T: ?Sized + encase::ShaderType + encase::internal::WriteInto,
{
    let mut buffer = encase::UniformBuffer::from(Vec::new());
    buffer
        .write(data)
        .expect("Failed to write uniform buffer. Did we run out of CPU memory?");
    buffer.into_inner()
}

fn write_storage_buffer<T>(data: &T) -> Vec<u8>
where
    T: ?Sized + encase::ShaderType + encase::internal::WriteInto,
{
    let mut buffer = encase::StorageBuffer::from(Vec::new());
    buffer
        .write(data)
        .expect("Failed to write storage buffer.  Did we run out of CPU memory?");
    buffer.into_inner()
}
