use nih_plug::prelude::*;
use peak_graph::PeakGraphPlugin;

fn main() {
    nih_export_standalone::<PeakGraphPlugin>();
}
