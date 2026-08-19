use std::{env, fs, path::PathBuf, time::Instant};

use neuro_engine::{GateEExperimentConfig, benchmark_gate_e_runtime, run_gate_e_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let development = args.iter().any(|arg| arg == "--development");
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = GateEExperimentConfig::default();
    if development {
        config.seed = 0x4741_5445_5f45_0101;
        config.model_seed_count = 4;
        config.training_episodes = 600;
        config.evaluation_episodes = 100;
    }
    if quick {
        config.model_seed_count = 2;
        config.training_episodes = 80;
        config.evaluation_episodes = 16;
        config.curve_window = 20;
    }
    if let Some(value) = argument(&args, "--lif-gain").and_then(|value| value.parse().ok()) {
        config.lif_input_gain = value;
    }
    if let Some(value) = argument(&args, "--trace-decay").and_then(|value| value.parse().ok()) {
        config.lif_spike_trace_decay = value;
        config.controller.hidden_leak = value;
    }
    if let Some(value) = argument(&args, "--tau-ms").and_then(|value| value.parse().ok()) {
        config.lif_membrane_time_constant_ms = value;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if development || quick {
                PathBuf::from("app/public/gate-e-development.json")
            } else {
                PathBuf::from("app/public/gate-e-v1.5.json")
            }
        });
    let started = Instant::now();
    let result = run_gate_e_experiment(config).expect("Gate E experiment");
    for report in &result.reports {
        let accuracy = report.metrics.correct_choice_fraction;
        println!(
            "delay={} controller={:?} accuracy={:.3} ci=[{:.3}, {:.3}] active={:.3} spikes/step={:.3} damage_drop={:?}",
            report.delay_steps,
            report.controller,
            accuracy.mean,
            accuracy.lower95,
            accuracy.upper95,
            report.activity.active_unit_fraction.mean,
            report.activity.emitted_spikes_per_step.mean,
            report.damage_accuracy_drop.map(|value| value.mean)
        );
    }
    for effect in &result.paired_effects {
        let value = effect.effect.correct_choice_fraction;
        println!(
            "{} effect={:.3} ci=[{:.3}, {:.3}]",
            effect.id, value.mean, value.lower95, value.upper95
        );
    }
    println!("boundaries={:?}", result.reliable_memory_boundary_steps);
    println!("budgets={:?}", result.controller_budgets);
    println!("costs={:?}", result.aggregate_costs);
    println!("acceptance={:?}", result.acceptance);
    println!(
        "benchmark_elapsed={:?} (platform-specific; excluded from deterministic report)",
        started.elapsed()
    );
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Gate E result"),
    )
    .expect("write Gate E result");
    println!("wrote {}", output.display());
    if let Some(path) = argument(&args, "--benchmark-output") {
        let benchmark = benchmark_gate_e_runtime(config, 3).expect("Gate E runtime benchmark");
        fs::write(
            path,
            serde_json::to_string_pretty(&benchmark).expect("serialize Gate E benchmark"),
        )
        .expect("write Gate E benchmark");
        println!("runtime_benchmark={benchmark:?}");
        println!("wrote {path}");
    }
    if !development && !quick && !result.acceptance.passed {
        std::process::exit(2);
    }
}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
