use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Instant;

use neuro_engine::{GateBExperimentConfig, run_gate_b_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = GateBExperimentConfig::default();
    let mut output = PathBuf::from("app/public/gate-b-v1.2.json");
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--quick" {
            config.model_seed_count = 3;
            config.training_episodes = 240;
            config.evaluation_episodes = 40;
            config.curve_window = 20;
        } else if argument == "--output" {
            output = args
                .next()
                .map(PathBuf::from)
                .ok_or("missing --output path")?;
        } else {
            return Err(format!("unknown argument: {}", argument.to_string_lossy()).into());
        }
    }

    let started = Instant::now();
    let result = run_gate_b_experiment(config)?;
    let elapsed = started.elapsed();
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    serde_json::to_writer(BufWriter::new(File::create(&output)?), &result)?;

    for report in &result.reports {
        let accuracy = report.metrics.correct_choice_fraction;
        println!(
            "{:?}/{:?} accuracy={:.3} ci=[{:.3}, {:.3}] branch={:.3} food={:.3}",
            report.condition,
            report.controller,
            accuracy.mean,
            accuracy.lower95,
            accuracy.upper95,
            report.metrics.branch_choice_fraction.mean,
            report.metrics.food_fraction.mean,
        );
    }
    for effect in &result.paired_effects {
        let accuracy = effect.effect.correct_choice_fraction;
        println!(
            "{} effect={:.3} ci=[{:.3}, {:.3}]",
            effect.id, accuracy.mean, accuracy.lower95, accuracy.upper95,
        );
    }
    println!("acceptance={:?}", result.acceptance);
    println!("benchmark_elapsed={elapsed:?} (not part of deterministic report)");
    println!("wrote {}", output.display());
    if config.model_seed_count >= 8 && !result.acceptance.passed {
        return Err("Gate B acceptance did not pass; report the negative result".into());
    }
    Ok(())
}
