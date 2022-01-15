//! The GUI loop implementation details.

use std::sync::mpsc;

use super::common;

/// [super::common::Component]s of the side-panel and [super::common::Widget]s of the main panel.
pub struct GuiData {
    /// List of components such as buttons, sliders etc.
    pub components: linked_hash_map::LinkedHashMap<String, Box<dyn common::Component>>,
    /// List of widgets such as 3d widgets.
    pub widgets: linked_hash_map::LinkedHashMap<String, Box<dyn common::Widget>>,
}

impl Default for GuiData {
    fn default() -> Self {
        Self {
            components: linked_hash_map::LinkedHashMap::new(),
            widgets: linked_hash_map::LinkedHashMap::new(),
        }
    }
}

/// Structure which holds data for main gui loop.
pub struct GuiLoop {
    egui_mq: egui_miniquad::EguiMq,
    to_gui_loop_receiver: mpsc::Receiver<Box<dyn common::ToGuiLoopMessage>>,
    from_gui_loop_sender: mpsc::Sender<Box<dyn common::FromGuiLoopMessage>>,
    data: GuiData,
}

impl GuiLoop {
    /// Creates `GuiLoop` given `miniquad::Context` and sender/receiver structs.
    pub fn new(
        ctx: &mut miniquad::Context,
        to_gui_loop_receiver: mpsc::Receiver<Box<dyn common::ToGuiLoopMessage>>,
        from_gui_loop_sender: mpsc::Sender<Box<dyn common::FromGuiLoopMessage>>,
    ) -> GuiLoop {
        GuiLoop {
            egui_mq: egui_miniquad::EguiMq::new(ctx),
            to_gui_loop_receiver,
            from_gui_loop_sender,
            data: GuiData::default(),
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
            egui::SidePanel::left("ver").show(egui_ctx, |ui| {
                for (label, var) in &mut self.data.components {
                    var.show(label, ui, &mut self.from_gui_loop_sender);
                }
            });

            egui::CentralPanel::default().show(egui_ctx, |ui0| {
                if self.data.widgets.is_empty() {
                    return;
                }
                // the 95% here is a slight hack. This is to leave some buffer of the few pixel
                // borders between the widgets/images.
                let available_width: f32 = 0.95 * ui0.available_width();
                let available_height: f32 = 0.95 * ui0.available_height();

                let mut aspect_ratios = std::vec::Vec::with_capacity(self.data.widgets.len());
                for (_, widget) in &mut self.data.widgets {
                    aspect_ratios.push(widget.aspect_ratio());
                }
                aspect_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let n = aspect_ratios.len();
                let median_aspect_ratio = if n % 2 == 1 {
                    aspect_ratios[n / 2]
                } else {
                    0.5 * aspect_ratios[n / 2] + 0.5 * aspect_ratios[n / 2 + 1]
                };

                let mut max_width = 0.0;
                let mut max_height = 0.0;

                for num_cols in 1..=n {
                    let num_rows: f32 = ((n as f32) / (num_cols as f32)).ceil();

                    let w: f32 = available_width / (num_cols as f32);
                    let h = (w / median_aspect_ratio).min(available_height / num_rows);
                    let w = median_aspect_ratio * h;
                    if w > max_width {
                        max_width = w;
                        max_height = h;
                    }
                }
                println!("{}",   egui_ctx.input().pointer.primary_down());


                ui0.horizontal_wrapped(|ui| {
                    for (_, widget) in &mut self.data.widgets {
                        let opt = widget.show(ui, max_width, max_height);
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
                            //println!("{} {}", hp.unwrap().x, hp.unwrap().y);
                        }
                    }
                });
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
