use neuro_engine::{Gate2ExperimentConfig, Gate2Summary, run_gate2_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_gate2_experiment(Gate2ExperimentConfig::default())?;
    print_summary(&result.summary);
    Ok(())
}

fn print_summary(summary: &Gate2Summary) {
    println!("experiment-001/v1");
    println!("no_drive_hz={:.6}", summary.no_drive.mean_firing_rate_hz);
    println!(
        "no_drive_active={:.6}",
        summary.no_drive.active_neuron_fraction
    );
    println!(
        "no_drive_last_spike_ms={}",
        optional(summary.no_drive_last_spike_ms)
    );
    println!("driven_hz={:.6}", summary.driven.mean_firing_rate_hz);
    println!("driven_active={:.6}", summary.driven.active_neuron_fraction);
    println!(
        "driven_excitatory_arrivals={}",
        summary.driven.excitatory_arrival_count
    );
    println!(
        "driven_inhibitory_arrivals={}",
        summary.driven.inhibitory_arrival_count
    );
    println!(
        "driven_ignored_arrivals={}",
        summary.driven.ignored_arrival_count
    );
    println!(
        "driven_sync_bin_fraction={:.6}",
        summary.driven.synchronous_burst_bin_fraction
    );
    println!(
        "driven_sync_bursts={}",
        summary.driven.synchronous_burst_count
    );
    println!(
        "driven_max_silence_ms={:.6}",
        summary.driven.maximum_silence_ms
    );
    println!(
        "driven_periodicity={:.6}",
        summary.driven.peak_autocorrelation
    );
    println!(
        "driven_period_lag_ms={}",
        optional(summary.driven.peak_autocorrelation_lag_ms)
    );
    println!(
        "withdrawal_hz={:.6}",
        summary.withdrawal.mean_firing_rate_hz
    );
    println!(
        "withdrawal_last_spike_ms={}",
        optional(summary.withdrawal_last_spike_after_stop_ms)
    );
    println!(
        "trajectory_mean_rms={:.6}",
        summary.stimulus_trajectory.mean_rms_spike_difference
    );
    println!(
        "trajectory_max_rms={:.6}",
        summary.stimulus_trajectory.maximum_rms_spike_difference
    );
    println!(
        "trajectory_changed_bins={:.6}",
        summary.stimulus_trajectory.changed_bin_fraction
    );
    println!("no_drive_digest={:016x}", summary.no_drive_digest);
    println!("driven_digest={:016x}", summary.driven_digest);
    println!(
        "stimulus_control_digest={:016x}",
        summary.stimulus_control_digest
    );
    println!(
        "stimulus_variant_digest={:016x}",
        summary.stimulus_variant_digest
    );
}

fn optional(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "none".to_owned())
}
