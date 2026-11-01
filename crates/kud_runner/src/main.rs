//! Headless runner used for simulation tests, profiling, and CI.
//!
//! Example:
//! `cargo run -p kud_runner --release -- --frames=600 --students=100 --hz=60`

use std::time::Instant;

use kud_engine::{Engine, EngineConfig};
use kud_render::{CameraDesc, NullRenderer};
use kud_sim::{Schedule, ScheduleBlock, SimConfig};

#[derive(Debug)]
struct Args {
    frames: u64,
    students: u32,
    hz: u32,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            frames: 600,
            students: 100,
            hz: 60,
        }
    }
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    for arg in std::env::args().skip(1) {
        if arg == "--help" || arg == "-h" {
            return Err("usage: kud_runner [--frames=N] [--students=N] [--hz=N]".to_string());
        }
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| format!("expected --key=value, got {arg}"))?;
        match key {
            "--frames" => args.frames = value.parse().map_err(|_| "invalid --frames")?,
            "--students" => args.students = value.parse().map_err(|_| "invalid --students")?,
            "--hz" => args.hz = value.parse().map_err(|_| "invalid --hz")?,
            _ => return Err(format!("unknown argument: {key}")),
        }
    }
    Ok(args)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = EngineConfig {
        fixed_hz: args.hz,
        sim: SimConfig::default(),
        ..EngineConfig::default()
    };
    let mut engine = Engine::new(config)?;
    engine.sim.spawn_player(glam::Vec3::ZERO);

    let side = (args.students as f32).sqrt().ceil() as u32;
    for id in 1..=args.students {
        let x = ((id - 1) % side) as f32 * 2.0;
        let z = ((id - 1) / side) as f32 * 2.0;
        let schedule = Schedule {
            blocks: vec![ScheduleBlock {
                start_minute: 0,
                end_minute: 24 * 60,
                destination: glam::Vec3::new(x, 0.0, z + 5.0),
                action: "Idle".to_string(),
                phase: 1,
            }],
        };
        engine
            .sim
            .spawn_student(id, glam::Vec3::new(x, 0.0, z), schedule);
    }

    let mut renderer = NullRenderer::default();
    let frame_seconds = 1.0 / args.hz as f32;
    let start = Instant::now();
    let mut total_steps = 0_u64;
    let mut total_draws = 0_u64;
    let mut total_events = 0_u64;

    for _ in 0..args.frames {
        let report = engine.update(frame_seconds)?;
        total_steps += u64::from(report.fixed_steps);
        total_events += report.events.len() as u64;
        total_draws += u64::from(engine.render(&mut renderer, CameraDesc::looking_at_origin(1.0))?);
    }

    let elapsed = start.elapsed();
    let summary = serde_json::json!({
        "frames": args.frames,
        "students": args.students,
        "fixed_hz": args.hz,
        "fixed_steps": total_steps,
        "draws": total_draws,
        "events": total_events,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "sim_ticks": engine.sim.tick_count,
    });
    println!("{summary}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
