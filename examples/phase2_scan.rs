use neuro_engine::{Phase2ExperimentConfig, SimDuration, run_phase2_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "magnitude_mv,interval_ms,firing_rate_hz,active_fraction,synchronous_fraction,trajectory_difference,changed_bin_fraction"
    );
    for magnitude_mv in [4.0, 6.0, 8.0, 10.0] {
        for interval_ms in [20_u64, 30, 40] {
            let config = Phase2ExperimentConfig {
                pattern_magnitude_mv: magnitude_mv,
                pattern_interval: SimDuration::from_micros(interval_ms * 1_000),
                ..Phase2ExperimentConfig::default()
            };
            let summary = run_phase2_experiment(config)?.summary;
            println!(
                "{magnitude_mv:.1},{interval_ms},{:.6},{:.6},{:.6},{:.6},{:.6}",
                summary.pattern.mean_firing_rate_hz,
                summary.pattern.active_neuron_fraction,
                summary.pattern.synchronous_burst_bin_fraction,
                summary.trajectory.mean_rms_spike_difference,
                summary.trajectory.changed_bin_fraction,
            );
        }
    }
    Ok(())
}
