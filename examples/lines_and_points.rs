fn main() {
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
}
