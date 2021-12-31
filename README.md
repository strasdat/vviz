# vviz
[![Latest version](https://img.shields.io/crates/v/vviz.svg)](https://crates.io/crates/vviz)
[![Documentation](https://docs.rs/vviz/badge.svg)](https://docs.rs/vviz)
[![Continuous integration](https://github.com/strasdat/vviz/actions/workflows/ci.yml/badge.svg)](https://github.com/strasdat/vviz/actions/workflows/ci.yml)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Rapid prototyping GUI, and visual printf-style debugging for computer vision development.

# Roadmap

 - 0.1: MVP
   - [x] components: slider, button, checkbox, combobox
   - [ ] multiple widgets for 3d rendering
   - [x] CI on github
   - [ ] create example folder
   - [ ] README and code comments
 - 0.2: Widget2 and Widget3 additions
   * Widget2: to display image
   * Widget3: add basic 3d orbital control
   * Widget3: line segments and points
   * start vviz book
 - 0.3: 2d overlays, improved controls
   * custom projective view given pinhole camera
   * 2d rendering
   * 2d image control
   * improved orbital control, using depth buffers
   * 3d phong shading option
 - 0.4: graph plotting using PlotWidget
 - 0.5: web/remote visualization, in addition to standalone lib
   * lib: vviz::Manger with websocket server
   * wasm app: vviz::Gui in browser using websocket client

# Acknowledgements

vviz is influenced by other open source projects, especially [Pangolin](https://github.com/stevenlovegrove/pangolin).
