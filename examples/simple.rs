use vviz::entities;

#[derive(
    Clone,
    Debug,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
)]
enum Options {
    Foo,
    Bar,
    Daz,
}

fn main() {
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let mut ui_bool = manager.add_bool("foo".to_string(), true);
        let mut ui_red = manager.add_ranged_number("red".to_string(), 0.1, 0.0, 1.0);
        let mut ui_counter = manager.add_ranged_number("counter".to_string(), 5, -50, 50);
        let opt: Options = Options::Daz;
        let mut ui_options = manager.add_enum("options".to_string(), opt);
        let _ui_integer = manager.add_number("const int".to_string(), 42);
        let mut ui_a_button = manager.add_button("a button".to_string());

        let w3d = manager.add_widget3("w3d".to_string());
        w3d.place_entity_at(
            "cube".to_string(),
            vviz::entities::colored_cube(0.5),
            nalgebra::Isometry3::<f32>::translation(0.0, 0.0, 0.75),
        );
        w3d.place_entity_at(
            "cube2".to_string(),
            vviz::entities::colored_cube(0.5),
            nalgebra::Isometry3::<f32>::translation(0.0, 0.0, -0.75),
        );

        let w2 = manager.add_widget3("w2".to_string());
        let triangles = vec![entities::ColoredTriangle {
            face: [[0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
            color: entities::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                alpha: 1.0,
            },
        }];
        w2.place_entity("trig".to_string(), entities::colored_triangles(triangles));
        let _w3 = manager.add_widget3("empty".to_string());

        loop {
            if ui_bool.get_new_value().is_some() {
                println!("{}", ui_bool.get_value());
            }
            if ui_red.get_new_value().is_some() {
                println!("{}", ui_red.get_value());
            }
            if ui_counter.get_new_value().is_some() {
                println!("{}", ui_counter.get_value());
            }
            if ui_options.get_new_value().is_some() {
                println!("new val {}", ui_options.get_value());
            }
            if ui_a_button.was_pressed() {
                println!("a button pressed");
            }

            manager.sync_with_gui();
        }
    });
}
