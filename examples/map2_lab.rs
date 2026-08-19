use std::{env, fs, path::PathBuf};

use neuro_engine::{Map2AExperimentConfig, run_map2a_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let development = args.iter().any(|arg| arg == "--development");
    let mut config = Map2AExperimentConfig::default();
    if development {
        config.map1_protocol.development_config_count = 16;
        config.map1_protocol.development_seed_count = 4;
        config.map1_protocol.confirmation_seed_count = 6;
        config.map1_protocol.confirmation_candidate_count = 4;
        config.map1_protocol.probe_protocol.pretraining_episodes = 160;
        config.map1_protocol.probe_protocol.adaptation_episodes = 120;
        config.map1_protocol.probe_protocol.evaluation_episodes = 40;
    }
    if quick {
        config.map1_protocol.development_config_count = 4;
        config.map1_protocol.development_seed_count = 2;
        config.map1_protocol.confirmation_seed_count = 2;
        config.map1_protocol.confirmation_candidate_count = 3;
        config.map1_protocol.probe_protocol.pretraining_episodes = 40;
        config.map1_protocol.probe_protocol.adaptation_episodes = 40;
        config.map1_protocol.probe_protocol.evaluation_episodes = 20;
        config.map1_protocol.probe_protocol.threshold_check_interval = 10;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick || development {
                PathBuf::from("app/public/map2a-development.json")
            } else {
                PathBuf::from("app/public/map2a-v0.3.json")
            }
        });
    let result = run_map2a_experiment(config).expect("Map 2A experiment");
    println!(
        "development={} confirmation={} additive={} controls={} stable_region={:?}",
        result.development_seed_results.len(),
        result.confirmation_seed_results.len(),
        result.additive_seed_results.len(),
        result.control_seed_results.len(),
        result.stable_region_parameter_ids,
    );
    for line in &result.conclusions {
        println!("{line}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create Map 2A output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Map 2A result"),
    )
    .expect("write Map 2A result");
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
