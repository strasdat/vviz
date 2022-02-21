//! Common structures shared between [super::manager::Manager] and [super::gui::GuiLoop].

use super::entities;
use super::gui;

use ::slice_of_array::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSize {
    pub width: i64,
    pub height: i64,   
}

impl Default for ImageSize {
    fn default() -> Self {
        ImageSize{width:640, height:480}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PinholeCamera {
    pub image_size: ImageSize,
    pub focal_length: f32,
    pub center: nalgebra::Vector2<f32>
}

impl PinholeCamera {
     fn default_from_size(size: ImageSize) -> Self{
         let w = size.width as f32;
         let h = size.height as f32;
        Self {
            image_size: size,
            focal_length: (w+h) * 0.25,
            center: nalgebra::Vector2::new(0.5*w - 0.5,0.5*h - 0.5),
        }
    }
}


impl Default for PinholeCamera {
    fn default() -> Self {
        Self::default_from_size(ImageSize::default())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClippingPlanes {
    pub near: f32,
    pub far: f32,
}

impl Default for ClippingPlanes {
    fn default() -> Self {
        Self{near: 0.01, far:100.0}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WidgetProjection {
    pub camera : PinholeCamera,
    pub clipping_planes: ClippingPlanes,
}


/// Component such as a button or a slider.
pub trait Component: downcast_rs::DowncastSync {
    /// How to display the component on the side panel.
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
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
        sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
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
                .send(FromGuiLoopMessage::UpdateEnumStringRepr(
                    UpdateEnumStringRepr {
                        label: label.to_string(),
                        value: self.value.clone(),
                    },
                ))
                .unwrap();
        }
    }
}

/// Variable bool (checkbox) or numeric (read-only text box).
///
/// Interfaced by [super::manager::UiVar].
pub struct Var<T> {
    /// Current value.
    pub value: T,
}

impl Component for Var<bool> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
    ) {
        if ui.checkbox(&mut self.value, label).changed() {
            sender
                .send(FromGuiLoopMessage::UpdateValueBool(UpdateValue {
                    label: label.to_string(),
                    value: self.value,
                }))
                .unwrap();
        }
    }
}

/// A button.
///
/// Interfaced by [super::manager::UiButton].
pub struct Button {
    /// Is true is recently pressed.
    pub pressed: bool,
}

impl Component for Button {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
    ) {
        if ui.button(label).clicked() {
            sender
                .send(FromGuiLoopMessage::UpdateButton(UpdateButton {
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
        _sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
    ) {
        ui.label(format!("{}: {}", label, self.value));
    }
}

/// A range value, represented as slider.
///
/// Interfaced by [super::manager::UiRangedVar].
pub struct RangedVar<T> {
    /// Current value.
    pub value: T,
    /// Min, max bounds.
    pub min_max: (T, T),
}

impl<T: Number> Component for RangedVar<T> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        sender: &mut std::sync::mpsc::Sender<FromGuiLoopMessage>,
    ) {
        if ui
            .add(egui::Slider::new(&mut self.value, self.min_max.0..=self.min_max.1).text(label))
            .changed()
        {
            sender
                .send(self.value.update_range_value_message(label.to_owned()))
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

    /// The aspect ratio of self.
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

/// [Widget] for 2d content.
pub struct Widget2 {
    aspect_ratio: f32,
    maybe_image: Option<miniquad::Texture>,
}

impl Widget2 {
    fn from_image(ctx: &mut miniquad::Context, rgba8: ImageRgba8) -> Self {
        let tex = miniquad::Texture::from_rgba8(
            ctx,
            rgba8.width as u16,
            rgba8.height as u16,
            rgba8.bytes.as_slice(),
        );
        Self {
            aspect_ratio: rgba8.width as f32 / rgba8.height as f32,
            maybe_image: Some(tex),
        }
    }

    // fn from_aspect_ratio(aspect_ratio: f32) -> Self {
    //     Self {
    //         aspect_ratio,
    //         maybe_image: None,
    //     }
    // }

    // fn from_image_size(width: f32, height: f32) -> Self {
    //     Self::from_aspect_ratio(width / height)
    // }
}

impl Widget for Widget2 {
    fn render(&mut self, _ctx: &mut miniquad::Context) {}

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        assigned_width: f32,
        assigned_height: f32,
    ) -> Option<egui::Response> {
        let w = (self.aspect_ratio * assigned_height).min(assigned_width);
        let h = w / self.aspect_ratio;

        let tex = egui::TextureId::User(self.maybe_image.unwrap().gl_internal_id() as u64);

        let r = ui
            .add(egui::Image::new(tex, egui::Vec2::new(w, h)).sense(egui::Sense::click_and_drag()));
        Some(r)
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

/// [Widget] for 3d content such as meshes, line segments and point clouds.
pub struct Widget3 {
    camera_pose_scene: nalgebra::Isometry3<f32>,
    entities: linked_hash_map::LinkedHashMap<String, entities::NamedEntity3>,
    mesh_pipeline: miniquad::Pipeline,
    segments_pipeline: miniquad::Pipeline,
    offscreen_pass: miniquad::RenderPass,
    aspect_ratio: f32,
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

        let mesh_pipeline = miniquad::Pipeline::with_params(
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

        let segments_pipeline = miniquad::Pipeline::with_params(
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
                primitive_type: miniquad::PrimitiveType::Lines,
                ..Default::default()
            },
        );

        Self {
            camera_pose_scene: nalgebra::Isometry3::<f32>::from_parts(
                nalgebra::Translation3::<f32>::new(0.0, 0.0, -4.0),
                nalgebra::UnitQuaternion::<f32>::from_euler_angles(0.0, 0.0, 0.),
            ),
            entities: linked_hash_map::LinkedHashMap::new(),
            mesh_pipeline,
            segments_pipeline,
            offscreen_pass,
            aspect_ratio: 640.0 / 480.0,
            texture_id: None,
        }
    }
}

impl Widget for Widget3 {
    fn render(&mut self, ctx: &mut miniquad::Context) {
        let proj = nalgebra_glm::perspective_fov_rh(60.0f32.to_radians(), 640.0, 480.0, 0.01, 10.0);

        // the offscreen render pipeline, following this example:
        // https://github.com/not-fl3/egui-miniquad/blob/master/examples/render_to_egui_image.rs
        ctx.begin_pass(
            self.offscreen_pass,
            miniquad::PassAction::clear_color(1.0, 1.0, 1.0, 1.),
        );
        for (_, named_entity) in &self.entities {
            match &named_entity.entity {
                entities::Entity3::Mesh(mesh) => {
                    let vertex_buffer = miniquad::Buffer::immutable(
                        ctx,
                        miniquad::BufferType::VertexBuffer,
                        mesh.vertices.as_position_color().unwrap().vertices.flat(),
                    );

                    let index_buffer = miniquad::Buffer::immutable(
                        ctx,
                        miniquad::BufferType::IndexBuffer,
                        mesh.faces.indices.flat(),
                    );

                    let offscreen_bind = miniquad::Bindings {
                        vertex_buffers: vec![vertex_buffer],
                        index_buffer,
                        images: vec![],
                    };

                    ctx.apply_pipeline(&self.mesh_pipeline);
                    ctx.apply_bindings(&offscreen_bind);

                    let vs_params = offscreen_shader::Uniforms {
                        mvp: proj
                            * self.camera_pose_scene.to_matrix()
                            * named_entity.scene_pose_entity.to_matrix(),
                    };
                    ctx.apply_uniforms(&vs_params);

                    ctx.draw(0, mesh.faces.indices.flat().len() as i32, 1);
                }
                entities::Entity3::LineSegments(segments) => {
                    let vertex_buffer = miniquad::Buffer::immutable(
                        ctx,
                        miniquad::BufferType::VertexBuffer,
                        segments.vertices.vertices.flat(),
                    );

                    let index_buffer = miniquad::Buffer::immutable(
                        ctx,
                        miniquad::BufferType::IndexBuffer,
                        segments.indices.flat(),
                    );

                    let offscreen_bind = miniquad::Bindings {
                        vertex_buffers: vec![vertex_buffer],
                        index_buffer,
                        images: vec![],
                    };

                    ctx.apply_pipeline(&self.segments_pipeline);
                    ctx.apply_bindings(&offscreen_bind);

                    let vs_params = offscreen_shader::Uniforms {
                        mvp: proj
                            * self.camera_pose_scene.to_matrix()
                            * named_entity.scene_pose_entity.to_matrix(),
                    };
                    ctx.apply_uniforms(&vs_params);

                    ctx.draw(0, segments.indices.flat().len() as i32, 1);
                }
            }
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

        if ui.ctx().input().pointer.secondary_down() {
            // TODO: Calculate delta scale based on scene depth.
            let delta = 0.01 * ui.ctx().input().pointer.delta();
            let mut scene_pose_camera = self.camera_pose_scene.inverse();
            let translation_update = scene_pose_camera
                .rotation
                .transform_vector(&nalgebra::Vector3::new(-delta.x, -delta.y, 0.0));
            scene_pose_camera.append_translation_mut(&nalgebra::Translation3 {
                vector: translation_update,
            });
            self.camera_pose_scene = scene_pose_camera.inverse();
        } else if ui.ctx().input().pointer.primary_down() {
            // TODO: Rotates about scene center. Make the center point of rotation configurable.
            let delta = 0.01 * ui.ctx().input().pointer.delta();
            let mut scaled_axis = nalgebra::Vector3::zeros();
            scaled_axis.x = -delta.y;
            scaled_axis.y = delta.x;

            let scene_rot_camera = self.camera_pose_scene.rotation.inverse();

            self.camera_pose_scene.rotation *= nalgebra::UnitQuaternion::from_scaled_axis(
                scene_rot_camera.transform_vector(&scaled_axis),
            );
        }

        Some(r)
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

/// Integer or floating point number.

pub trait Number:
    egui::emath::Numeric + downcast_rs::DowncastSync + std::fmt::Display + serde::Serialize
{
    /// AddVar message.
    fn add_var_message(self, label: String) -> ToGuiLoopMessage;

    /// AddRangedVar message.
    fn add_ranged_var_message(self, label: String, min_max: (Self, Self)) -> ToGuiLoopMessage;

    /// UpdateRangedValue message
    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage;
}

impl Number for usize {
    fn add_var_message(self, label: String) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddVarUSize(AddVar::<usize> { label, value: self })
    }

    fn add_ranged_var_message(self, label: String, min_max: (usize, usize)) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddRangedVarUSize(AddRangedVar::<usize> {
            label,
            min_max,
            value: self,
        })
    }

    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage {
        FromGuiLoopMessage::UpdateRangedValueUSize(UpdateRangedValue { label, value: self })
    }
}

impl Number for i32 {
    fn add_var_message(self, label: String) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddVarI32(AddVar::<i32> { label, value: self })
    }

    fn add_ranged_var_message(self, label: String, min_max: (i32, i32)) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddRangedVarI32(AddRangedVar::<i32> {
            label,
            min_max,
            value: self,
        })
    }

    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage {
        FromGuiLoopMessage::UpdateRangedValueI32(UpdateRangedValue { label, value: self })
    }
}

impl Number for i64 {
    fn add_var_message(self, label: String) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddVarI64(AddVar::<i64> { label, value: self })
    }

    fn add_ranged_var_message(self, label: String, min_max: (i64, i64)) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddRangedVarI64(AddRangedVar::<i64> {
            label,
            min_max,
            value: self,
        })
    }

    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage {
        FromGuiLoopMessage::UpdateRangedValueI64(UpdateRangedValue { label, value: self })
    }
}

impl Number for f32 {
    fn add_var_message(self, label: String) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddVarF32(AddVar::<f32> { label, value: self })
    }

    fn add_ranged_var_message(self, label: String, min_max: (f32, f32)) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddRangedVarF32(AddRangedVar::<f32> {
            label,
            min_max,
            value: self,
        })
    }

    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage {
        FromGuiLoopMessage::UpdateRangedValueF32(UpdateRangedValue { label, value: self })
    }
}

impl Number for f64 {
    fn add_var_message(self, label: String) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddVarF64(AddVar::<f64> { label, value: self })
    }

    fn add_ranged_var_message(self, label: String, min_max: (f64, f64)) -> ToGuiLoopMessage {
        ToGuiLoopMessage::AddRangedVarF64(AddRangedVar::<f64> {
            label,
            min_max,
            value: self,
        })
    }

    fn update_range_value_message(self, label: String) -> FromGuiLoopMessage {
        FromGuiLoopMessage::UpdateRangedValueF64(UpdateRangedValue { label, value: self })
    }
}

/// Message from  [super::manager::Manager] to [super::gui::GuiLoop], such as to add a component or
/// widget.
#[derive(Serialize, Deserialize, Debug)]
pub enum ToGuiLoopMessage {
    /// enum combobox
    AddEnumStringRepr(AddEnumStringRepr),
    /// button
    AddButton(AddButton),
    /// bool checkbox
    AddVarBool(AddVar<bool>),
    /// usize textbox
    AddVarUSize(AddVar<usize>),
    /// i32 textbox
    AddVarI32(AddVar<i32>),
    /// i64 textbox
    AddVarI64(AddVar<i64>),
    /// f32 textbox
    AddVarF32(AddVar<f32>),
    /// f64 textbox
    AddVarF64(AddVar<f64>),
    /// usize slider
    AddRangedVarUSize(AddRangedVar<usize>),
    /// i32 textbox
    AddRangedVarI32(AddRangedVar<i32>),
    /// i64 textbox
    AddRangedVarI64(AddRangedVar<i64>),
    /// f32 textbox
    AddRangedVarF32(AddRangedVar<f32>),
    /// f64 textbox
    AddRangedVarF64(AddRangedVar<f64>),
    /// 2d widget
    AddWidget2(AddWidget2),
    /// 3d widget
    AddWidget3(AddWidget3),
    /// place 3d entity
    PlaceEntity3(PlaceEntity3),
    /// delete component
    DeleteComponent(DeleteComponent),
    /// update pose of 3d entity
    UpdateScenePoseEntity3(UpdateScenePoseEntity3),
}

impl ToGuiLoopMessage {
    /// How that component or widget shall be displayed.
    pub fn update_gui(self, data: &mut gui::GuiData, ctx: &mut miniquad::Context) {
        use ToGuiLoopMessage::*;

        match self {
            AddEnumStringRepr(e) => {
                e.update_gui(data, ctx);
            }
            AddButton(e) => {
                e.update_gui(data, ctx);
            }
            AddVarBool(e) => {
                e.update_gui(data, ctx);
            }
            AddVarUSize(e) => {
                e.update_gui(data, ctx);
            }
            AddVarI32(e) => {
                e.update_gui(data, ctx);
            }
            AddVarI64(e) => {
                e.update_gui(data, ctx);
            }
            AddVarF32(e) => {
                e.update_gui(data, ctx);
            }
            AddVarF64(e) => {
                e.update_gui(data, ctx);
            }
            AddRangedVarUSize(e) => {
                e.update_gui(data, ctx);
            }
            AddRangedVarI32(e) => {
                e.update_gui(data, ctx);
            }
            AddRangedVarI64(e) => {
                e.update_gui(data, ctx);
            }
            AddRangedVarF32(e) => {
                e.update_gui(data, ctx);
            }
            AddRangedVarF64(e) => {
                e.update_gui(data, ctx);
            }
            AddWidget2(e) => {
                e.update_gui(data, ctx);
            }
            AddWidget3(e) => {
                e.update_gui(data, ctx);
            }
            PlaceEntity3(e) => {
                e.update_gui(data, ctx);
            }
            DeleteComponent(e) => {
                e.update_gui(data, ctx);
            }
            UpdateScenePoseEntity3(e) => {
                e.update_gui(data, ctx);
            }
        }
    }
}

/// Add an enum (as string representation) as combo box to side panel.
#[derive(Serialize, Deserialize, Debug)]
pub struct AddEnumStringRepr {
    /// The name of the enum.
    pub label: String,
    /// Initial value.
    pub value: String,
    /// List of possible values.
    pub values: std::vec::Vec<String>,
}

impl AddEnumStringRepr {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
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
#[derive(Serialize, Deserialize, Debug)]
pub struct AddButton {
    /// The name of button.
    pub label: String,
}

impl AddButton {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Button { pressed: false }));
    }
}

/// Add bool (as checkbox) or numeric value (as read-only text box) to side panel.
///
/// Also see [Var].
#[derive(Serialize, Deserialize, Debug)]
pub struct AddVar<T> {
    /// The name of variable.
    pub label: String,
    /// The initial value.
    pub value: T,
}

impl AddVar<bool> {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<bool> { value: self.value }));
    }
}

impl<T: Number> AddVar<T> {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<T> { value: self.value }));
    }
}

/// Add a numeric value as a slider to side panel.
///
/// Also see [RangedVar].
#[derive(Serialize, Deserialize, Debug)]
pub struct AddRangedVar<T> {
    /// Name of variable.
    pub label: String,
    /// Initial value.
    pub value: T,
    /// Min, max bounds
    pub min_max: (T, T),
}

impl<T: Number> AddRangedVar<T> {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components.insert(
            self.label,
            Box::new(RangedVar::<T> {
                value: self.value,
                min_max: self.min_max,
            }),
        );
    }
}

/// u8 RGBA image
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageRgba8 {
    /// raw bytes
    pub bytes: Vec<u8>,
    /// image width
    pub width: u32,
    /// image height
    pub height: u32,
}

/// Adds [Widget2] to main panel.
#[derive(Serialize, Deserialize, Debug)]
pub struct AddWidget2 {
    /// Name of widget
    pub label: String,
    /// Image to show in (background of) widget
    pub proj: WidgetProjection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClearWidget2AndUpdateProjection {
     /// Name of widget
     pub label: String,
    pub proj: WidgetProjection,
}

#[derive(Serialize, Deserialize, Debug)]
/// Tries to update background image of width `label`.
/// 
/// This operation is no-op if
///  - widget `label` does not exist,
///  - image.image_size != widget.image_size
pub struct TryUpdateImage {
    /// Name of widget
    pub label: String,
   pub image: ImageRgba8,
}

impl AddWidget2 {
    fn update_gui(self, data: &mut gui::GuiData, ctx: &mut miniquad::Context) {
        data.widgets
            .insert(self.label, Box::new(Widget2::from_image(ctx, self.image)));
    }
}

/// Adds [Widget3] to main panel.
#[derive(Serialize, Deserialize, Debug)]
pub struct AddWidget3 {
    /// Name of widget
    pub label: String,
}

impl AddWidget3 {
    fn update_gui(self, data: &mut gui::GuiData, ctx: &mut miniquad::Context) {
        data.widgets.insert(self.label, Box::new(Widget3::new(ctx)));
    }
}

/// Place [super::entities::Entity3] in corresponding [Widget3].
#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceEntity3 {
    /// Name of widget.
    pub widget_label: String,
    /// The 3d entity
    pub named_entity: entities::NamedEntity3,
}

impl PlaceEntity3 {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.widgets
            .get_mut(&self.widget_label)
            .unwrap()
            .downcast_mut::<Widget3>()
            .unwrap()
            .entities
            .insert(self.named_entity.label.clone(), self.named_entity);
    }
}

/// Updates pose of [super::entities::Entity3] in corresponding [Widget3].
///
/// It is no-op, if an entity with that name `entity_label` does not exist.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateScenePoseEntity3 {
    /// Name of widget.
    pub widget_label: String,
    /// Name of entity.
    pub entity_label: String,
    /// Pose of the entity in the scene.
    pub scene_pose_entity: nalgebra::Isometry3<f32>,
}

impl UpdateScenePoseEntity3 {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        let maybe_entity = data
            .widgets
            .get_mut(&self.widget_label)
            .unwrap()
            .downcast_mut::<Widget3>()
            .unwrap()
            .entities
            .get_mut(&self.entity_label);
        if maybe_entity.is_none() {
            // No-op.
            return;
        }
        maybe_entity.unwrap().scene_pose_entity = self.scene_pose_entity;
    }
}

/// Delete that component from side panel.
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteComponent {
    /// Name/identifier of component
    pub label: String,
}

impl DeleteComponent {
    fn update_gui(self, data: &mut gui::GuiData, _ctx: &mut miniquad::Context) {
        data.components.remove(&self.label);
    }
}

/// Message from [super::gui::GuiLoop] to [super::manager::Manager].
#[derive(Serialize, Deserialize, Debug)]
pub enum FromGuiLoopMessage {
    /// enum combobox update
    UpdateEnumStringRepr(UpdateEnumStringRepr),
    /// bool checkbox update
    UpdateValueBool(UpdateValue<bool>),
    /// usize slider update
    UpdateRangedValueUSize(UpdateRangedValue<usize>),
    /// i32 slider update
    UpdateRangedValueI32(UpdateRangedValue<i32>),
    /// i64 slider update
    UpdateRangedValueI64(UpdateRangedValue<i64>),
    /// f32 slider update
    UpdateRangedValueF32(UpdateRangedValue<f32>),
    /// f64 slider update
    UpdateRangedValueF64(UpdateRangedValue<f64>),
    /// button update
    UpdateButton(UpdateButton),
}

impl FromGuiLoopMessage {
    /// How to update the state given user interactions (button presses etc.).
    pub fn update(
        &self,
        components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>,
    ) {
        use FromGuiLoopMessage::*;

        match self {
            UpdateEnumStringRepr(e) => e.update(components),
            UpdateValueBool(e) => e.update(components),
            UpdateRangedValueUSize(e) => e.update(components),
            UpdateRangedValueI32(e) => e.update(components),
            UpdateRangedValueI64(e) => e.update(components),
            UpdateRangedValueF32(e) => e.update(components),
            UpdateRangedValueF64(e) => e.update(components),
            UpdateButton(e) => e.update(components),
        }
    }
}

/// [super::manager::UiEnum]  (i.e. slider) update.
///
/// See also [EnumStringRepr].
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateEnumStringRepr {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: String,
}

impl UpdateEnumStringRepr {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<EnumStringRepr>()
            .unwrap()
            .value = self.value.clone();
    }
}

/// [Var] update.
///
/// See also [super::manager::UiVar].
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateValue<T> {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: T,
}

impl UpdateValue<bool> {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<Var<bool>>()
            .unwrap()
            .value = self.value;
    }
}

/// [RangedVar] (slider) update.
///
/// See also [super::manager::UiRangedVar].
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRangedValue<T> {
    /// The name.
    pub label: String,
    /// The new value.
    pub value: T,
}

impl<T: Number> UpdateRangedValue<T> {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<RangedVar<T>>()
            .unwrap()
            .value = self.value;
    }
}

/// [Button] press event.
///
/// See also [super::manager::UiButton].
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateButton {
    /// The name.
    pub label: String,
}

impl UpdateButton {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<Button>()
            .unwrap()
            .pressed = true;
    }
}
