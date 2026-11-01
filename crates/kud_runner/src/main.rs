//! Headless runner used for simulation tests, profiling, replay, and CI.
//!
//! Examples:
//! ```text
//! kud_runner bench --frames=600 --students=100 --hz=60
//! kud_runner record --scenario=school_day --ticks=600 --out=run.json
//! kud_runner verify --replay=run.json
//! ```

mod bench;
mod cli;

use std::time::Instant;

use kud_core::CoreResult;
use kud_engine::{Engine, EngineConfig};
use kud_render::{CameraDesc, NullRenderer};
use kud_sim::input::InputCommand;
use kud_sim::replay::{Replay, ReplayError};
use kud_sim::scenario::Scenario;
use kud_sim::SimConfig;

use crate::cli::{Command, Options};

fn main() {
    let options = match Options::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    if options.help {
        println!("{}", cli::USAGE);
        return;
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let result = match options.command {
        Command::Bench => run_bench(&options),
        Command::Record => run_record(&options),
        Command::Verify => run_verify(&options),
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

/// Builds an engine for a scenario at an explicit rate and configuration.
fn build_engine(hz: u32, config: SimConfig, scenario: Scenario) -> CoreResult<Engine> {
    let mut engine = Engine::new(EngineConfig {
        fixed_hz: hz,
        sim: config,
        ..EngineConfig::default()
    })?;
    engine.sim = scenario
        .build(config)
        .map_err(|_| kud_core::CoreError::InvalidConfig {
            field: "scenario",
            reason: "scenario setup produced a duplicate or reserved student id",
        })?;
    Ok(engine)
}

/// Simulation configuration described by the command line.
fn options_config(options: &Options) -> SimConfig {
    SimConfig {
        minutes_per_tick: options.minutes_per_tick,
    }
}

/// Runs a fixed number of frames and reports timing and simulation counters.
fn run_bench(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = options.scenario()?;
    let mut engine = build_engine(options.hz, options_config(options), scenario)?;
    let mut renderer = NullRenderer::default();
    let frame_seconds = 1.0 / f64::from(options.hz) as f32;

    let mut frame_micros = Vec::with_capacity(options.frames as usize);
    let mut total_steps = 0_u64;
    let mut total_draws = 0_u64;
    let mut total_events = 0_u64;

    // Warm up so the first-frame allocator and branch predictor cost is excluded.
    for _ in 0..options.warmup {
        engine.update(frame_seconds)?;
        engine.render(&mut renderer, CameraDesc::looking_at_origin(1.0))?;
    }
    frame_micros.clear();

    let start = Instant::now();
    for _ in 0..options.frames {
        let frame_start = Instant::now();
        let report = engine.update(frame_seconds)?;
        total_steps += u64::from(report.fixed_steps);
        total_events += report.events.len() as u64;
        total_draws += u64::from(engine.render(&mut renderer, CameraDesc::looking_at_origin(1.0))?);
        frame_micros.push(frame_start.elapsed().as_micros() as u64);
    }
    let elapsed = start.elapsed();

    let stats = bench::TimingStats::from_samples(&mut frame_micros);
    let summary = serde_json::json!({
        "command": "bench",
        "scenario": scenario.name(),
        "frames": options.frames,
        "students": engine.sim.registry.len(),
        "fixed_hz": options.hz,
        "fixed_steps": total_steps,
        "draws": total_draws,
        "events": total_events,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "sim_ticks": engine.sim.tick_count,
        "frame_us": stats.as_json(),
        "state_hash": format!("{:#018x}", engine.sim.state_hash()),
    });
    println!("{summary}");

    if let Some(budget) = options.budget_us {
        if stats.p95 > budget {
            return Err(format!("frame p95 {} us exceeds budget {} us", stats.p95, budget).into());
        }
    }
    Ok(())
}

/// Records a deterministic replay driven by a scripted input pattern.
fn run_record(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = options.scenario()?;
    let config = options_config(options);
    let mut engine = build_engine(options.hz, config, scenario)?;
    let dt = 1.0 / options.hz as f32;
    let mut replay = Replay::new(options.hz, options.seed, scenario, config);

    for tick in 0..options.ticks {
        let command = scripted_command(tick);
        replay.record_tick(command);
        engine.sim.tick_with_input(dt, command);
    }
    replay.finish(engine.sim.state_hash());

    let json = serde_json::to_string_pretty(&replay)?;
    match &options.path {
        Some(path) => {
            std::fs::write(path, &json)?;
            eprintln!("wrote {path}");
        }
        None => println!("{json}"),
    }
    eprintln!(
        "recorded {} ticks of {}, final hash {:#018x}",
        replay.ticks,
        scenario.name(),
        replay.final_hash
    );
    Ok(())
}

/// Replays a recording and fails on the first divergent tick.
fn run_verify(options: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let path = options
        .path
        .as_ref()
        .ok_or("verify requires --replay=PATH")?;
    let text = std::fs::read_to_string(path)?;
    let replay: Replay = serde_json::from_str(&text)?;
    replay.validate()?;

    // The replay is self-contained: scenario and config come from the file,
    // so verification cannot silently run against a different setup.
    let scenario = replay.scenario;
    let mut engine = build_engine(replay.fixed_hz, replay.config, scenario)?;
    let dt = 1.0 / replay.fixed_hz as f32;

    for tick in 0..replay.ticks {
        engine.sim.tick_with_input(dt, replay.command_at(tick));
    }

    let actual = engine.sim.state_hash();
    if actual != replay.final_hash {
        return Err(ReplayError::Desynchronized {
            tick: replay.ticks,
            expected: replay.final_hash,
            actual,
        }
        .into());
    }

    println!(
        "{}",
        serde_json::json!({
            "command": "verify",
            "scenario": scenario.name(),
            "replay": path,
            "ticks": replay.ticks,
            "state_hash": format!("{actual:#018x}"),
            "result": "match",
        })
    );
    Ok(())
}

/// Deterministic input pattern used when recording without a live device.
///
/// Walks a square so the recording exercises movement and perception instead of
/// sitting idle, while remaining a pure function of the tick index.
fn scripted_command(tick: u64) -> InputCommand {
    match (tick / 60) % 5 {
        0 => InputCommand::Move { x: 127, z: 0 },
        1 => InputCommand::Move { x: 0, z: 127 },
        2 => InputCommand::Move { x: -127, z: 0 },
        3 => InputCommand::Move { x: 0, z: -127 },
        _ => InputCommand::Idle,
    }
}
