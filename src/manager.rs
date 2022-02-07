//! User API for UI interaction.

use linked_hash_map::LinkedHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc;

use crate::common::FromGuiLoopMessage;
use crate::common::ToGuiLoopMessage;

use super::common;
use super::entities;

/// Shared data between the varies ui structs such [UiButton], [UiWidget3] and [UiVar<T>].
pub struct Shared {
    components: LinkedHashMap<String, Box<dyn common::Component>>,
    message_queue: std::collections::VecDeque<common::ToGuiLoopMessage>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            components: LinkedHashMap::new(),
            message_queue: std::collections::VecDeque::new(),
        }
    }
}

struct LocalConnection {}

struct WebsocketServerConnection {
    _thread_join_handle: std::thread::JoinHandle<()>,
}

enum ManagerConnection {
    Local(LocalConnection),
    WebsocketServer(WebsocketServerConnection),
}

/// The users employ the [Manager] to add [super::common::Component]s and [super::common::Widget]s
/// to the gui, and receive state updates.
///
/// It communicates with [super::gui::GuiLoop] through sender and receiver structs.
pub struct Manager {
    to_gui_loop_sender: mpsc::Sender<common::ToGuiLoopMessage>,
    from_gui_loop_receiver: mpsc::Receiver<common::FromGuiLoopMessage>,
    _connection: ManagerConnection,
    shared: Rc<RefCell<Shared>>,
}

/// Ui element to manipulate an enum. It is represented as a combo box.
pub struct UiEnum<T> {
    shared: Rc<RefCell<Shared>>,
    label: String,
    cache: T,
}

impl<
        T: std::fmt::Debug + ToString + strum::VariantNames + std::str::FromStr + PartialEq + Clone,
    > UiEnum<T>
{
    fn new(shared: Rc<RefCell<Shared>>, label: String, value: T) -> Self {
        let mut values_map = std::vec::Vec::new();
        for str in T::VARIANTS {
            let owned_str = str.to_string();
            values_map.push(owned_str);
        }

        shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::AddEnumStringRepr(
                common::AddEnumStringRepr {
                    label: label.clone(),
                    value: value.to_string(),
                    values: values_map.clone(),
                },
            ));

        shared.borrow_mut().components.insert(
            label.clone(),
            Box::new(common::EnumStringRepr {
                value: value.to_string(),
                values: values_map,
            }),
        );
        Self {
            shared,
            label,
            cache: value,
        }
    }

    /// Returns the current enum value.
    pub fn get_value(&mut self) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let string_repr = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::EnumStringRepr>()
            .unwrap()
            .value
            .clone();
        let value: T = FromStr::from_str(&string_repr).unwrap();
        self.cache = value.clone();
        value
    }

    /// Only returns the current enum value if it was updated.
    pub fn get_new_value(&mut self) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let string_repr = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::EnumStringRepr>()
            .unwrap()
            .value
            .clone();
        let value: T = FromStr::from_str(&string_repr).unwrap();
        if value != self.cache {
            self.cache = value.clone();
            return Some(value);
        }
        None
    }
}

/// Represents a button in the side-panel.
pub struct UiButton {
    shared: Rc<RefCell<Shared>>,
    label: String,
}

impl UiButton {
    fn new(shared: Rc<RefCell<Shared>>, label: String) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::AddButton(common::AddButton {
                label: label.clone(),
            }));
        shared
            .borrow_mut()
            .components
            .insert(label.clone(), Box::new(common::Button { pressed: false }));
        Self { shared, label }
    }

    /// Returns true if button was pressed.
    pub fn was_pressed(&mut self) -> bool {
        let pressed = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::Button>()
            .unwrap()
            .pressed;
        if pressed {
            self.shared
                .borrow_mut()
                .components
                .get_mut(&self.label)
                .unwrap()
                .downcast_mut::<common::Button>()
                .unwrap()
                .pressed = false;
        }
        pressed
    }
}

/// Ui element for a [bool] or number ([i32], [i64], [f32], [f64]).
///
/// The bool is represented as a checkbox. The [Number][super::common::Number] is
/// considered constant and represented as a readonly text box.
pub struct UiVar<T> {
    shared: Rc<RefCell<Shared>>,
    label: String,
    cache: T,
}

impl UiVar<bool> {
    fn new(shared: Rc<RefCell<Shared>>, label: String, value: bool) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::AddVarBool(common::AddVar::<bool> {
                label: label.clone(),
                value,
            }));
        shared
            .borrow_mut()
            .components
            .insert(label.clone(), Box::new(common::Var::<bool> { value }));
        Self {
            shared,
            label,
            cache: value,
        }
    }

    /// Returns the current boolean value.
    pub fn get_value(&mut self) -> bool {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::Var<bool>>()
            .unwrap()
            .value;
        self.cache = value;
        value
    }

    /// Only returns the current boolean value if it was updated.
    pub fn get_new_value(&mut self) -> Option<bool> {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::Var<bool>>()
            .unwrap()
            .value;
        if value != self.cache {
            self.cache = value;
            return Some(value);
        }
        None
    }
}

impl<T: common::Number> UiVar<T> {
    fn new(shared: Rc<RefCell<Shared>>, label: String, value: T) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(value.add_var_message(label.clone()));
        shared
            .borrow_mut()
            .components
            .insert(label.clone(), Box::new(common::Var::<T> { value }));
        Self {
            shared,
            label,
            cache: value,
        }
    }

    /// Returns the current numeric value.
    pub fn get_value(&mut self) -> T {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::Var<T>>()
            .unwrap()
            .value;
        self.cache = value;
        value
    }

    /// Only returns the current numeric value if it was updated.
    pub fn get_new_value(&mut self) -> Option<T> {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::Var<T>>()
            .unwrap()
            .value;
        if value != self.cache {
            self.cache = value;
            return Some(value);
        }
        None
    }
}

/// Ui element for a [super::common::Number] ([i32], [i64], [f32], [f64]) with a given range
/// `[min, max]`.
///
/// It is represented as a slider.
pub struct UiRangedVar<T> {
    shared: Rc<RefCell<Shared>>,
    label: String,
    cache: T,
}

impl<T: common::Number> UiRangedVar<T> {
    fn new(shared: Rc<RefCell<Shared>>, label: String, value: T, (min, max): (T, T)) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(value.add_ranged_var_message(label.clone(), (min, max)));
        shared.borrow_mut().components.insert(
            label.clone(),
            Box::new(common::RangedVar::<T> {
                value,
                min_max: (min, max),
            }),
        );
        Self {
            shared,
            label,
            cache: value,
        }
    }

    /// Returns the current numeric value; it is guaranteed to be within its bounds `[min, max]`
    pub fn get_value(&mut self) -> T {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::RangedVar<T>>()
            .unwrap()
            .value;
        self.cache = value;
        value
    }

    /// Only returns the current numeric value if it was updated.
    /// In this case, is guaranteed to be within its bounds `[min, max]`
    pub fn get_new_value(&mut self) -> Option<T> {
        let value = self
            .shared
            .borrow()
            .components
            .get(&self.label)
            .unwrap()
            .downcast_ref::<common::RangedVar<T>>()
            .unwrap()
            .value;
        if value != self.cache {
            self.cache = value;
            return Some(value);
        }
        None
    }
}

/// 2d widget.
pub struct UiWidget2 {
    // label: String,
// hared: Rc<RefCell<Shared>>,
}

impl UiWidget2 {
    fn new(
        shared: Rc<RefCell<Shared>>,
        label: String,
        rgba8: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    ) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::AddWidget2(common::AddWidget2 {
                label,
                image: common::ImageRgba8 {
                    width: rgba8.width(),
                    height: rgba8.height(),
                    bytes: rgba8.into_raw(),
                },
            }));

        Self {}
    }
}

/// 3d widget.
pub struct UiWidget3 {
    label: String,
    shared: Rc<RefCell<Shared>>,
}

impl UiWidget3 {
    fn new(shared: Rc<RefCell<Shared>>, label: String) -> Self {
        shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::AddWidget3(common::AddWidget3 {
                label: label.clone(),
            }));

        Self { label, shared }
    }

    /// Adds new [entities::Entity3] to [UiWidget3]. If an entity with such `label` already exists
    /// it will be replaced.
    pub fn place_entity(&self, label: String, entity: entities::Entity3) {
        self.shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::PlaceEntity3(common::PlaceEntity3 {
                widget_label: self.label.clone(),
                named_entity: entities::NamedEntity3 {
                    label,
                    entity,
                    scene_pose_entity: nalgebra::Isometry3::<f32>::identity(),
                },
            }));
    }

    /// Adds new [entities::Entity3] to [UiWidget3] at specified pose. If an entity with such
    /// `label` already exists it will be replaced.
    ///
    /// Here `scene`_pose_entity` is the pose (3d position and orientation) of the entity in the
    /// scene reference frame.
    pub fn place_entity_at(
        &self,
        label: String,
        entity: entities::Entity3,
        scene_pose_entity: nalgebra::Isometry3<f32>,
    ) {
        self.shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::PlaceEntity3(common::PlaceEntity3 {
                widget_label: self.label.clone(),
                named_entity: entities::NamedEntity3 {
                    label,
                    entity,
                    scene_pose_entity,
                },
            }));
    }

    /// Updates `scene`_pose_entity` of the [entities::Entity3] with name `label`.
    ///
    /// If no such entity exists, this is no-op.
    ///
    /// Here `scene`_pose_entity` is the pose (3d position and orientation) of the entity in the
    /// scene reference frame.
    pub fn update_scene_pose_entity(
        &self,
        label: String,
        scene_pose_entity: nalgebra::Isometry3<f32>,
    ) {
        self.shared
            .borrow_mut()
            .message_queue
            .push_back(ToGuiLoopMessage::UpdateScenePoseEntity3(
                common::UpdateScenePoseEntity3 {
                    widget_label: self.label.clone(),
                    entity_label: label,
                    scene_pose_entity,
                },
            ));
    }
}

impl Manager {
    /// Constructs local [Manager] from sender/receiver. This usually needs not be called by the
    /// user, since it is constructed by the [super::app].
    pub fn new_local(
        to_gui_loop_sender: mpsc::Sender<common::ToGuiLoopMessage>,
        from_gui_loop_receiver: mpsc::Receiver<common::FromGuiLoopMessage>,
    ) -> Self {
        Self {
            to_gui_loop_sender,
            from_gui_loop_receiver,
            _connection: ManagerConnection::Local(LocalConnection {}),
            shared: Rc::new(RefCell::new(Shared::default())),
        }
    }

    /// Constructs remote [Manager] from sender/receiver. This usually needs not be called by the
    /// user, since it is constructed by the [super::app].
    pub fn new_remote() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();

        let mut websocket = tungstenite::accept(listener.accept().unwrap().0).unwrap();
        let (to_gui_loop_sender, to_gui_loop_receiver) = std::sync::mpsc::channel();
        let (from_gui_loop_sender, from_gui_loop_receiver) = std::sync::mpsc::channel();

        let thread_join_handle = std::thread::spawn(move || loop {
            let msg = websocket.read_message().unwrap();

            let from_msg: Vec<FromGuiLoopMessage> =
                serde_json::from_str(msg.to_text().unwrap()).unwrap();
            for m in from_msg {
                from_gui_loop_sender.send(m).unwrap();
            }

            let collection: Vec<ToGuiLoopMessage> = to_gui_loop_receiver.try_iter().collect();

            websocket
                .write_message(tungstenite::Message::Text(
                    serde_json::to_string(&collection).unwrap(),
                ))
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(15));
        });

        Self {
            to_gui_loop_sender,
            from_gui_loop_receiver,
            _connection: ManagerConnection::WebsocketServer(WebsocketServerConnection {
                _thread_join_handle: thread_join_handle,
            }),
            shared: Rc::new(RefCell::new(Shared::default())),
        }
    }

    /// Adding button to side-panel.
    pub fn add_button(&self, label: String) -> UiButton {
        UiButton::new(self.shared.clone(), label)
    }

    /// Adds boolean as a checkbox to side-panel.
    pub fn add_bool(&self, label: String, value: bool) -> UiVar<bool> {
        UiVar::<bool>::new(self.shared.clone(), label, value)
    }

    /// Adds number [i32, i64, f32, f64] as a read-only text box to side-panel.
    pub fn add_number<T: common::Number>(&self, label: String, value: T) -> UiVar<T> {
        UiVar::<T>::new(self.shared.clone(), label, value)
    }

    /// Adds number [i32, i64, f32, f64] as a slider to side-panel.
    pub fn add_ranged_value<T: common::Number>(
        &self,
        label: String,
        value: T,
        (min, max): (T, T),
    ) -> UiRangedVar<T> {
        UiRangedVar::<T>::new(self.shared.clone(), label, value, (min, max))
    }

    /// Adds enum as combo box box to side-panel.
    pub fn add_enum<
        T: Clone + std::fmt::Debug + ToString + strum::VariantNames + std::str::FromStr + PartialEq,
    >(
        &self,
        label: String,
        value: T,
    ) -> UiEnum<T> {
        UiEnum::<T>::new(self.shared.clone(), label, value)
    }

    /// Adds a new 2d widget to the main panel.
    pub fn add_widget2(
        &self,
        label: String,
        image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    ) -> UiWidget2 {
        UiWidget2::new(self.shared.clone(), label, image)
    }

    /// Adds a new 3d widget to the main panel.
    pub fn add_widget3(&self, label: String) -> UiWidget3 {
        UiWidget3::new(self.shared.clone(), label)
    }

    /// Sync call to update [Manager] with [super::gui::GuiLoop]. Should be called repeatably, e.g.
    /// in a loop.
    ///
    /// Example
    /// ``` no_run
    /// vviz::app::spawn(vviz::app::VVizMode::Local, |mut manager: vviz::manager::Manager| {
    ///     let mut ui_a_button = manager.add_button("a button".to_string());
    ///     loop {
    ///        if ui_a_button.was_pressed() {
    ///           println!("a button pressed");
    ///         }
    ///         manager.sync_with_gui();
    ///     }
    /// });
    /// ```
    pub fn sync_with_gui(&mut self) {
        loop {
            let maybe_front = self.shared.borrow_mut().message_queue.pop_front();
            if maybe_front.is_none() {
                break;
            }
            self.to_gui_loop_sender.send(maybe_front.unwrap()).unwrap();
        }

        for m in self.from_gui_loop_receiver.try_iter() {
            m.update(&mut self.shared.borrow_mut().components);
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}
