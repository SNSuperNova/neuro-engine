use std::{env, fs, path::PathBuf};

use neuro_engine::{M1NEConfig, run_m1ne_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = M1NEConfig::default();
    if quick {
        config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .development_seed_count = 1;
        config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .confirmation_seed_count = 1;
        config.m1xe_protocol.m1x_protocol.oracle_iterations = 20;
        config.m1xe_protocol.m1x_protocol.oracle_training_trials = 8;
        config.m1xe_protocol.m1x_protocol.oracle_restart_count = 1;
        config
            .m1xe_protocol
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
                PathBuf::from("app/public/norm-enabled-sufficiency-development.json")
            } else {
                PathBuf::from("app/public/norm-enabled-sufficiency-v1.1.json")
            }
        });
    let result = run_m1ne_experiment(config).expect("M1-NE sufficiency experiment");
    for conclusion in &result.conclusions {
        println!("{conclusion}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result.published()).expect("serialize published M1-NE"),
    )
    .expect("write M1-NE artifact");
    println!("wrote {}", output.display());
    if let Some(raw_output) = argument(&args, "--raw-output").map(PathBuf::from) {
        if let Some(parent) = raw_output.parent() {
            fs::create_dir_all(parent).expect("create raw output directory");
        }
        fs::write(
            &raw_output,
            serde_json::to_string_pretty(&result).expect("serialize raw M1-NE"),
        )
        .expect("write raw M1-NE artifact");
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
