use std::f64::consts::PI;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Colors, Print, ResetColor, SetColors, SetForegroundColor},
    terminal,
};
use rand::Rng;

const FRAME_DURATION: Duration = Duration::from_millis(30);
const DECAY: f32 = 0.80;
const INTRO_FRAMES: u32 = 90;  // 90 × 30 ms = 2.7 s
const OUTRO_FRAMES: u32 = 80;  // 80 × 30 ms = 2.4 s

type Rgb = (u8, u8, u8);

// ── Animation phase ───────────────────────────────────────────────────────────
enum Phase {
    Intro(u32),   // frames elapsed
    Running,
    Outro(u32),   // frames elapsed
}

// ── Pixel buffer ──────────────────────────────────────────────────────────────
// '▀' upper-half block: foreground = top logical pixel, background = bottom.
// Terminal cells ~2× taller than wide  →  logical pixels are approximately square.
struct Canvas {
    w: usize,
    h: usize, // = (term_rows − 1) × 2
    px: Vec<(u8, u8, u8, f32)>,
}

impl Canvas {
    fn new(tw: u16, th: u16) -> Self {
        let w = tw as usize;
        let h = th.saturating_sub(1) as usize * 2;
        Canvas { w, h, px: vec![(0, 0, 0, 0.0); w * h] }
    }

    fn reset(&mut self) { self.px.fill((0, 0, 0, 0.0)); }

    fn decay(&mut self) {
        for p in &mut self.px {
            p.3 *= DECAY;
            if p.3 < 0.02 { *p = (0, 0, 0, 0.0); }
        }
    }

    fn put(&mut self, x: i32, y: i32, (r, g, b): Rgb, intensity: f32) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 { return; }
        let p = &mut self.px[y as usize * self.w + x as usize];
        if intensity > p.3 { *p = (r, g, b, intensity); }
    }

    fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, color: Rgb, intensity: f32) {
        if rx < 0.5 || ry < 0.5 { return; }
        let h = ((rx - ry) / (rx + ry)).powi(2);
        let circ = PI * (rx + ry) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()));
        let steps = (circ as usize * 2).max(64);
        for i in 0..steps {
            let a = 2.0 * PI * i as f64 / steps as f64;
            self.put(
                (cx + rx * a.cos()).round() as i32,
                (cy + ry * a.sin()).round() as i32,
                color, intensity,
            );
        }
    }

    fn disc(&mut self, cx: f64, cy: f64, r: f64, color: Rgb, intensity: f32) {
        if r < 0.1 { return; }
        let ri = r.ceil() as i32 + 1;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let d = f64::sqrt((dx * dx + dy * dy) as f64);
                if d <= r {
                    let alpha = if d > r - 1.0 { (1.0 - (d - (r - 1.0))) as f32 } else { 1.0 };
                    self.put(cx as i32 + dx, cy as i32 + dy, color, intensity * alpha.max(0.0));
                }
            }
        }
    }

    fn render(&self, out: &mut impl Write) -> io::Result<()> {
        let term_rows = self.h / 2;
        let mut last_fg = (255u8, 0, 0);
        let mut last_bg = (1u8, 0, 0);

        for ty in 0..term_rows {
            queue!(out, cursor::MoveTo(0, ty as u16))?;
            for tx in 0..self.w {
                let t = self.px[(ty * 2) * self.w + tx];
                let b = self.px[(ty * 2 + 1) * self.w + tx];

                let apply = |p: (u8, u8, u8, f32)| -> (u8, u8, u8) {
                    if p.3 > 0.01 {
                        ((p.0 as f32 * p.3) as u8, (p.1 as f32 * p.3) as u8, (p.2 as f32 * p.3) as u8)
                    } else { (0, 0, 0) }
                };

                let fg = apply(t);
                let bg = apply(b);

                if fg != last_fg || bg != last_bg {
                    queue!(out, SetColors(Colors::new(
                        Color::Rgb { r: fg.0, g: fg.1, b: fg.2 },
                        Color::Rgb { r: bg.0, g: bg.1, b: bg.2 },
                    )))?;
                    last_fg = fg;
                    last_bg = bg;
                }
                queue!(out, Print('▀'))?;
            }
        }
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────
fn smoothstep(lo: f64, hi: f64, x: f64) -> f64 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Draw the layered sun glow.  `scale` changes the radius, `boost` adds a hot
/// white core (used during the outro collapse).
fn draw_sun(canvas: &mut Canvas, cx: f64, cy: f64, scale: f64, boost: f32) {
    if scale < 0.01 { return; }
    let s = scale;
    canvas.disc(cx, cy, 7.0 * s, (200, 100,   0), (0.18 + boost * 0.20).min(1.0));
    canvas.disc(cx, cy, 5.0 * s, (255, 160,  20), (0.40 + boost * 0.30).min(1.0));
    canvas.disc(cx, cy, 3.2 * s, (255, 220,  60), (0.72 + boost * 0.20).min(1.0));
    canvas.disc(cx, cy, 1.8 * s, (255, 255, 160), (0.92 + boost * 0.08).min(1.0));
    canvas.disc(cx, cy, 0.8 * s, (255, 255, 240), 1.00);
}

// ── Solar system ──────────────────────────────────────────────────────────────
struct Planet {
    orbit_rx: f64,
    orbit_ry: f64,
    angle:    f64,
    speed:    f64,
    color:    Rgb,
    size:     f64,
}

fn rand_color(rng: &mut impl Rng) -> Rgb {
    let palette: &[Rgb] = &[
        (255,  80,  80), (80, 220, 80), (80, 160, 255),
        (255, 170,  50), (200, 80, 255), (60, 220, 200),
        (255, 235,  70), (255, 110, 180), (130, 130, 255),
        (160, 255, 110),
    ];
    palette[rng.random_range(0..palette.len())]
}

fn make_system(tw: u16, th: u16) -> Vec<Planet> {
    let mut rng = rand::rng();
    let count  = rng.random_range(3..=8usize);
    let lw     = tw as f64;
    let lh     = th.saturating_sub(1) as f64 * 2.0;
    let aspect = lw / lh;

    let max_ry = (lh / 2.0 - 6.0).min((lw / 2.0 - 8.0) / aspect);
    let min_ry = 6.0;
    let step   = (max_ry - min_ry) / count as f64;

    (0..count).map(|i| {
        let base  = min_ry + step * i as f64;
        let ry    = (base + rng.random_range(-step * 0.15..step * 0.15)).max(min_ry);
        let speed = 0.030 / (ry / min_ry).sqrt() * rng.random_range(0.6f64..1.4);
        Planet {
            orbit_rx: ry * aspect,
            orbit_ry: ry,
            angle:    rng.random_range(0.0..2.0 * PI),
            speed,
            color:    rand_color(&mut rng),
            size:     rng.random_range(1.2f64..2.8),
        }
    }).collect()
}

// ── Scene drawing ─────────────────────────────────────────────────────────────

/// Normal running frame.
fn draw_running(canvas: &mut Canvas, planets: &[Planet], cx: f64, cy: f64) {
    canvas.decay();
    for p in planets {
        canvas.ellipse(cx, cy, p.orbit_rx, p.orbit_ry, (30, 30, 48), 0.50);
    }
    draw_sun(canvas, cx, cy, 1.0, 0.0);
    for p in planets {
        let (px, py) = planet_pos(p, cx, cy, 1.0);
        let (r, g, b) = p.color;
        canvas.disc(px, py, p.size + 2.0, (r / 3, g / 3, b / 3), 0.30);
        canvas.disc(px, py, p.size, p.color, 1.00);
    }
}

/// Intro animation:  t ∈ [0, 1]
///   0.00 – 0.10  creation flash
///   0.05 – 0.40  sun materialises
///   0.20 – 0.85  orbits expand from centre outward, staggered per planet
///   0.35 – 1.00  planets appear
fn draw_intro(canvas: &mut Canvas, planets: &[Planet], cx: f64, cy: f64, t: f64) {
    canvas.decay();

    // Creation flash: brief white disc from the centre
    if t < 0.18 {
        let ft = t / 0.18;
        let flash_r = ft * canvas.w.min(canvas.h) as f64 * 0.25;
        canvas.disc(cx, cy, flash_r, (255, 255, 240), ((1.0 - ft) * 0.85) as f32);
    }

    // Sun
    let sun_s = smoothstep(0.05, 0.45, t);
    draw_sun(canvas, cx, cy, sun_s, 0.0);

    // Orbits + planets staggered
    let n = planets.len().max(1) as f64;
    for (i, p) in planets.iter().enumerate() {
        let fi = i as f64 / n;

        // Each orbit expands from 0 → full size
        let orbit_lo = 0.20 + 0.30 * fi;
        let orbit_hi = orbit_lo + 0.28;
        let os = smoothstep(orbit_lo, orbit_hi, t);
        canvas.ellipse(cx, cy, p.orbit_rx * os, p.orbit_ry * os, (30, 30, 48), (0.50 * os as f32).min(0.50));

        // Planet appears shortly after its orbit
        let planet_lo = orbit_lo + 0.12;
        let planet_hi = planet_lo + 0.20;
        let ps = smoothstep(planet_lo, planet_hi, t) as f32;
        if ps > 0.01 {
            let (px, py) = planet_pos(p, cx, cy, os);
            let (r, g, b) = p.color;
            canvas.disc(px, py, p.size + 2.0, (r / 3, g / 3, b / 3), 0.30 * ps);
            canvas.disc(px, py, p.size, p.color, ps);
        }
    }
}

/// Outro animation:  t ∈ [0, 1]
///   0.00 – 0.70  planets spiral inward (orbits shrink to 0)
///   0.40 – 0.75  sun swells and brightens (mass accretion)
///   0.70 – 0.85  supernova shockwave expands
///   0.80 – 1.00  everything fades to black
fn draw_outro(canvas: &mut Canvas, planets: &[Planet], cx: f64, cy: f64, t: f64) {
    canvas.decay();

    // Orbits + planets shrink inward
    let orbit_scale = 1.0 - smoothstep(0.0, 0.72, t);
    for p in planets {
        canvas.ellipse(cx, cy, p.orbit_rx * orbit_scale, p.orbit_ry * orbit_scale, (30, 30, 48), 0.50);
        if orbit_scale > 0.02 {
            let (px, py) = planet_pos(p, cx, cy, orbit_scale);
            let (r, g, b) = p.color;
            canvas.disc(px, py, p.size + 2.0, (r / 3, g / 3, b / 3), 0.30);
            canvas.disc(px, py, p.size, p.color, 1.00);
        }
    }

    // Sun swells as it gains mass, then collapses before the nova
    let swell    = smoothstep(0.35, 0.72, t);        // 1→ +70 % radius
    let collapse = smoothstep(0.70, 0.82, t);         // shrinks back to 0
    let sun_scale = (1.0 + swell * 0.70) * (1.0 - collapse);
    let boost     = (swell * 0.8) as f32;
    draw_sun(canvas, cx, cy, sun_scale, boost);

    // Supernova shockwave
    if t > 0.70 {
        let ft = smoothstep(0.70, 0.87, t);
        let max_r = canvas.w.min(canvas.h) as f64 * 0.55;
        let wave_r = ft * max_r;
        let fade   = (1.0 - smoothstep(0.75, 1.00, t)) as f32;
        canvas.disc(cx, cy, wave_r, (255, 255, 210), fade * 0.90);
    }
}

fn planet_pos(p: &Planet, cx: f64, cy: f64, orbit_scale: f64) -> (f64, f64) {
    (
        cx + p.orbit_rx * orbit_scale * p.angle.cos(),
        cy + p.orbit_ry * orbit_scale * p.angle.sin(),
    )
}

// ── Main loop ─────────────────────────────────────────────────────────────────
fn run(out: &mut impl Write) -> io::Result<()> {
    let (mut tw, mut th) = terminal::size()?;
    let mut canvas  = Canvas::new(tw, th);
    let mut planets = make_system(tw, th);
    let mut phase   = Phase::Intro(0);
    let mut t0      = Instant::now();

    loop {
        // Input
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent { code: KeyCode::Char('q' | 'Q'), .. }) => {
                    if !matches!(phase, Phase::Outro(_)) {
                        phase = Phase::Outro(0);
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Char('r' | 'R'), .. }) => {
                    planets = make_system(tw, th);
                    canvas.reset();
                    phase = Phase::Intro(0);
                }
                Event::Resize(w, h) => {
                    tw = w; th = h;
                    canvas  = Canvas::new(tw, th);
                    planets = make_system(tw, th);
                    phase   = Phase::Intro(0);
                }
                _ => {}
            }
        }

        // Frame rate cap
        let elapsed = t0.elapsed();
        if elapsed < FRAME_DURATION { std::thread::sleep(FRAME_DURATION - elapsed); }
        t0 = Instant::now();

        let cx = tw as f64 / 2.0;
        let cy = th.saturating_sub(1) as f64;

        // Advance planet angles every frame (even during animations)
        for p in &mut planets {
            p.angle = (p.angle + p.speed) % (2.0 * PI);
        }

        // Draw the appropriate phase
        match &mut phase {
            Phase::Intro(f) => {
                let t = *f as f64 / INTRO_FRAMES as f64;
                draw_intro(&mut canvas, &planets, cx, cy, t);
                *f += 1;
                if *f > INTRO_FRAMES { phase = Phase::Running; }
            }
            Phase::Running => {
                draw_running(&mut canvas, &planets, cx, cy);
            }
            Phase::Outro(f) => {
                let t = *f as f64 / OUTRO_FRAMES as f64;
                draw_outro(&mut canvas, &planets, cx, cy, t);
                *f += 1;
                if *f > OUTRO_FRAMES { return Ok(()); }
            }
        }

        canvas.render(out)?;

        // Status bar
        let label = match &phase {
            Phase::Intro(_)  => "".to_string(),
            Phase::Outro(_)  => "".to_string(),
            Phase::Running   => format!(" [q] Quit  │  [r] New system  │  {} planets ", planets.len()),
        };
        queue!(out,
            cursor::MoveTo(0, th - 1),
            ResetColor,
            SetForegroundColor(Color::DarkGrey),
            Print(&label),
        )?;

        out.flush()?;
    }
}

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = run(&mut out);

    execute!(out, terminal::LeaveAlternateScreen, cursor::Show, ResetColor)?;
    terminal::disable_raw_mode()?;
    result
}
