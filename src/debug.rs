use std::time::{Duration, Instant};

pub struct FpsTracker {
    fps_counter: u32,
    fps_timer: Instant,
    render_time_accum: Duration,
    last_frame_render_time: Duration,
}

impl FpsTracker {
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            fps_counter: 0,
            fps_timer: now,
            render_time_accum: Duration::ZERO,
            last_frame_render_time: Duration::ZERO,
        }
    }

    pub fn begin_render(&self) -> Instant {
        Instant::now()
    }

    pub fn end_render(&mut self, start: Instant) {
        let render_time = start.elapsed();
        self.last_frame_render_time = render_time;
        self.render_time_accum += render_time;
    }

    pub fn tick(&mut self) {
        self.fps_counter += 1;

        let elapsed = self.fps_timer.elapsed();
        if elapsed >= Duration::from_secs(1) {
            println!(
                "Actual FPS: {:.2}",
                self.fps_counter as f64 / elapsed.as_secs_f64()
            );
            println!(
                "Last Frame Render Time: {:.2} ms",
                self.last_frame_render_time.as_secs_f32() * 1000.0
            );
            println!(
                "Average Render Time: {:.2} ms",
                self.render_time_accum.as_secs_f32() * 1000.0 / self.fps_counter as f32
            );
            self.fps_counter = 0;
            self.fps_timer = Instant::now();
            self.render_time_accum = Duration::ZERO;
        }
    }
}

pub struct FramePacer {
    frame_duration: Duration,
    next_frame_time: Instant,
}

impl FramePacer {
    pub fn new(target_fps: u16) -> Self {
        let frame_duration = Duration::from_secs_f64(1.0 / target_fps.max(1) as f64);

        Self {
            frame_duration,
            next_frame_time: Instant::now(),
        }
    }

    pub fn should_request_redraw(&mut self, now: Instant) -> bool {
        if now < self.next_frame_time {
            return false;
        }

        self.next_frame_time = now + self.frame_duration;
        true
    }

    
pub fn next_frame_time(&self) -> Instant {
        self.next_frame_time
    }
}