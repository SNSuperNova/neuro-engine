use neuro_engine::{Phase2ExperimentConfig, run_phase2_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_phase2_experiment(Phase2ExperimentConfig::default())?;
    let summary = result.summary;

    println!("phase2.pattern=center-cross");
    println!(
        "phase2.window_ms={:.0}..{:.0}",
        result.pattern_start.as_micros() as f64 / 1_000.0,
        result.pattern_end.as_micros() as f64 / 1_000.0
    );
    println!("phase2.input_events={}", summary.pattern_input_event_count);
    println!(
        "control.mean_firing_rate_hz={:.6}",
        summary.control.mean_firing_rate_hz
    );
    println!(
        "pattern.mean_firing_rate_hz={:.6}",
        summary.pattern.mean_firing_rate_hz
    );
    println!(
        "control.active_neuron_fraction={:.6}",
        summary.control.active_neuron_fraction
    );
    println!(
        "pattern.active_neuron_fraction={:.6}",
        summary.pattern.active_neuron_fraction
    );
    println!(
        "pattern.synchronous_burst_bin_fraction={:.6}",
        summary.pattern.synchronous_burst_bin_fraction
    );
    println!(
        "trajectory.mean_rms_spike_difference={:.6}",
        summary.trajectory.mean_rms_spike_difference
    );
    println!(
        "trajectory.changed_bin_fraction={:.6}",
        summary.trajectory.changed_bin_fraction
    );
    println!("control.digest={:016x}", summary.control_digest);
    println!("pattern.digest={:016x}", summary.pattern_digest);
    Ok(())
}
