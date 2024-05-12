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
    ) -> Result<Self, encase::internal::Error>
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &write_uniform_buffer(data)?,
            usage,
        });

        Ok(Self {
            buffer,
            variant: TypedBufferVariant::Uniform,
            _phantom: PhantomData,
        })
    }

    pub fn new_storage(
        device: &wgpu::Device,
        label: &str,
        data: &T,
        usage: wgpu::BufferUsages,
    ) -> Result<Self, encase::internal::Error>
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &write_storage_buffer(data)?,
            usage,
        });

        Ok(Self {
            buffer,
            variant: TypedBufferVariant::Storage,
            _phantom: PhantomData,
        })
    }

    pub fn new_storage_with_runtime_array(
        device: &wgpu::Device,
        label: &str,
        data: &T,
        count: u64,
        usage: wgpu::BufferUsages,
    ) -> Result<Self, encase::internal::Error>
    where
        T: encase::ShaderType + encase::internal::WriteInto + encase::CalculateSizeFor,
    {
        let contents = write_storage_buffer(data)?;
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

        Ok(Self {
            buffer,
            variant: TypedBufferVariant::Storage,
            _phantom: PhantomData,
        })
    }

    pub fn update(&self, queue: &wgpu::Queue, data: &T) -> Result<(), encase::internal::Error>
    where
        T: encase::ShaderType + encase::internal::WriteInto,
    {
        let contents = match self.variant {
            TypedBufferVariant::Uniform => write_uniform_buffer(data)?,
            TypedBufferVariant::Storage => write_storage_buffer(data)?,
        };
        queue.write_buffer(&self.buffer, 0, contents.as_slice());

        Ok(())
    }

    pub fn as_entire_buffer_binding(&self) -> wgpu::BufferBinding<'_> {
        self.buffer.as_entire_buffer_binding()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

fn write_uniform_buffer<T>(data: &T) -> Result<Vec<u8>, encase::internal::Error>
where
    T: ?Sized + encase::ShaderType + encase::internal::WriteInto,
{
    let mut buffer = encase::UniformBuffer::from(Vec::new());
    buffer.write(data)?;
    Ok(buffer.into_inner())
}

fn write_storage_buffer<T>(data: &T) -> Result<Vec<u8>, encase::internal::Error>
where
    T: ?Sized + encase::ShaderType + encase::internal::WriteInto,
{
    let mut buffer = encase::StorageBuffer::from(Vec::new());
    buffer.write(data)?;
    Ok(buffer.into_inner())
}
