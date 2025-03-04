use super::Seconds;

#[derive(Default, Clone)]
pub struct TimeStats {
    pub avg_delta_time: f32,
    pub avg_gpu_time: f32,
}

#[derive(Default)]
pub struct TimeCounters {
    pub frame_count: u64,
    pub last_results: Option<Vec<wgpu_profiler::GpuTimerQueryResult>>,
    cpu_delta_times: Vec<Seconds>,
    gpu_frame_times: Vec<Seconds>,
}

impl TimeCounters {
    pub fn push_frame(
        &mut self,
        cpu_delta_time: Seconds,
        gpu_time_results: Option<Vec<wgpu_profiler::GpuTimerQueryResult>>,
    ) {
        self.frame_count += 1;
        self.cpu_delta_times.push(cpu_delta_time);
        if self.cpu_delta_times.len() > 10 {
            self.cpu_delta_times.remove(0);
        }
        if let Some(gpu_time_results) = gpu_time_results {
            let time_range = get_time_range(&gpu_time_results);
            if let Some(frame_time) = time_range.map(|v| Seconds((v.end - v.start) as f32)) {
                self.gpu_frame_times.push(frame_time);
                if self.gpu_frame_times.len() > 10 {
                    self.gpu_frame_times.remove(0);
                }
            }
            self.last_results = Some(gpu_time_results);
        }
    }

    pub fn stats(&self) -> TimeStats {
        TimeStats {
            avg_delta_time: avg_seconds(&self.cpu_delta_times),
            avg_gpu_time: avg_seconds(&self.gpu_frame_times),
        }
    }
}

fn get_time_range(
    gpu_time_results: &[wgpu_profiler::GpuTimerQueryResult],
) -> Option<std::ops::Range<f64>> {
    let time_range = gpu_time_results
        .iter()
        .filter_map(|v| v.time.clone().or_else(|| get_time_range(&v.nested_queries)))
        .reduce(|a, b| a.start.min(b.start)..a.end.max(b.end));
    time_range
}

fn avg_seconds(data: &[Seconds]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().map(|v| v.0).sum::<f32>() / (data.len() as f32)
}
