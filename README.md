# vviz
[![Latest version](https://img.shields.io/crates/v/vviz.svg)](https://crates.io/crates/vviz)
[![Documentation](https://docs.rs/vviz/badge.svg)](https://docs.rs/vviz)
[![Continuous integration](https://github.com/strasdat/vviz/actions/workflows/ci.yml/badge.svg)](https://github.com/strasdat/vviz/actions/workflows/ci.yml)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Rapid prototyping GUI, and visual printf-style debugging for computer vision development.

Its core dependencies are [egui](https://github.com/emilk/egui) and 
[Miniquad](https://github.com/not-fl3/miniquad). For a full list of dependencies, please inspect the
[Cargo.toml](Cargo.toml) file.


## Getting started

### Placing a 3d entity in the main panel.


```rust, no_run
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let w3d = manager.add_widget3("w3d".to_string());
        w3d.place_entity_at(
            "cube".to_string(),
            vviz::entities::colored_cube(1.0),
            vviz::math::rot_x(0.7),
        );
        loop {
            manager.sync_with_gui();
        }
    });
```

![simple example image](https://media.giphy.com/media/EcUl5vMa7prRt8gO6I/giphy.gif)

### Interacting with ranged values (sliders) and enums (combo boxes).


```rust, no_run
    #[derive(
        Clone,
        Debug,
        PartialEq,
        strum_macros::Display,
        strum_macros::EnumString,
        strum_macros::EnumVariantNames,
    )]
    enum Manipulation {
        Position,
        Orientation,
    }

    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let w3d = manager.add_widget3("w3d".to_string());

        w3d.place_entity("cube".to_string(), vviz::entities::colored_cube(1.0));
        let mut scene_pose_entity = nalgebra::Isometry3::<f32>::identity();

        let mut ui_delta = manager.add_ranged_value("delta".to_string(), 0.0, (-1.0, 1.0));
        let mut ui_dim = manager.add_ranged_value("dimension".to_string(), 0, (0, 2));
        let mut ui_manipulation =
            manager.add_enum("manipulation".to_string(), Manipulation::Position);

        loop {
            if ui_delta.get_new_value().is_some()
                || ui_dim.get_new_value().is_some()
                || ui_manipulation.get_new_value().is_some()
            {
                let delta = ui_delta.get_value();
                let manipulation = ui_manipulation.get_value();
                match manipulation {
                    Manipulation::Position => {
                        scene_pose_entity.translation.vector[ui_dim.get_value()] = delta;
                    }
                    Manipulation::Orientation => {
                        let mut scaled_axis = nalgebra::Vector3::zeros();
                        scaled_axis[ui_dim.get_value()] = delta;
                        scene_pose_entity.rotation =
                            nalgebra::UnitQuaternion::<f32>::from_scaled_axis(scaled_axis);
                    }
                }
                w3d.update_scene_pose_entity("cube".to_string(), scene_pose_entity)
            }
            manager.sync_with_gui();
        }
    });
```

![interaction example gif](https://media.giphy.com/media/0tIFBhoJepwm8MmsV1/giphy.gif)


### Multiple widgets


```rust, no_run
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let w3d = manager.add_widget3("w3d".to_string());
        w3d.place_entity_at(
            "cube".to_string(),
            vviz::entities::colored_cube(0.5),
            nalgebra::Isometry3::<f32>::translation(0.0, 0.75, 0.0),
        );
        w3d.place_entity_at(
            "cube2".to_string(),
            vviz::entities::colored_cube(0.5),
            nalgebra::Isometry3::<f32>::translation(0.0, -0.75, 0.0),
        );

        let w2 = manager.add_widget3("w2".to_string());
        let triangles = vec![vviz::entities::ColoredTriangle {
            face: [[2.0, -2.0, 0.0], [2.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
            color: vviz::entities::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                alpha: 1.0,
            },
        }];
        w2.place_entity(
            "triangles".to_string(),
            vviz::entities::colored_triangles(triangles),
        );
        let _w3 = manager.add_widget3("empty".to_string());

        let mut ui_a_button = manager.add_button("a button".to_string());
        loop {
            if ui_a_button.was_pressed() {
                println!("a button pressed");
            }
            manager.sync_with_gui();
        }
    });
```

![multi-widget example gif](https://media.giphy.com/media/DHM12WLKEmh1N7bUGT/giphy.gif)

### 2d widgets - to show an image

```rust, no_run
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let image: image::DynamicImage = vviz::utilities::load_image_from_url(
            "https://rustacean.net/assets/rustacean-orig-noshadow.png",
        )
        .unwrap();
        manager.add_widget2("img".to_string(), image);
        manager.sync_with_gui();
    });
    
```

![widget2 gif](https://media.giphy.com/media/MlPcFqkBh7zhVdlQT2/giphy.gif)

```rust, no_run
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let w3d = manager.add_widget3("w3d".to_string());
        w3d.place_entity_at(
            "axis".to_string(),
            vviz::entities::Axis3::from_scale(1.0).into(),
            vviz::math::rot_x(0.7),
        );

        w3d.place_entity_at(
            "points".to_string(),
            vviz::entities::ColoredPoints3::from_arrays_and_color(
                vec![
                    [0.50, 0.50, 0.5],
                    [0.25, 0.50, 0.5],
                    [0.50, 0.25, 0.5],
                    [0.25, 0.25, 0.5],
                ],
                vviz::entities::Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    alpha: 1.0,
                },
            )
            .into(),
            vviz::math::rot_x(0.7),
        );
        loop {
            manager.sync_with_gui();
        }
    });
    
```

![lines and points gif](https://media.giphy.com/media/AtBtWE7U49UKVRlQcA/giphy.gif)

## Goals
 - Enabling 2d and 3d visualization for computer vision development
 - Visual "printf-style" debugging - populating the visualizer with content
 - User interaction by binding UI components (slider, checkbox, ...) to  primitive types (f32, bool, 
   ...). 
 - Minimal API surface - users are interacting with vviz through a small set of functions and 
   structs.
 - Targeting desktop app and browser app (in v0.5.0)
 - Remote visualization (in v0.5.0)

## Non-goals
 - Full fledged GUI framework
 - Aesthetics customization - e.g. precisely define location / shape of button, slider etc.
 - Commitment to egui / miniquad for backend. The GUI and 3d rendering backend might be replaced
   before version 1.0.0 is reached.
 - Stable API before version 1.0.0.
   

## Roadmap

 - 0.1: MVP
   - [x] components: slider, button, checkbox, combobox
   - [x] multiple widgets for 3d rendering
   - [x] CI on github
   - [x] create examples folder
   - [x] README and code comments
 - **0.2: Widget2 and Widget3 additions**
   - [x] Widget2: to display image
   - [x] Widget3: add basic 3d orbital control
   - [x] Widget3: line segments and points
   - [x] start vviz book
 - 0.3: 2d overlays, improved controls & widget3 features
   - [ ] custom projective view given pinhole camera
   - [ ] 2d rendering
   - [ ] 2d image control
   - [ ] improved orbital control, using depth buffers
   - [ ] textured mesh
   - [ ] 3d phong shading option
 - 0.4: graph plotting using PlotWidget
 - 0.5: web/remote visualization, in addition to standalone lib
   - [ ] lib: vviz::Manger with websocket server
   - [ ] wasm app: vviz::Gui in browser using websocket client

## Acknowledgements

vviz is influenced by other open source projects, especially 
[Pangolin](https://github.com/stevenlovegrove/pangolin).
