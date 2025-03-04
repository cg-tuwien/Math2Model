use web_time::Instant;

#[derive(Debug, Default, Clone, Copy)]
pub struct Seconds(pub f32);

pub struct FrameTime {
    pub frame: u64,
    pub delta: Seconds,
    pub elapsed: Seconds,
}

pub struct FrameCounter {
    pub frame: u64,
    pub first_render_instant: Option<Instant>,
    pub render_instant: Option<Instant>,
}
impl FrameCounter {
    pub fn new() -> Self {
        Self {
            frame: 0,
            first_render_instant: None,
            render_instant: None,
        }
    }

    pub fn new_frame(&mut self) -> FrameTime {
        let frame = self.frame;
        let now = Instant::now();
        let first_render_instant = *self.first_render_instant.get_or_insert(now);
        let previous_render_instant = *self.render_instant.get_or_insert(now);
        let delta = Seconds((now - previous_render_instant).as_secs_f32());
        let elapsed = Seconds((now - first_render_instant).as_secs_f32());
        self.render_instant = Some(now);
        self.frame += 1;
        FrameTime {
            frame,
            delta,
            elapsed,
        }
    }
}
