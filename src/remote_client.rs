use vviz::common::FromGuiLoopMessage;

fn main() {
    let (to_gui_loop_sender, to_gui_loop_receiver) = std::sync::mpsc::channel();

    let (from_gui_loop_sender, from_gui_loop_receiver) = std::sync::mpsc::channel();

    let conf = miniquad::conf::Conf {
        high_dpi: true,
        ..Default::default()
    };

    std::thread::spawn(move || {
        let (mut socket, _response) =
            tungstenite::connect(reqwest::Url::parse("ws://localhost:9001").unwrap())
                .expect("Can't connect");

        loop {
            let collection: Vec<FromGuiLoopMessage> = from_gui_loop_receiver.try_iter().collect();
            socket
                .write_message(tungstenite::Message::Text(
                    serde_json::to_string(&collection).unwrap(),
                ))
                .unwrap();

            let msg = socket.read_message().expect("Error reading message");
            let to_msg: Vec<vviz::common::ToGuiLoopMessage> =
                serde_json::from_str(msg.to_text().unwrap()).unwrap();
            for m in to_msg {
                to_gui_loop_sender.send(m).unwrap();
            }

            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    });
    miniquad::start(conf, |mut ctx| {
        miniquad::UserData::owning(
            vviz::gui::GuiLoop::new(&mut ctx, to_gui_loop_receiver, from_gui_loop_sender),
            ctx,
        )
    });
}
