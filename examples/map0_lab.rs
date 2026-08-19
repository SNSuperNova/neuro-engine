use std::{env, fs, path::PathBuf};

use neuro_engine::{Map0ExperimentConfig, run_map0_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let development = args.iter().any(|arg| arg == "--development");
    let mut config = Map0ExperimentConfig::default();
    if development {
        config.development_config_count = 16;
        config.development_seed_count = 4;
        config.confirmation_seed_count = 6;
        config.confirmation_candidate_count = 4;
        config.pretraining_episodes = 160;
        config.adaptation_episodes = 120;
        config.evaluation_episodes = 40;
        config.threshold_check_interval = 20;
    }
    if quick {
        config.development_config_count = 4;
        config.development_seed_count = 2;
        config.confirmation_seed_count = 2;
        config.confirmation_candidate_count = 3;
        config.pretraining_episodes = 40;
        config.adaptation_episodes = 40;
        config.evaluation_episodes = 20;
        config.threshold_check_interval = 10;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick || development {
                PathBuf::from("app/public/map0-development.json")
            } else {
                PathBuf::from("app/public/map0-v0.1.json")
            }
        });
    let result = run_map0_experiment(config).expect("Map 0 experiment");
    println!(
        "development={} confirmation={} controls={} stable_region={:?}",
        result.development_seed_results.len(),
        result.confirmation_seed_results.len(),
        result.control_seed_results.len(),
        result.stable_region_parameter_ids,
    );
    for line in &result.conclusions {
        println!("{line}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create Map 0 output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Map 0 result"),
    )
    .expect("write Map 0 result");
    println!("wrote {}", output.display());
    if !result.acceptance.passed {
        std::process::exit(2);
    }
}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
