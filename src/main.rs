use std::io;

use tracker_tui::{TuiTrackerApp, osc_synths::PolyphonicOscSynth};

fn main() -> io::Result<()> {
    let allowed_instruments = vec![
        PolyphonicOscSynth::sine_specification(),
        PolyphonicOscSynth::saw_specification(),
        PolyphonicOscSynth::square_specification(),
    ];

    ratatui::run(|terminal| TuiTrackerApp::new(allowed_instruments).run(terminal))
}
