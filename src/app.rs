//! The app entry point.

use clap::Parser;

use super::common;
use super::gui;
use super::manager;

struct App {
    to_gui_loop_receiver: Option<std::sync::mpsc::Receiver<common::ToGuiLoopMessage>>,
    from_gui_loop_sender: Option<std::sync::mpsc::Sender<common::FromGuiLoopMessage>>,
}

impl App {
    fn new() -> Self {
        App {
            to_gui_loop_receiver: None,
            from_gui_loop_sender: None,
        }
    }

    fn spawn(mut self, f: impl FnOnce(manager::Manager) + Send + 'static) {
        let (to_gui_loop_sender, to_gui_loop_receiver) = std::sync::mpsc::channel();
        self.to_gui_loop_receiver = Some(to_gui_loop_receiver);

        let (from_gui_loop_sender, from_gui_loop_receiver) = std::sync::mpsc::channel();
        self.from_gui_loop_sender = Some(from_gui_loop_sender);

        std::thread::spawn(move || {
            let manager = manager::Manager::new_local(to_gui_loop_sender, from_gui_loop_receiver);
            f(manager);
        });
        self.block_on_gui_loop();
    }

    fn block_on_gui_loop(self) {
        let conf = miniquad::conf::Conf {
            high_dpi: true,
            ..Default::default()
        };
        miniquad::start(conf, |mut ctx| {
            miniquad::UserData::owning(
                gui::GuiLoop::new(
                    &mut ctx,
                    self.to_gui_loop_receiver.unwrap(),
                    self.from_gui_loop_sender.unwrap(),
                ),
                ctx,
            )
        });
    }
}

/// Visualization mode
#[derive(Clone, Debug, PartialEq, clap::ArgEnum)]
pub enum VVizMode {
    /// Create a local GUI window and render loop for visualization.
    Local,
    /// Connect via websocket to remote app.
    Remote,
}

/// Application arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// visualization mode
    #[clap(short, long, arg_enum)]
    pub mode: VVizMode,
}

/// This spawns the application thread - which one whishes to visually/interactively debug.
///
/// Example
/// ``` no_run
/// vviz::app::spawn(|mut manager: vviz::manager::Manager| {
///     let mut ui_a_button = manager.add_button("a button".to_string());
///
///     // Some initial application logic...
///
///     loop {
///        if ui_a_button.was_pressed() {
///           println!("a button pressed");
///         }
///
///         /// repeated application logic...
///
///         manager.sync_with_gui();
///     }
/// });
/// ```
pub fn spawn(mode: VVizMode, f: impl FnOnce(manager::Manager) + Send + 'static) {
    match mode {
        VVizMode::Local => {
            let vviz = App::new();
            vviz.spawn(f);
        }
        VVizMode::Remote => {
            let manager = manager::Manager::new_remote();
            f(manager);
        }
    }
}
