use std::sync::mpsc;

use super::common;

pub struct GuiLoop {
    egui_mq: egui_miniquad::EguiMq,
    to_gui_loop_receiver: mpsc::Receiver<Box<dyn common::ToGuiLoopMessage>>,
    from_gui_loop_sender: mpsc::Sender<Box<dyn common::FromGuiLoopMessage>>,
    data: common::GuiData,
}

impl GuiLoop {
    pub fn new(
        ctx: &mut miniquad::Context,
        to_gui_loop_receiver: mpsc::Receiver<Box<dyn common::ToGuiLoopMessage>>,
        from_gui_loop_sender: mpsc::Sender<Box<dyn common::FromGuiLoopMessage>>,
    ) -> GuiLoop {
        GuiLoop {
            egui_mq: egui_miniquad::EguiMq::new(ctx),
            to_gui_loop_receiver,
            from_gui_loop_sender,
            data: common::GuiData::default(),
        }
    }
}

impl miniquad::EventHandler for GuiLoop {
    fn update(&mut self, _ctx: &mut miniquad::Context) {}

    fn draw(&mut self, ctx: &mut miniquad::Context) {
        for m in self.to_gui_loop_receiver.try_iter() {
            m.update_gui(&mut self.data, ctx);
        }

        for (_, w) in &mut self.data.widgets {
            w.render(ctx);
        }

        self.egui_mq.run(ctx, |egui_ctx| {
            //self.side_panel();

            //let Self { egui_mq, .. } = self;

            egui::SidePanel::left("ver").show(egui_ctx, |ui| {
                for (label, var) in &mut self.data.components {
                    var.show(label, ui, &mut self.from_gui_loop_sender);
                }
            });

            egui::CentralPanel::default().show(egui_ctx, |ui0| {
                let w = ui0.available_width();
                let h = ui0.available_height();

                for (_, widget) in &mut self.data.widgets {
                    let opt = widget.show(ui0, w, h);
                    let r = opt.unwrap();
                    // println!(
                    //     "{} {} {} {}",
                    //     r.rect.center().x,
                    //     r.rect.center().y,
                    //     r.rect.width(),
                    //     r.rect.height()
                    // );
                    let hp = r.hover_pos();
                    if hp.is_some() {
                        println!("{} {}", hp.unwrap().x, hp.unwrap().y);
                    }
                }
            });
        });

        self.egui_mq.draw(ctx);

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut miniquad::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(ctx, dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        character: char,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
    ) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
