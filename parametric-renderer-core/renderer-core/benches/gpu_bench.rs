#![allow(unused)]

use std::{
    cell::Cell,
    ops::Range,
    sync::{Arc, OnceLock},
};

use criterion::{
    measurement::{Measurement, ValueFormatter},
    Throughput,
};
use tracing::{info, trace, warn};
use wgpu::{Adapter, DeviceType, Limits, QuerySet};

fn get_timer() -> &'static WgpuTimer {
    static TIMER: OnceLock<WgpuTimer> = OnceLock::new();
    TIMER.get_or_init(|| {
        WgpuTimer::new(pollster::block_on(async {
            GPUHandle::new().await.unwrap()
        }))
    })
}

fn main() {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    pub fn criterion_benchmark(c: &mut Criterion<&WgpuTimer>) {
        // Maybe also measure throughput? https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#throughput-measurements
        // See https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/bench.rs#L165
        // which gets the throughput from https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/benches/mlx-gemv/gemv.rs#L114

        let timer = get_timer();
        let handle = timer.handle();

        // TOOD: Create an application

        c.bench_function("render", |b| {
            b.iter(|| {
                let tsw = timer.timestamp_writes();
                // The render loop
                // https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/bench.rs#L52-L78

                // call render_commands
                dispatch(handle, &workload, &bind_groups, &pipeline, Some(tsw));
                timer.increment_query();
            })
        });
    }

    criterion_group!(
        name = benches;
        config = Criterion::default().with_measurement(&*get_timer());
        targets = criterion_benchmark
    );
    criterion_main!(benches);
}
// From https://github.com/FL33TW00D/wgpu-bench/blob/93754d88ed55218d53eef40f3e3814eff8f52467/src/handle.rs#L17
pub struct GPUHandle {
    pub inner: Arc<GPUHandleInner>,
}
pub struct GPUHandleInner {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl std::ops::Deref for GPUHandle {
    type Target = GPUHandleInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl GPUHandle {
    fn get_features() -> wgpu::Features {
        wgpu::Features::default() | wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::SUBGROUP
    }

    pub async fn new() -> Result<Self, anyhow::Error> {
        let adapter = Self::select_adapter();

        let mut device_descriptor = wgpu::DeviceDescriptor {
            label: Some("rumble"),
            required_features: Self::get_features(),
            required_limits: Limits {
                max_buffer_size: (2 << 29) - 1,
                max_storage_buffer_binding_size: (2 << 29) - 1,
                max_compute_invocations_per_workgroup: 1024,
                ..Default::default()
            },
        };
        let device_request = adapter.request_device(&device_descriptor, None).await;
        let (device, queue) = if let Err(e) = device_request {
            warn!("Failed to create device with error: {:?}", e);
            warn!("Trying again with reduced limits");
            device_descriptor.required_limits = adapter.limits();
            let device_request = adapter.request_device(&device_descriptor, None).await;
            device_request.unwrap()
        } else {
            device_request.unwrap()
        };

        Ok(Self {
            inner: Arc::new(GPUHandleInner { device, queue }),
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn select_adapter() -> Adapter {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backends.clone(),
            ..Default::default()
        });

        let adapter = {
            let mut most_performant_adapter = None;
            let mut current_score = -1;

            instance
                .enumerate_adapters(backends)
                .into_iter()
                .for_each(|adapter| {
                    let info = adapter.get_info();
                    let score = match info.device_type {
                        DeviceType::DiscreteGpu => 5,
                        DeviceType::Other => 4, //Other is usually discrete
                        DeviceType::IntegratedGpu => 3,
                        DeviceType::VirtualGpu => 2,
                        DeviceType::Cpu => 1,
                    };

                    if score > current_score {
                        most_performant_adapter = Some(adapter);
                        current_score = score;
                    }
                });

            if let Some(adapter) = most_performant_adapter {
                adapter
            } else {
                panic!("No adapter found, please check if your GPU is supported");
            }
        };
        info!("Using adapter {:?}", adapter.get_info());
        adapter
    }
}

// From https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/lib.rs#L36
// But log has been replaced with tracing.
pub const MAX_QUERIES: u32 = 4096;

/// Start and end index in the counter sample buffer
#[derive(Debug, Clone, Copy)]
pub struct QueryPair {
    pub start: u32,
    pub end: u32,
}

impl QueryPair {
    pub fn first() -> Self {
        Self { start: 0, end: 1 }
    }

    pub fn size(&self) -> wgpu::BufferAddress {
        ((self.end - self.start + 1) as usize * std::mem::size_of::<u64>()) as wgpu::BufferAddress
    }

    pub fn start_address(&self) -> wgpu::BufferAddress {
        (self.start as usize * std::mem::size_of::<u64>()) as wgpu::BufferAddress
    }

    pub fn end_address(&self) -> wgpu::BufferAddress {
        ((self.end + 1) as usize * std::mem::size_of::<u64>()) as wgpu::BufferAddress
    }
}

impl From<QueryPair> for Range<u32> {
    fn from(val: QueryPair) -> Self {
        val.start..val.end + 1
    }
}

pub struct WgpuTimer {
    handle: GPUHandle,
    query_set: QuerySet,
    resolve_buffer: wgpu::Buffer,
    destination_buffer: wgpu::Buffer,
    current_query: Cell<QueryPair>,
}

//TODO: dumb
unsafe impl Send for WgpuTimer {}
unsafe impl Sync for WgpuTimer {}

impl WgpuTimer {
    pub const COMPUTE_PER_QUERY: u64 = 100;

    pub fn new(handle: GPUHandle) -> Self {
        let query_set = handle.device().create_query_set(&wgpu::QuerySetDescriptor {
            count: MAX_QUERIES,
            ty: wgpu::QueryType::Timestamp,
            label: None,
        });

        let size = MAX_QUERIES as u64 * std::mem::size_of::<u64>() as u64;

        let resolve_buffer = handle.device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
            mapped_at_creation: false,
        });

        let destination_buffer = handle.device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            handle,
            query_set,
            resolve_buffer,
            destination_buffer,
            current_query: QueryPair::first().into(),
        }
    }

    pub fn resolve_pass(&self, encoder: &mut wgpu::CommandEncoder, pass_query: QueryPair) {
        let resolution_range = pass_query.into();
        trace!("Resolution range: {:?}", resolution_range);
        encoder.resolve_query_set(&self.query_set, resolution_range, &self.resolve_buffer, 0);
        let size = pass_query.size();
        trace!("Resolution size in bytes: {:?}", size);
        encoder.copy_buffer_to_buffer(&self.resolve_buffer, 0, &self.destination_buffer, 0, size);
    }

    pub fn handle(&self) -> &GPUHandle {
        &self.handle
    }

    pub fn query_set(&self) -> &QuerySet {
        &self.query_set
    }

    pub fn increment_query(&self) {
        let pair = self.current_query.get();
        if pair.end + 2 >= MAX_QUERIES {
            panic!("Number of queries exceeds MAX_QUERIES, reduce duration of benchmark");
        }
        self.current_query.set(QueryPair {
            start: pair.start + 2,
            end: pair.end + 2,
        });
    }

    pub fn current_query(&self) -> QueryPair {
        self.current_query.get()
    }

    //Fetches the current query as ComputePassTimestampWrites
    pub fn timestamp_writes(&self) -> wgpu::ComputePassTimestampWrites {
        wgpu::ComputePassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(self.current_query().start),
            end_of_pass_write_index: Some(self.current_query().end),
        }
    }

    pub fn hardware_elapsed(&self, timestamps: &[u64]) -> u64 {
        assert!(timestamps.len() % 2 == 0);
        let mut elapsed = 0;
        for i in (0..timestamps.len()).step_by(2) {
            elapsed += timestamps[i + 1] - timestamps[i];
        }
        elapsed
    }
}

impl Measurement for &WgpuTimer {
    type Intermediate = u32; // Index of the start query

    type Value = u64; // Raw unscaled GPU counter
                      // Must be multiplied by the timestamp period to get nanoseconds

    fn start(&self) -> Self::Intermediate {
        trace!("\nQuery at start of pass: {:?}", self.current_query());
        0
    }

    fn end(&self, start_index: Self::Intermediate) -> Self::Value {
        trace!("\nQuery at end of pass: {:?}", self.current_query());
        let mut encoder = self
            .handle
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        //Large window, eg 0..512
        let pass_query = QueryPair {
            start: start_index,
            end: self.current_query().end - 2, //decrement here to counteract last iter
        };
        trace!("Pass range: {:?}", pass_query);

        self.resolve_pass(&mut encoder, pass_query);
        self.handle().queue().submit(Some(encoder.finish()));
        self.handle.device().poll(wgpu::Maintain::Wait);

        self.destination_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| ());
        self.handle.device().poll(wgpu::Maintain::Wait);
        let timestamps: Vec<u64> = {
            let byte_range = pass_query.start_address()..pass_query.end_address();
            let timestamp_view = self.destination_buffer.slice(byte_range).get_mapped_range();
            (*bytemuck::cast_slice(&timestamp_view)).to_vec()
        };
        trace!("Timestamps: {:?}", timestamps);
        self.destination_buffer.unmap();
        self.current_query.set(QueryPair::first());
        self.hardware_elapsed(&timestamps) / WgpuTimer::COMPUTE_PER_QUERY
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        (self.handle.queue().get_timestamp_period() as f64) * (*value as f64)
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &WgpuTimerFormatter
    }
}

struct WgpuTimerFormatter;

impl ValueFormatter for WgpuTimerFormatter {
    fn format_value(&self, value: f64) -> String {
        format!("{:.4} ns", value)
    }

    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String {
        match throughput {
            Throughput::Bytes(b) => format!(
                "{:.4} GiB/s",
                (*b as f64) / (1024.0 * 1024.0 * 1024.0) / (value * 1e-9)
            ),
            Throughput::Elements(e) => {
                let gflop = (*e as f64) / 1e9;
                let seconds = value * 1e-9;
                let gigaflop_per_second = gflop / seconds;
                format!("{:.4} GFLOP/s", gigaflop_per_second)
            }
            _ => unreachable!(),
        }
    }

    fn scale_values(&self, _typical_value: f64, _values: &mut [f64]) -> &'static str {
        "ns"
    }

    /// TODO!
    fn scale_throughputs(
        &self,
        _typical_value: f64,
        throughput: &Throughput,
        _values: &mut [f64],
    ) -> &'static str {
        match throughput {
            Throughput::Bytes(_) => "GiB/s",
            Throughput::Elements(_) => "elements/s",
            _ => unreachable!(),
        }
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "ns"
    }
}
