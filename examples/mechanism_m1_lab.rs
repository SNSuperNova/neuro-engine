use std::{env, fs, path::PathBuf};

use neuro_engine::{M1ExperimentConfig, run_m1_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = M1ExperimentConfig::default();
    if quick {
        config.development_seed_count = 1;
        config.confirmation_seed_count = 2;
        config.phase_trial_count = 40;
        config.evaluation_trial_count = 16;
        config.threshold_check_interval = 10;
        config
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .map1_protocol
            .probe_protocol
            .pretraining_episodes = 80;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick {
                PathBuf::from("app/public/mechanism-m1-development.json")
            } else {
                PathBuf::from("app/public/mechanism-m1-v0.2.json")
            }
        });
    let result = run_m1_experiment(config).expect("M1 experiment");
    for conclusion in &result.conclusions {
        println!("{conclusion}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create M1 output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize M1 result"),
    )
    .expect("write M1 result");
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
