use neuro_engine::{
    ActivityRegulatorConfig, Phase2ExperimentConfig, Phase2ProtocolConfig, SimDuration,
    evaluate_phase2_input_stability,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seeds = [
        0x4e45_5552_4f32_0001,
        0x6c17_0b37_9b6f_61fe,
        0xb109_6f42_857d_6226,
    ];
    println!(
        "threshold,cooldown_ms,pass_fraction,severe,mean_rate_hz,mean_active,mean_synchronous"
    );
    for threshold in [17, 18, 19, 20, 21] {
        for cooldown_ms in [1, 2, 3] {
            let config = Phase2ProtocolConfig {
                base: Phase2ExperimentConfig {
                    pattern_magnitude_mv: 6.0,
                    pattern_interval: SimDuration::from_micros(50_000),
                    ..Phase2ExperimentConfig::default()
                },
                input_jitter: SimDuration::from_micros(20_000),
                activity_regulator: Some(ActivityRegulatorConfig {
                    unique_spike_threshold: threshold,
                    cooldown: SimDuration::from_micros(cooldown_ms * 1_000),
                    ..ActivityRegulatorConfig::default()
                }),
                ..Phase2ProtocolConfig::default()
            };
            let evaluation = evaluate_phase2_input_stability(config, &seeds, 10)?;
            println!(
                "{},{},{:.4},{},{:.4},{:.4},{:.4}",
                threshold,
                cooldown_ms,
                evaluation.pass_fraction,
                evaluation.severe_instability_count,
                evaluation.mean_firing_rate_hz,
                evaluation.mean_active_fraction,
                evaluation.mean_synchronous_fraction,
            );
        }
    }
    Ok(())
}
