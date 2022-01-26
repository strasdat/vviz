fn main() {
    vviz::app::spawn(|mut manager: vviz::manager::Manager| {
        let image: image::DynamicImage = vviz::utilities::load_image_from_url(
            "https://rustacean.net/assets/rustacean-orig-noshadow.png",
        )
        .unwrap();
        manager.add_widget2("img".to_string(), image.into_rgba8());
        manager.sync_with_gui();
    });
}
