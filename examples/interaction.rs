fn main() {
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

        let mut ui_delta = manager.add_ranged_value("delta".to_string(), 0.0, -1.0, 1.0);
        let mut ui_dim = manager.add_ranged_value("dimension".to_string(), 0, 0, 2);
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
}
