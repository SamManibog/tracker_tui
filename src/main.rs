use std::io;

use tracker_tui::{TuiTrackerApp, osc_synths::PolyphonicOscSynth};

fn main() -> io::Result<()> {
    let allowed_instruments = vec![
        PolyphonicOscSynth::sine_specification(),
        PolyphonicOscSynth::saw_specification(),
        PolyphonicOscSynth::square_specification(),
    ];

    //ratatui::run(|terminal| TuiTrackerApp::new(allowed_instruments).run(terminal))
    ratatui::run(|terminal| {
        tracker_tui::shift_grid::ShiftGridTest::new(40, 60, 3..=10).run(terminal)
    })
}
