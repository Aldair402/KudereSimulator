//! Command-line parsing for the headless runner.
//!
//! Hand-rolled to keep the runner dependency-free apart from the engine itself.
//! Every parse failure is a user-facing message, never a panic.

use kud_sim::scenario::Scenario;

/// Usage text printed for `--help` and on a parse error.
pub const USAGE: &str = "\
kud_runner — headless simulation runner

USAGE:
    kud_runner <bench|record|verify> [OPTIONS]

COMMANDS:
    bench     Run frames and report timing percentiles and a state hash
    record    Record a deterministic replay to a JSON file
    verify    Replay a recording and fail on divergence

OPTIONS:
    --scenario=NAME     grid | school_day            (default: grid)
    --students=N        Students in the scenario     (default: 100)
    --frames=N        Frames to run for bench      (default: 600)
    --ticks=N         Ticks to record              (default: 600)
    --hz=N            Fixed simulation rate        (default: 60)
    --minutes-per-tick=N  In-game minutes per tick   (default: 1)
    --warmup=N        Frames excluded from timing  (default: 30)
    --seed=N          Replay seed                  (default: 0)
    --budget-us=N     Fail if frame p95 exceeds N microseconds
    --out=PATH        Output file for record
    --replay=PATH     Input file for verify (scenario comes from the file)
    --help            Show this message
";

/// Subcommand selected on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Run a timing benchmark.
    Bench,
    /// Record a replay.
    Record,
    /// Verify a replay.
    Verify,
}

/// Parsed command-line options.
#[derive(Debug, Clone)]
pub struct Options {
    /// Selected subcommand.
    pub command: Command,
    /// Scenario name.
    pub scenario_name: String,
    /// Students to spawn.
    pub students: u32,
    /// Frames to run for `bench`.
    pub frames: u64,
    /// Ticks to record for `record`.
    pub ticks: u64,
    /// Fixed simulation rate in hertz.
    pub hz: u32,
    /// In-game minutes advanced per tick.
    pub minutes_per_tick: u16,
    /// Warmup frames excluded from timing.
    pub warmup: u64,
    /// Replay seed.
    pub seed: u64,
    /// Optional frame-time budget in microseconds.
    pub budget_us: Option<u64>,
    /// Input or output path.
    pub path: Option<String>,
    /// Whether help was requested.
    pub help: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            command: Command::Bench,
            scenario_name: "grid".to_string(),
            students: 100,
            frames: 600,
            ticks: 600,
            hz: 60,
            minutes_per_tick: 1,
            warmup: 30,
            seed: 0,
            budget_us: None,
            path: None,
            help: false,
        }
    }
}

impl Options {
    /// Parses arguments, defaulting to `bench` when no subcommand is given.
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut options = Self::default();
        let mut saw_command = false;

        for arg in args {
            let arg: String = arg.into();
            if arg == "--help" || arg == "-h" {
                options.help = true;
                return Ok(options);
            }

            if !arg.starts_with('-') {
                if saw_command {
                    return Err(format!("unexpected argument: {arg}\n\n{USAGE}"));
                }
                options.command = match arg.as_str() {
                    "bench" => Command::Bench,
                    "record" => Command::Record,
                    "verify" => Command::Verify,
                    other => return Err(format!("unknown command: {other}\n\n{USAGE}")),
                };
                saw_command = true;
                continue;
            }

            let (key, value) = arg
                .split_once('=')
                .ok_or_else(|| format!("expected --key=value, got {arg}"))?;

            match key {
                "--scenario" => options.scenario_name = value.to_string(),
                "--students" => options.students = parse_number(key, value)?,
                "--frames" => options.frames = parse_number(key, value)?,
                "--ticks" => options.ticks = parse_number(key, value)?,
                "--hz" => options.hz = parse_number(key, value)?,
                "--minutes-per-tick" => options.minutes_per_tick = parse_number(key, value)?,
                "--warmup" => options.warmup = parse_number(key, value)?,
                "--seed" => options.seed = parse_number(key, value)?,
                "--budget-us" => options.budget_us = Some(parse_number(key, value)?),
                "--out" | "--replay" => options.path = Some(value.to_string()),
                _ => return Err(format!("unknown option: {key}")),
            }
        }

        if options.hz == 0 || options.hz > 240 {
            return Err("--hz must be between 1 and 240".to_string());
        }
        Ok(options)
    }

    /// Resolves the scenario from its name and student count.
    pub fn scenario(&self) -> Result<Scenario, String> {
        match self.scenario_name.as_str() {
            "grid" => Ok(Scenario::Grid {
                students: self.students,
            }),
            "school_day" => Ok(Scenario::SchoolDay {
                students: self.students,
            }),
            other => Err(format!(
                "unknown scenario: {other} (expected grid or school_day)"
            )),
        }
    }
}

/// Parses an integer option with a message that names the offending flag.
fn parse_number<T: std::str::FromStr>(key: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("invalid value for {key}: {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_bench() {
        let options = Options::parse(Vec::<String>::new()).unwrap();
        assert_eq!(options.command, Command::Bench);
        assert_eq!(options.hz, 60);
    }

    #[test]
    fn parses_subcommand_and_options() {
        let options = Options::parse([
            "record",
            "--scenario=school_day",
            "--students=10",
            "--ticks=120",
            "--out=run.json",
        ])
        .unwrap();
        assert_eq!(options.command, Command::Record);
        assert_eq!(options.students, 10);
        assert_eq!(options.ticks, 120);
        assert_eq!(options.path.as_deref(), Some("run.json"));
        assert!(matches!(
            options.scenario().unwrap(),
            Scenario::SchoolDay { students: 10 }
        ));
    }

    #[test]
    fn help_short_circuits_parsing() {
        assert!(Options::parse(["--help"]).unwrap().help);
    }

    #[test]
    fn rejects_unknown_command_and_option() {
        assert!(Options::parse(["frobnicate"]).is_err());
        assert!(Options::parse(["--nope=1"]).is_err());
    }

    #[test]
    fn rejects_missing_equals_and_bad_numbers() {
        assert!(Options::parse(["--frames"]).is_err());
        assert!(Options::parse(["--frames=abc"]).is_err());
    }

    #[test]
    fn rejects_out_of_range_hz() {
        assert!(Options::parse(["--hz=0"]).is_err());
        assert!(Options::parse(["--hz=1000"]).is_err());
    }

    #[test]
    fn rejects_unknown_scenario() {
        let options = Options::parse(["--scenario=nope"]).unwrap();
        assert!(options.scenario().is_err());
    }
}
