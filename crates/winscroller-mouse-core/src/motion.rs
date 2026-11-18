//! Motion tracking — 1-to-1 port of `Motion.js`.
//! Tracks the pointer's recent path, speed, stillness, and shake-to-find turns.

pub const MAX_POINTS: usize = 160;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    /// Normalized age: 0 = newest (at pointer), 1 = oldest (end of tail)
    pub t: f32,
}

#[derive(Clone, Copy)]
struct PointRaw {
    x: f32,
    y: f32,
    time_ms: f64,
}

pub struct MotionState {
    pts: Vec<PointRaw>,
    pub x: f32,
    pub y: f32,
    last_move_ms: f64,
    pub speed: f32,
    pub moved: f32,
    turns: Vec<f64>,
    dir: i32,
    run: f32,
}

impl Default for MotionState {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionState {
    pub fn new() -> Self {
        Self {
            pts: Vec::with_capacity(MAX_POINTS),
            x: f32::NAN,
            y: f32::NAN,
            last_move_ms: 0.0,
            speed: 0.0,
            moved: 0.0,
            turns: Vec::with_capacity(16),
            dir: 0,
            run: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn has_points(&self) -> bool {
        !self.pts.is_empty()
    }

    /// Add a new sample in virtual screen coordinates. Returns true if pointer moved.
    pub fn add(&mut self, x: f32, y: f32, now_ms: f64) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        let moved = self.x.is_nan() || x != self.x || y != self.y;
        if moved {
            let dx = if self.x.is_nan() { 0.0 } else { x - self.x };
            let dy = if self.y.is_nan() { 0.0 } else { y - self.y };
            let dist = (dx * dx + dy * dy).sqrt();
            let dt = if let Some(prev) = self.pts.first() {
                (now_ms - prev.time_ms).max(8.0)
            } else {
                16.0
            };
            let inst_speed = (dist / dt as f32) * 1000.0;
            self.speed = (self.speed * 0.6 + inst_speed * 0.4).min(8000.0);
            self.moved = dist;
            self.last_move_ms = now_ms;
            self.note_turn(dx, now_ms);
            self.pts.insert(
                0,
                PointRaw {
                    x,
                    y,
                    time_ms: now_ms,
                },
            );
            if self.pts.len() > MAX_POINTS {
                self.pts.truncate(MAX_POINTS);
            }
        } else {
            self.speed *= 0.7;
            self.moved = 0.0;
        }
        self.x = x;
        self.y = y;
        moved
    }

    /// Prune points older than `life_secs`.
    pub fn prune(&mut self, now_ms: f64, life_secs: f32) {
        let cut = now_ms - life_secs as f64 * 1000.0;
        while let Some(last) = self.pts.last() {
            if last.time_ms < cut {
                self.pts.pop();
            } else {
                break;
            }
        }
    }

    /// 0..1 speed fraction (~3000 px/s = 1.0)
    pub fn speed01(&self) -> f32 {
        (self.speed / 3000.0).clamp(0.0, 1.0)
    }

    /// Seconds since last pointer movement
    pub fn still_for(&self, now_ms: f64) -> f32 {
        if self.last_move_ms > 0.0 {
            ((now_ms - self.last_move_ms) / 1000.0).max(0.0) as f32
        } else {
            1e9
        }
    }

    /// Fill `out` with points relative to screen origin (ox, oy), newest first, t 0..1.
    /// Reuses caller's Vec to avoid per-frame allocations!
    pub fn points_for_into(
        &self,
        now_ms: f64,
        life_secs: f32,
        ox: f32,
        oy: f32,
        out: &mut Vec<Point>,
    ) {
        out.clear();
        let span = (life_secs as f64 * 1000.0).max(1.0);
        for p in &self.pts {
            let t = ((now_ms - p.time_ms) / span) as f32;
            if t > 1.0 {
                break;
            }
            out.push(Point {
                x: p.x - ox,
                y: p.y - oy,
                t: t.max(0.0),
            });
        }
    }

    fn note_turn(&mut self, dx: f32, now_ms: f64) {
        if dx == 0.0 {
            return;
        }
        let dir = if dx > 0.0 { 1 } else { -1 };
        if dir == self.dir {
            self.run += dx.abs();
            return;
        }
        if self.dir != 0 && self.run >= 30.0 {
            self.turns.push(now_ms);
        }
        self.dir = dir;
        self.run = dx.abs();
        while self.turns.len() > 12 {
            self.turns.remove(0);
        }
    }

    /// Returns true when the pointer has been shaken back and forth.
    pub fn shaken(&mut self, now_ms: f64, sensitivity: u32) -> bool {
        let s = sensitivity.clamp(1, 10) as f64;
        let need = (7.0 - s * 0.4).round() as usize;
        let within = 600.0 + s * 40.0;
        let count = self.turns.iter().filter(|&&t| now_ms - t <= within).count();
        if count >= need {
            self.turns.clear();
            true
        } else {
            false
        }
    }
}
