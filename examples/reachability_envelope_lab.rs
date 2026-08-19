use std::{env, fs, path::PathBuf};

use neuro_engine::{M1XEConfig, run_m1xe_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = M1XEConfig::default();
    if quick {
        config.m1x_protocol.m1_protocol.development_seed_count = 1;
        config.m1x_protocol.m1_protocol.confirmation_seed_count = 1;
        config.m1x_protocol.oracle_iterations = 20;
        config.m1x_protocol.oracle_training_trials = 8;
        config.m1x_protocol.oracle_restart_count = 1;
        config
            .m1x_protocol
            .m1_protocol
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
                PathBuf::from("app/public/reachability-envelope-development.json")
            } else {
                PathBuf::from("app/public/reachability-envelope-v1.0.json")
            }
        });
    let result = run_m1xe_experiment(config).expect("M1-XE envelope experiment");
    for conclusion in &result.conclusions {
        println!("{conclusion}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result.published()).expect("serialize published M1-XE"),
    )
    .expect("write M1-XE artifact");
    println!("wrote {}", output.display());
    if let Some(raw_output) = argument(&args, "--raw-output").map(PathBuf::from) {
        if let Some(parent) = raw_output.parent() {
            fs::create_dir_all(parent).expect("create raw output directory");
        }
        fs::write(
            &raw_output,
            serde_json::to_string_pretty(&result).expect("serialize raw M1-XE"),
        )
        .expect("write raw M1-XE artifact");
        println!("wrote raw {}", raw_output.display());
    }
    if !quick && !result.acceptance.passed {
        std::process::exit(2);
    }
}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
