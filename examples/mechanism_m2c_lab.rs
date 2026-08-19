use std::{env, fs, path::PathBuf};

use neuro_engine::{M2CExperimentConfig, run_m2c_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = M2CExperimentConfig::default();
    if quick {
        config.m1_protocol.development_seed_count = 1;
        config.m1_protocol.confirmation_seed_count = 2;
        config.m1_protocol.phase_trial_count = 48;
        config.m1_protocol.evaluation_trial_count = 16;
        config.m1_protocol.threshold_check_interval = 8;
        config.candidate_rewiring_intervals = [4, 8, 12, 16];
        config
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
                PathBuf::from("app/public/mechanism-m2c-development.json")
            } else {
                PathBuf::from("app/public/mechanism-m2c-v0.3.json")
            }
        });
    let result = run_m2c_experiment(config).expect("M2C experiment");
    for conclusion in &result.conclusions {
        println!("{conclusion}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create M2C output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize M2C result"),
    )
    .expect("write M2C result");
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
