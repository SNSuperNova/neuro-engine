use std::{env, fs, path::PathBuf};

use neuro_engine::{Map2CExperimentConfig, run_map2c_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let development = args.iter().any(|arg| arg == "--development");
    let mut config = Map2CExperimentConfig::default();
    if development {
        let protocol = &mut config.map2b_protocol.map2a_protocol.map1_protocol;
        protocol.development_config_count = 16;
        protocol.development_seed_count = 4;
        protocol.confirmation_seed_count = 6;
        protocol.confirmation_candidate_count = 4;
        protocol.probe_protocol.pretraining_episodes = 160;
        config.stream_trial_count = 480;
        config.minimum_change_gap = 24;
        config.change_probability = 0.035;
        config.settling_window = 16;
        config.minimum_rule_changes = 4;
        config.minimum_region_size = 3;
    }
    if quick {
        let protocol = &mut config.map2b_protocol.map2a_protocol.map1_protocol;
        protocol.development_config_count = 4;
        protocol.development_seed_count = 2;
        protocol.confirmation_seed_count = 2;
        protocol.confirmation_candidate_count = 3;
        protocol.probe_protocol.pretraining_episodes = 40;
        config.stream_trial_count = 160;
        config.minimum_change_gap = 15;
        config.change_probability = 0.05;
        config.settling_window = 10;
        config.minimum_rule_changes = 2;
        config.minimum_region_size = 2;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick || development {
                PathBuf::from("app/public/map2c-development.json")
            } else {
                PathBuf::from("app/public/map2c-v0.5.json")
            }
        });
    let result = run_map2c_experiment(config).expect("Map 2C experiment");
    println!(
        "development={} confirmation={} reset={} no_supply={} frozen={} stable_region={:?}",
        result.development_seed_results.len(),
        result.confirmation_seed_results.len(),
        result.reset_seed_results.len(),
        result.no_supply_seed_results.len(),
        result.frozen_seed_results.len(),
        result.stable_region_parameter_ids,
    );
    for line in &result.conclusions {
        println!("{line}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create Map 2C output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Map 2C result"),
    )
    .expect("write Map 2C result");
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
