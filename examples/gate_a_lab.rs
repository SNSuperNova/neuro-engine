use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use neuro_engine::{GateAExperimentConfig, run_gate_a_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = GateAExperimentConfig::default();
    let mut output = PathBuf::from("app/public/gate-a-v1.1.json");
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--quick" {
            config.model_seed_count = 3;
            config.training_episodes = 160;
            config.evaluation_episodes = 16;
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

    let result = run_gate_a_experiment(config)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    serde_json::to_writer(BufWriter::new(File::create(&output)?), &result)?;
    for variant in &result.variants {
        println!(
            "{} learned={:.3} disabled={:.3} effect={:.3} ci=[{:.3}, {:.3}]",
            variant.id,
            variant.learned.foods_eaten.mean,
            variant.learning_disabled.foods_eaten.mean,
            variant.paired_effect.foods_eaten.mean,
            variant.paired_effect.foods_eaten.lower95,
            variant.paired_effect.foods_eaten.upper95,
        );
    }
    println!("acceptance={:?}", result.acceptance);
    println!("wrote {}", output.display());
    if config.model_seed_count >= 8 && !result.acceptance.passed {
        return Err("Gate A acceptance did not pass; report the negative result".into());
    }
    Ok(())
}
