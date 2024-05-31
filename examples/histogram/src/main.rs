use histogram::HistogramPlugin;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<HistogramPlugin>();
}
