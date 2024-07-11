use std::{cell::Cell, sync::OnceLock};

use criterion::{
    criterion_group, criterion_main,
    measurement::{Measurement, ValueFormatter},
    Criterion, Throughput,
};
use glamour::Vector2;
use pollster::FutureExt;
use renderer_core::application::{CpuApplication, ProfilerSettings, WindowOrFallback};

fn get_timer() -> &'static WgpuTimer {
    static TIMER: OnceLock<WgpuTimer> = OnceLock::new();
    TIMER.get_or_init(|| WgpuTimer::new())
}

const DEFAULT_SHADER_CODE: &'static str = include_str!("../../shaders/HeartSphere.wgsl");

pub fn criterion_benchmark(c: &mut Criterion<&WgpuTimer>) {
    // Maybe also measure throughput? https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#throughput-measurements
    // See https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/bench.rs#L165
    // which gets the throughput from https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/benches/mlx-gemv/gemv.rs#L114

    let timer = get_timer();

    let mut app = CpuApplication::new().unwrap();
    app.set_profiling(ProfilerSettings { gpu: true });

    app.update_models(vec![renderer_core::application::ModelInfo {
        label: "Default Model".to_owned(),
        transform: renderer_core::transform::Transform {
            position: glamour::Point3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        material_info: renderer_core::application::MaterialInfo {
            color: glamour::Vector3::new(0.6, 1.0, 1.0),
            emissive: glamour::Vector3::new(0.0, 0.0, 0.0),
            roughness: 0.7,
            metallic: 0.1,
        },
        evaluate_image_code: DEFAULT_SHADER_CODE.to_owned(),
    }]);
    app.create_surface(WindowOrFallback::Headless {
        size: Vector2::new(1280, 720),
    })
    .block_on()
    .unwrap();

    let mut group = c.benchmark_group("render");
    // group.throughput(throughput);
    group.bench_function("render", |b| {
        b.iter(|| {
            // The render loop
            // https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/bench.rs#L52-L78

            // If we aren't allowed to mutate, then we'll either use a different criterion function
            // or we'll use the internal render_commands function
            app.render().unwrap();
            app.gpu.as_ref().unwrap().force_wait();
            timer.increment_query(app.get_profiling_data().unwrap());
        })
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(&*get_timer());
    targets = criterion_benchmark
);
criterion_main!(benches);

// From https://github.com/FL33TW00D/wgpu-bench/blob/db76a8dc5508ba183f36df9f6b2551712d582355/src/lib.rs#L36
// But log has been replaced with tracing.

pub struct WgpuTimer {
    current_query: Cell<(f64, f64)>,
}

//TODO: dumb
unsafe impl Send for WgpuTimer {}
unsafe impl Sync for WgpuTimer {}

impl WgpuTimer {
    pub fn new() -> Self {
        Self {
            current_query: Cell::new((0.0, 0.0)),
        }
    }

    pub fn increment_query(&self, profiling_data: Vec<wgpu_profiler::GpuTimerQueryResult>) {
        let mut range = (f64::INFINITY, f64::NEG_INFINITY);

        for query in profiling_data {
            if let Some(time) = query.time {
                range = (range.0.min(time.start), range.1.max(time.end));
            }
        }
        if range.0 == f64::INFINITY {
            panic!("No queries found");
        }
        self.current_query.set((range.0, range.1));
    }

    pub fn current_query(&self) -> (f64, f64) {
        self.current_query.get()
    }
}

impl Measurement for &WgpuTimer {
    type Intermediate = (); // No intermediate value

    type Value = f64; // Time from first query to last query

    fn start(&self) -> Self::Intermediate {
        ()
    }

    fn end(&self, _: Self::Intermediate) -> Self::Value {
        let (start, end) = self.current_query();
        (end - start) * 1000f64 // Convert to ms
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0f64
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        *value
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &WgpuTimerFormatter
    }
}

struct WgpuTimerFormatter;

impl ValueFormatter for WgpuTimerFormatter {
    fn format_value(&self, value: f64) -> String {
        format!("{:.4} ms", value)
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
        "ms"
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
        "ms"
    }
}
