fn main() {
    use clap::Parser;
    let args = vviz::app::Args::parse();

    vviz::app::spawn(args.mode, |mut manager: vviz::manager::Manager| {
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
}
