<div align="center">
  <h1>Plext</h1>
</div>
<div align="center">
  <a href="https://github.com/223230/plext/actions/workflows/test.yml"><img src="https://github.com/223230/plext/actions/workflows/test.yml/badge.svg"></a>
  <a href="https://github.com/223230/plext/actions/workflows/docs.yml"><img src="https://github.com/223230/plext/actions/workflows/docs.yaml/badge.svg"></a>
</div>
<br/>
<div align="center">
  Views and associated data structures for plug-in UIs made using <a href="https://github.com/robbert-vdh/nih-plug">nih-plug</a> and <a href="https://github.com/vizia/vizia">VIZIA</a>.
</div>

<hr/>

## 🧰 What's (going to be) included

Consider this a "roadmap" or to-do list. Currently, only the checked items are
included. Do you think something's missing? File a feature request so it can be
added!

### 📊 Visualizers

**General/Utility**
- [x] Grid backdrop

**Peak/Waveform Analysis**
  - [ ] Peak meter
  - [x] Peak graph
  - [x] Oscilloscope
  - [x] Static waveform

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
- **WaveformBuffer** - A buffer for waveform analysis
- **PeakBuffer** - A buffer for peak analysis

## 📃 Needing documentation

**utils**
- RingBuffer *(needs review)*
- WaveformBuffer *(needs review)*
- PeakBuffer

**visualizers**
- Oscilloscope *(needs review)*
- PeakGraph
- Waveform
- Grid
