use std::{env, fs, path::PathBuf};

use neuro_engine::{Map2BExperimentConfig, run_map2b_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let development = args.iter().any(|arg| arg == "--development");
    let mut config = Map2BExperimentConfig::default();
    if development {
        let protocol = &mut config.map2a_protocol.map1_protocol;
        protocol.development_config_count = 16;
        protocol.development_seed_count = 4;
        protocol.confirmation_seed_count = 6;
        protocol.confirmation_candidate_count = 4;
        protocol.probe_protocol.pretraining_episodes = 160;
        protocol.probe_protocol.adaptation_episodes = 120;
        protocol.probe_protocol.evaluation_episodes = 40;
    }
    if quick {
        let protocol = &mut config.map2a_protocol.map1_protocol;
        protocol.development_config_count = 4;
        protocol.development_seed_count = 2;
        protocol.confirmation_seed_count = 2;
        protocol.confirmation_candidate_count = 3;
        protocol.probe_protocol.pretraining_episodes = 40;
        protocol.probe_protocol.adaptation_episodes = 40;
        protocol.probe_protocol.evaluation_episodes = 20;
        protocol.probe_protocol.threshold_check_interval = 10;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick || development {
                PathBuf::from("app/public/map2b-development.json")
            } else {
                PathBuf::from("app/public/map2b-v0.4.json")
            }
        });
    let result = run_map2b_experiment(config).expect("Map 2B experiment");
    println!(
        "development={} confirmation={} no_resource={} no_supply={} controls={} stable_region={:?}",
        result.development_seed_results.len(),
        result.confirmation_seed_results.len(),
        result.no_resource_seed_results.len(),
        result.no_supply_seed_results.len(),
        result.control_seed_results.len(),
        result.stable_region_parameter_ids,
    );
    for line in &result.conclusions {
        println!("{line}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create Map 2B output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Map 2B result"),
    )
    .expect("write Map 2B result");
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
