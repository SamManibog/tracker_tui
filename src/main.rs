use std::io;

use tracker_tui::TuiTrackerApp;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| TuiTrackerApp::new().run(terminal))
}
