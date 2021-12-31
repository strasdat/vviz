#![allow(dead_code)]

use ::slice_of_array::prelude::*;
use enum_as_inner::EnumAsInner;

pub struct GuiData {
    pub components: linked_hash_map::LinkedHashMap<String, Box<dyn Component>>,
    pub widgets: linked_hash_map::LinkedHashMap<String, Box<dyn Widget>>,
}

impl GuiData {
    pub fn default() -> Self {
        Self {
            components: linked_hash_map::LinkedHashMap::new(),
            widgets: linked_hash_map::LinkedHashMap::new(),
        }
    }
}

pub trait FromGuiLoopMessage: Send {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>);
}

pub struct UpdateEnumStringRepr {
    pub label: String,
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

pub struct UpdateValue<T> {
    pub label: String,
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

pub struct UpdateButton {
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

pub struct UpdateRangedValue<T> {
    pub label: String,
    pub value: T,
}

pub trait Numbers: egui::emath::Numeric + downcast_rs::DowncastSync + std::fmt::Display {}

impl Numbers for i32 {}
impl Numbers for i64 {}
impl Numbers for f32 {}
impl Numbers for f64 {}

impl<T: Numbers> FromGuiLoopMessage for UpdateRangedValue<T> {
    fn update(&self, components: &mut linked_hash_map::LinkedHashMap<String, Box<dyn Component>>) {
        components
            .get_mut(&self.label)
            .unwrap()
            .downcast_mut::<RangedVar<T>>()
            .unwrap()
            .value = self.value;
    }
}

pub trait Component: downcast_rs::DowncastSync {
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

pub struct EnumStringRepr {
    pub value: String,
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

pub struct Var<T> {
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

pub struct Button {
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

impl<T: Numbers> Component for Var<T> {
    fn show(
        &mut self,
        label: &str,
        ui: &mut egui::Ui,
        _sender: &mut std::sync::mpsc::Sender<Box<dyn FromGuiLoopMessage>>,
    ) {
        ui.label(format!("{}: {}", label, self.value));
    }
}

pub struct RangedVar<T> {
    pub value: T,
    pub min: T,
    pub max: T,
}

impl<T: Numbers> Component for RangedVar<T> {
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

pub trait ToGuiLoopMessage: Send {
    fn update_gui(self: Box<Self>, data: &mut GuiData, ctx: &mut miniquad::Context);
}

pub struct AddEnumStringRepr {
    pub label: String,
    pub value: String,
    pub values: std::vec::Vec<String>,
}

impl ToGuiLoopMessage for AddEnumStringRepr {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.components.insert(
            self.label,
            Box::new(EnumStringRepr {
                value: self.value,
                values: self.values,
            }),
        );
    }
}

pub struct AddButton {
    pub label: String,
}

impl ToGuiLoopMessage for AddButton {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Button { pressed: false }));
    }
}

pub struct AddVar<T> {
    pub label: String,
    pub value: T,
}

impl ToGuiLoopMessage for AddVar<bool> {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<bool> { value: self.value }));
    }
}

impl<T: Numbers> ToGuiLoopMessage for AddVar<T> {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.components
            .insert(self.label, Box::new(Var::<T> { value: self.value }));
    }
}

pub struct AddRangedVar<T> {
    pub label: String,
    pub value: T,
    pub min: T,
    pub max: T,
}

impl<T: Numbers> ToGuiLoopMessage for AddRangedVar<T> {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
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

pub struct AddWidget3 {
    pub label: String,
}

impl ToGuiLoopMessage for AddWidget3 {
    fn update_gui(self: Box<Self>, data: &mut GuiData, ctx: &mut miniquad::Context) {
        data.widgets.insert(self.label, Box::new(Widget3::new(ctx)));
    }
}

pub struct DeleteComponent {
    pub label: String,
}

impl ToGuiLoopMessage for DeleteComponent {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.components.remove(&self.label);
    }
}

pub trait Widget: downcast_rs::DowncastSync {
    fn render(&mut self, ctx: &mut miniquad::Context);

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        assigned_width: f32,
        assigned_height: f32,
    ) -> Option<egui::Response>;

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

pub struct Widget3 {
    pub entities: linked_hash_map::LinkedHashMap<String, NamedEntity3>,
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

#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
}

struct PositionColorVertices {
    vertices: std::vec::Vec<[f32; 7]>,
}

impl PositionColorVertices {
    fn to_array(position: nalgebra::Vector3<f32>, color: Color) -> [f32; 7] {
        [
            position.x,
            position.y,
            position.z,
            color.r,
            color.g,
            color.b,
            color.alpha,
        ]
    }
}

#[repr(C)]
struct PositionUvVertices {
    vertices: std::vec::Vec<[f32; 5]>,
}

impl PositionUvVertices {
    fn to_array(position: nalgebra::Vector3<f32>, uv: nalgebra::Vector2<f32>) -> [f32; 5] {
        [position.x, position.y, position.z, uv.x, uv.y]
    }
}

pub struct Texture {}

struct PositionUvVerticesAndTexture {
    vertices: PositionUvVertices,
    texture: Texture,
}

#[derive(EnumAsInner)]
enum Vertices {
    PositionColor(PositionColorVertices),
    PositionUvAndTexture(PositionUvVerticesAndTexture),
}

pub struct Entity3 {
    vertices: Vertices,
    faces: Faces,
}

pub struct NamedEntity3 {
    pub label: String,
    pub entity: Entity3,
    pub scene_pose_entity: nalgebra::Isometry3<f32>,
}

impl Entity3 {
    fn from_position_color_vertices_and_faces(
        vertices: PositionColorVertices,
        faces: Faces,
    ) -> Self {
        Self {
            vertices: Vertices::PositionColor(vertices),
            faces,
        }
    }

    fn from_position_uv_vertices_texture_and_faces(
        vertices: PositionUvVertices,
        texture: Texture,
        faces: Faces,
    ) -> Self {
        Self {
            vertices: Vertices::PositionUvAndTexture(PositionUvVerticesAndTexture {
                vertices,
                texture,
            }),
            faces,
        }
    }
}

#[repr(C)]

pub struct Faces {
    indices: std::vec::Vec<[i16; 3]>,
}

impl Faces {
    fn new(indices: std::vec::Vec<[i16; 3]>) -> Self {
        Self { indices }
    }
}

pub fn colored_cube(scale: f32) -> Entity3 {
    #[rustfmt::skip]
    let vertices = PositionColorVertices{vertices: vec![
       [-scale, -scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [ scale, -scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [ scale,  scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [-scale,  scale, -scale,    1.0, 0.5, 0.5, 1.0],

       [-scale, -scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [ scale, -scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [ scale,  scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [-scale,  scale,  scale,    0.5, 1.0, 0.5, 1.0],
  
       [-scale, -scale, -scale,    0.5, 0.5, 1.0, 1.0],
       [-scale,  scale, -scale,    0.5, 0.5, 1.0, 1.0],
       [-scale,  scale,  scale,    0.5, 0.5, 1.0, 1.0],
       [-scale, -scale,  scale,    0.5, 0.5, 1.0, 1.0],
  
       [ scale, -scale, -scale,    1.0, 0.5, 0.0, 1.0],
       [ scale,  scale, -scale,    1.0, 0.5, 0.0, 1.0],
       [ scale,  scale,  scale,    1.0, 0.5, 0.0, 1.0],
       [ scale, -scale,  scale,    1.0, 0.5, 0.0, 1.0],
       
       [-scale, -scale, -scale,    0.0, 0.5, 1.0, 1.0],
       [-scale, -scale,  scale,    0.0, 0.5, 1.0, 1.0],
       [ scale, -scale,  scale,    0.0, 0.5, 1.0, 1.0],
       [ scale, -scale, -scale,    0.0, 0.5, 1.0, 1.0],
         
       [-scale,  scale, -scale,    1.0, 0.0, 0.5, 1.0],
       [-scale,  scale,  scale,    1.0, 0.0, 0.5, 1.0],
       [ scale,  scale,  scale,    1.0, 0.0, 0.5, 1.0],
       [ scale,  scale, -scale,    1.0, 0.0, 0.5, 1.0],
    ]};

    let faces = Faces::new(vec![
        [0, 1, 2],
        [0, 2, 3],
        [6, 5, 4],
        [7, 6, 4],
        [8, 9, 10],
        [8, 10, 11],
        [14, 13, 12],
        [15, 14, 12],
        [16, 17, 18],
        [16, 18, 19],
        [22, 21, 20],
        [23, 22, 20],
    ]);

    Entity3::from_position_color_vertices_and_faces(vertices, faces)
}

pub struct PlaceEntity {
    pub widget_label: String,
    pub named_entity: NamedEntity3,
}

impl ToGuiLoopMessage for PlaceEntity {
    fn update_gui(self: Box<Self>, data: &mut GuiData, _ctx: &mut miniquad::Context) {
        data.widgets
            .get_mut(&self.widget_label)
            .unwrap()
            .downcast_mut::<Widget3>()
            .unwrap()
            .entities
            .insert(self.named_entity.label.clone(), self.named_entity);
    }
}

pub struct ColoredTriangle {
    pub face: [[f32; 3]; 3],
    pub color: Color,
}

impl ColoredTriangle {
    fn vec_of_arrays(vec_of_triangles: &[ColoredTriangle]) -> std::vec::Vec<[f32; 7]> {
        let mut result = std::vec::Vec::<[f32; 7]>::with_capacity(3 * vec_of_triangles.len());
        for triangle in vec_of_triangles {
            for vertex in triangle.face {
                result.push([
                    vertex[0],
                    vertex[1],
                    vertex[2],
                    triangle.color.r,
                    triangle.color.g,
                    triangle.color.b,
                    triangle.color.alpha,
                ])
            }
        }
        result
    }
}

pub fn colored_triangles(triangles: std::vec::Vec<ColoredTriangle>) -> Entity3 {
    let vertices = PositionColorVertices {
        vertices: ColoredTriangle::vec_of_arrays(&triangles),
    };
    let mut faces: Vec<[i16; 3]> = std::vec::Vec::new();

    let len: i16 = triangles.len().try_into().unwrap();
    for i in 0..len {
        faces.push([i * 3, i * 3 + 1, i * 3 + 2])
    }
    Entity3::from_position_color_vertices_and_faces(vertices, Faces::new(faces))
}
