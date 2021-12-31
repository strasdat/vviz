//! Common structures shared between manager and gui loop.

use super::entities;
use super::gui;

use ::slice_of_array::prelude::*;

/// Component such as a button or a slider.
pub trait Component: downcast_rs::DowncastSync {
    /// How to display the component on the side panel.
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    );
}

impl core::fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "dyn Component")
    }
}

downcast_rs::impl_downcast!(sync Component);

/// String representation of enum.
pub struct EnumStringRepr {
    /// String representation of the current value.
    pub value: String,
    /// All possible values.
    pub values: std::vec::Vec<String>,
}

impl Component for EnumStringRepr {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        let mut selected = self.value.clone();

        egui::ComboBox::from_label(label)
            .selected_text(format!("{:?}", &selected))
            .show_ui(ui, |ui| {
                for str in &mut self.values {
                    ui.selectable_value(&mut selected, str.to_string(), format!("{:?}", &str));
                }
            });

        if *self.value != selected {
            self.value = selected.to_string();
            sender
                .send(Box::new(UpdateEnumStringRepr {
                    label: label.to_string(),
                    value: self.value.clone(),
                }))
                .unwrap();
        }
    }
}

/// Variable bool (checkbox) or numeric (read-only text box).
pub struct Var<T> {
    /// Current value.
    pub value: T,
}

impl Component for Var<bool> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        if ui.checkbox(&mut self.value, label).changed() {
            sender
                .send(Box::new(UpdateValue {
                    label: label.to_string(),
                    value: self.value,
                }))
                .unwrap();
        }
    }
}

/// A button.
pub struct Button {
    /// Is true is recently pressed.
    pub pressed: bool,
}

impl Component for Button {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        if ui.button(label).clicked() {
            sender
                .send(Box::new(UpdateButton {
                    label: label.to_string(),
                }))
                .unwrap();
        }
    }
}

impl<T: Number> Component for Var<T> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        _sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        ui.label(format!("{}: {}", label, self.value));
    }
}

/// A range value, represented as slider.
pub struct RangedVar<T> {
    /// Current value.
    pub value: T,
    /// Min bound.
    pub min: T,
    /// Max bound.
    pub max: T,
}

impl<T: Number> Component for RangedVar<T> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        if ui
            .add(egui::Slider::new(&mut self.value, self.min..=self.max))
            .changed()
        {
            sender
                .send(Box::new(UpdateRangedValue {
                    label: label.to_string(),
                    value: self.value,
                }))
                .unwrap();
        }
    }
}

/// Widget for main panel.
pub trait Widget: downcast_rs::DowncastSync {
    /// How to render.
    fn render(&mut self, ctx: &mut miniquad::Context);

    /// How to display the rendered content.
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        assigned_width: f32,
        assigned_height: f32,
    ) -> Option<egui::Response>;

    /// The apect ratio of the
    fn aspect_ratio(&self) -> f32;
}

downcast_rs::impl_downcast!(sync Widget);

mod offscreen_shader {

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 pos;
    attribute vec4 color0;
    varying lowp vec4 color;
    uniform mat4 mvp;
    void main() {
        gl_Position = mvp * pos;
        color = color0;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    void main() {
        gl_FragColor = color;
    }
    "#;

    pub fn meta() -> miniquad::ShaderMeta {
        miniquad::ShaderMeta {
            images: vec![],
            uniforms: miniquad::UniformBlockLayout {
                uniforms: vec![miniquad::UniformDesc::new(
                    "mvp",
                    miniquad::UniformType::Mat4,
                )],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: nalgebra::Matrix4<f32>,
    }
}

/// Widget for 3d content such as meshes, line segments and point clouds.
pub struct Widget3 {
    entities: linked_hash_map::LinkedHashMap<String, entities::NamedEntity3>,
    offscreen_pipeline: miniquad::Pipeline,
    //offscreen_bind: miniquad::Bindings,
    offscreen_pass: miniquad::RenderPass,
    aspect_ratio: f32,
    rx: f32,
    ry: f32,
    texture_id: Option<egui::TextureId>,
}

impl Widget3 {
    fn new(ctx: &mut miniquad::Context) -> Self {
        let color_img = miniquad::Texture::new_render_texture(
            ctx,
            miniquad::TextureParams {
                width: 640,
                height: 480,
                format: miniquad::TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = miniquad::Texture::new_render_texture(
            ctx,
            miniquad::TextureParams {
                width: 640,
                height: 480,
                format: miniquad::TextureFormat::Depth,
                ..Default::default()
            },
        );

        let offscreen_pass = miniquad::RenderPass::new(ctx, color_img, depth_img);

        let offscreen_shader = miniquad::Shader::new(
            ctx,
            offscreen_shader::VERTEX,
            offscreen_shader::FRAGMENT,
            offscreen_shader::meta(),
        )
        .unwrap();

        let offscreen_pipeline = miniquad::Pipeline::with_params(
            ctx,
            &[miniquad::BufferLayout {
                stride: (3 + 4) * std::mem::size_of::<f32>() as i32,
                ..Default::default()
            }],
            &[
                miniquad::VertexAttribute::new("pos", miniquad::VertexFormat::Float3),
                miniquad::VertexAttribute::new("color0", miniquad::VertexFormat::Float4),
            ],
            offscreen_shader,
            miniquad::PipelineParams {
                depth_test: miniquad::Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        Self {
            entities: linked_hash_map::LinkedHashMap::new(),
            offscreen_pipeline,
            //offscreen_bind,
            offscreen_pass,
            aspect_ratio: 640.0 / 480.0,
            rx: 0.,
            ry: 0.,
            texture_id: None,
        }
    }
}

impl Widget for Widget3 {
    fn render(&mut self, ctx: &mut miniquad::Context) {
        let proj = nalgebra_glm::perspective_fov_rh(60.0f32.to_radians(), 640.0, 480.0, 0.01, 10.0);
        let view = nalgebra_glm::look_at_rh(
            &nalgebra_glm::vec3(0.0, 1.5, 3.0),
            &nalgebra_glm::vec3(0.0, 0.0, 0.0),
            &nalgebra_glm::vec3(0.0, 1.0, 0.0),
        );
        let view_proj = proj * view;

        self.rx += 0.01;
        self.ry += 0.03;
        let camera_pose_scene = nalgebra::Isometry3::<f32>::from_parts(
            nalgebra::Translation3::<f32>::new(0.0, 0.0, 0.0),
            nalgebra::UnitQuaternion::<f32>::from_euler_angles(self.rx, self.ry, 0.),
        );

        // the offscreen render pipeline, following this example:
        // https://github.com/not-fl3/egui-miniquad/blob/master/examples/render_to_egui_image.rs
        ctx.begin_pass(
            self.offscreen_pass,
            miniquad::PassAction::clear_color(1.0, 1.0, 1.0, 1.),
        );
        for (_, named_entity) in &self.entities {
            let vertex_buffer = miniquad::Buffer::immutable(
                ctx,
                miniquad::BufferType::VertexBuffer,
                named_entity
                    .entity
                    .vertices
                    .as_position_color()
                    .unwrap()
                    .vertices
                    .flat(),
            );

            let index_buffer = miniquad::Buffer::immutable(
                ctx,
                miniquad::BufferType::IndexBuffer,
                named_entity.entity.faces.indices.flat(),
            );

            let offscreen_bind = miniquad::Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![],
            };

            ctx.apply_pipeline(&self.offscreen_pipeline);
            ctx.apply_bindings(&offscreen_bind);

            let vs_params = offscreen_shader::Uniforms {
                mvp: view_proj
                    * camera_pose_scene.to_matrix()
                    * named_entity.scene_pose_entity.to_matrix(),
            };
            ctx.apply_uniforms(&vs_params);

            ctx.draw(0, named_entity.entity.faces.indices.flat().len() as i32, 1);
        }
        ctx.end_render_pass();

        // Extract texture from offscreen render pass
        let mq_texture = self.offscreen_pass.texture(ctx);
        //print!("{} {}", mq_texture.width, mq_texture.height);

        // create egui TextureId from Miniquad GL texture Id
        self.texture_id = Some(egui::TextureId::User(mq_texture.gl_internal_id() as u64));

        ctx.clear(Some((1., 1., 1., 1.)), None, None);
        ctx.begin_default_pass(miniquad::PassAction::clear_color(0.3, 0.3, 0.3, 1.0));
        ctx.end_render_pass();
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        assigned_width: f32,
        assigned_height: f32,
    ) -> Option<egui::Response> {
        let w = (self.aspect_ratio * assigned_height).min(assigned_width);
        let h = w / self.aspect_ratio;

        let r = ui.add(
            egui::Image::new(self.texture_id.unwrap(), egui::Vec2::new(w, h))
                .sense(egui::Sense::click_and_drag()),
        );
        Some(r)
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

/// Integer or floating point number.

pub trait Number: egui::emath::Numeric + downcast_rs::DowncastSync + std::fmt::Display {}

impl Number for i32 {}
impl Number for i64 {}
impl Number for f32 {}
impl Number for f64 {}

/// Message from  `Manager` to `GuiLoop`, such as to add a component or widget.
pub trait ToGuiLoopMessage: Send {
    /// How that component or widget shall be displayed.
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, ctx: &mut miniquad::Context);
}

/// Add an enum (as string representation) as combo box to side panel.
pub struct AddEnumStringRepr {
    /// The name of the enum.
    pub label: String,
    /// Initial value.
    pub value: String,
    /// List of possible values.
    pub values: std::vec::Vec<String>,
}

impl ToGuiLoopMessage for AddEnumStringRepr {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components.insert(
            self.label,
            Box::new(EnumStringRepr {
                value: self.value,
                values: self.values,
            }),
        );
    }
}

/// To add a button to side panel.
pub struct AddButton {
    /// The name of button.
    pub label: String,
}

impl ToGuiLoopMessage for AddButton {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Button { pressed: false }));
    }
}

/// Add bool (as checkbox) or numeric value (ad read-only text box) to side panel.
pub struct AddVar<T> {
    /// The name of variable.
    pub label: String,
    /// The initial value.
    pub value: T,
}

impl ToGuiLoopMessage for AddVar<bool> {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<bool> { value: self.value }));
    }
}

impl<T: Number> ToGuiLoopMessage for AddVar<T> {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<T> { value: self.value }));
    }
}

/// Add a numeric value as a slider to side panel.

pub struct AddRangedVar<T> {
    /// Name of variable.
    pub label: String,
    /// Initial value.
    pub value: T,
    /// Min bounds
    pub min: T,
    /// Max bounds
    pub max: T,
}

impl<T: Number> ToGuiLoopMessage for AddRangedVar<T> {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components.insert(
            self.label,
            Box::new(RangedVar::<T> {
                value: self.value,
                min: self.min,
                max: self.max,
            }),
        );
    }
}

/// Adds 3d widget to main panel.
pub struct AddWidget3 {
    /// Name of widget
    pub label: String,
}

impl ToGuiLoopMessage for AddWidget3 {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, ctx: &mut miniquad::Context) {
        data.widgets.insert(self.label, Box::new(Widget3::new(ctx)));
    }
}

/// Place3d  entity in corresponding 3d widget.
pub struct PlaceEntity {
    /// Name of widget.
    pub widget_label: String,
    /// The 3d entity
    pub named_entity: entities::NamedEntity3,
}

impl ToGuiLoopMessage for PlaceEntity {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.widgets
            .get_mut(&self.widget_label)
            .unwrap()
            .downcast_mut::<Widget3>()
            .unwrap()
            .entities
            .insert(self.named_entity.label.clone(), self.named_entity);
    }
}

/// Delete that component from side panel.
pub struct DeleteComponent {
    /// Name/identifier of component
    pub label: String,
}

impl ToGuiLoopMessage for DeleteComponent {
    fn update_gui(self: Box<Self>, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components.remove(&self.label);
    }
}

/// Message from `GuiLoop` to `Manager`.
pub trait FromGuiLoopMessage: Send {
    /// How to update the state given user interactions (button presses etc.).
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>);
}

/// Enum update.

pub struct UpdateEnumStringRepr {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: String,
}

impl FromGuiLoopMessage for UpdateEnumStringRepr {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<EnumStringRepr>()
            .unwrap()
            .value = self.value.clone();
    }
}

/// Variable update..
pub struct UpdateValue<T> {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: T,
}

impl FromGuiLoopMessage for UpdateValue<bool> {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<Var<bool>>()
            .unwrap()
            .value = self.value;
    }
}

/// Slider update.
pub struct UpdateRangedValue<T> {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: T,
}

impl<T: Number> FromGuiLoopMessage for UpdateRangedValue<T> {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<RangedVar<T>>()
            .unwrap()
            .value = self.value;
    }
}

/// Button press event.
pub struct UpdateButton {
    /// The name.
    pub label: String,
}

impl FromGuiLoopMessage for UpdateButton {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<Button>()
            .unwrap()
            .pressed = true;
    }
}
