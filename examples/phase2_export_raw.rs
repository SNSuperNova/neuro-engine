use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use neuro_engine::{Phase2ProtocolConfig, run_phase2_protocol, run_phase2_trial_artifact};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Phase2ProtocolConfig::default();
    let mut output = PathBuf::from("target/phase2-v1/raw-events.jsonl");
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--quick" {
            config.structure_seed_count = 1;
            config.trials_per_pattern = 6;
            config.training_trials_per_pattern = 4;
            config.single_pixel_trials = 1;
            config.functional_group_size = 3;
            config.label_permutation_count = 5;
            config.connection_swap_multiplier = 1;
        } else if argument == "--output" {
            output = args
                .next()
                .map(PathBuf::from)
                .ok_or("missing --output path")?;
        } else {
            return Err(format!("unknown argument: {}", argument.to_string_lossy()).into());
        }
    }

    let result = run_phase2_protocol(config)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = BufWriter::new(File::create(&output)?);
    for trial in &result.trials {
        let artifact = run_phase2_trial_artifact(
            config,
            trial.metadata.structure_seed,
            trial.metadata.pattern,
            trial.metadata.trial_index,
            trial.metadata.split,
        )?;
        if artifact.response.metadata.event_digest != trial.metadata.event_digest {
            return Err("raw event regeneration digest mismatch".into());
        }
        serde_json::to_writer(&mut writer, &artifact)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    println!(
        "wrote {} trials to {}",
        result.trials.len(),
        output.display()
    );
    Ok(())
}
