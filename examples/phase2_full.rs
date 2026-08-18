use std::env;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use neuro_engine::{Phase2ProtocolConfig, run_phase2_protocol};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let mut config = Phase2ProtocolConfig::default();
    let mut output = PathBuf::from("target/phase2-v1");
    while let Some(argument) = args.next() {
        if argument == "--quick" {
            config.structure_seed_count = 1;
            config.trials_per_pattern = 6;
            config.training_trials_per_pattern = 4;
            config.single_pixel_trials = 1;
            config.functional_group_size = 3;
            config.label_permutation_count = 5;
            config.connection_swap_multiplier = 1;
        } else if argument == "--one-seed" {
            config.structure_seed_count = 1;
        } else if argument == "--group-size" {
            config.functional_group_size = args
                .next()
                .ok_or("missing --group-size value")?
                .to_string_lossy()
                .parse()?;
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
    fs::create_dir_all(&output)?;
    serde_json::to_writer_pretty(
        BufWriter::new(File::create(output.join("report.json"))?),
        &result.report,
    )?;
    serde_json::to_writer(
        BufWriter::new(File::create(output.join("responses.json"))?),
        &result.trials,
    )?;

    println!("version={}", result.report.version);
    println!(
        "selected_structure_seeds={:?}",
        result.report.selected_structure_seeds
    );
    println!(
        "rejected_structure_seed_count={}",
        result.report.rejected_structure_seed_count
    );
    for seed in &result.report.seeds {
        println!(
            "seed={:016x} stability={:.3} prototype={:.3} linear={:.3} permutation_p={:.4} separation={:.3} causal_groups={}/{}",
            seed.structure_seed,
            seed.stability_pass_fraction,
            seed.readout.prototype.accuracy,
            seed.readout.linear.accuracy,
            seed.readout.permutation_p_value,
            seed.readout.separation_ratio,
            seed.ablations
                .iter()
                .filter(|ablation| ablation.selective_effect)
                .count(),
            seed.ablations.len(),
        );
    }
    println!(
        "hypotheses=H1:{} H2:{} H3:{}",
        result.report.hypotheses.h1_reproducible_response,
        result.report.hypotheses.h2_decodable_information,
        result.report.hypotheses.h3_causal_functional_groups,
    );
    println!("wrote {}", output.display());
    Ok(())
}
