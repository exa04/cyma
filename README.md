# Plext

Views and associated data structures for plug-in UIs made using
[nih-plug](https://github.com/robbert-vdh/nih-plug) and
[VIZIA](https://github.com/vizia/vizia).

Plext contains many views that you can use to quickly make good-looking plug-in
UIs using VIZIA.

## What's (going to be) included

Consider this a "roadmap" or to-do list. Currently, only the checked items are
included. Do you think something's missing? File a feature request so it can be
added!

### 📊 Visualizers

**Peak/Waveform Analysis**
  - [ ] Peak meter
  - [ ] Peak graph
  - [x] Oscilloscope
  - [ ] Static waveform

**Loudness Analysis**
  - [ ] Loudness meter
  - [ ] Loudness graph

**Stereo Monitoring**
  - [ ] Stereo monitor
  - [ ] Spectral stereo monitor

**Spectral Analysis**
  - [ ] Spectrogram
  - [ ] Ceptral / Spectral envelope

### 🎛️ Controls

**Primitive Controls**
  - [ ] Button
  - [ ] Toggle button
  - [ ] Switch
  - [ ] Drop-down menu

**Ranged Inputs**
  - [ ] Knob
  - [ ] Slider
  - [ ] Overlay Slider
  - [ ] XY-Pad

**Visual Editors**
  - [ ] Envelope editor
  - [ ] Filter display / editor
  - [ ] EQ

### 🛠️ Utils

- **RingBuffer** - A generic circular buffer
- **MaximaBuffer** - A buffer for waveform analysis
- **PeakBuffer** - A buffer for peak analysis

## Needing documentation
**utils**
- RingBuffer *(needs review)*
- MaximaBuffer *(needs review)*
- PeakBuffer

**visualizers**
- Oscilloscope *(needs review)*