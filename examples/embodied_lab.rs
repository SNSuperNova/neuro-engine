use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use neuro_engine::{EmbodiedExperimentConfig, run_embodied_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = EmbodiedExperimentConfig::default();
    let mut output = PathBuf::from("app/public/embodied-v1.json");
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--quick" {
            config.training_episodes = 120;
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

    let result = run_embodied_experiment(config)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    serde_json::to_writer(BufWriter::new(File::create(&output)?), &result)?;
    for evaluation in &result.evaluations {
        println!(
            "{} food={:.3} energy={:.3} survival={:.1} completion={:.3}",
            evaluation.label,
            evaluation.mean_foods_eaten,
            evaluation.mean_final_energy,
            evaluation.mean_steps_survived,
            evaluation.completion_fraction,
        );
    }
    println!(
        "weights={}/{} rms_change={:.5} acceptance={:?}",
        result.plasticity.changed_weight_count,
        result.plasticity.total_weight_count,
        result.plasticity.root_mean_square_change,
        result.acceptance,
    );
    println!("wrote {}", output.display());
    if !result.acceptance.passed {
        return Err("embodied learning acceptance did not pass".into());
    }
    Ok(())
}
