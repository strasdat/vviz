fn main() {
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let w3d = manager.add_widget3("w3d".to_string());
        w3d.place_entity("cube".to_string(), vviz::entities::colored_cube(1.0));
        loop {
            manager.sync_with_gui();
        }
    });
}
