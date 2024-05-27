use nih_plug::prelude::*;
use histogram::HistogramPlugin;

fn main() {
    nih_export_standalone::<HistogramPlugin>();
}
