use linked_hash_map::LinkedHashMap;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc;

use super::common;

pub struct Stuff {
    components: LinkedHashMap<String, Box<dyn common::Component>>,
    //widgets: LinkedHashMap<String, Box<dyn common::Widget>>,
    message_queue: VecDeque<Box<dyn common::ToGuiLoopMessage>>,
}

impl Stuff {
    pub fn default() -> Self {
        Self {
            components: LinkedHashMap::new(),
            //widgets: LinkedHashMap::new(),
            message_queue: VecDeque::new(),
        }
    }
}

pub struct Manager {
    to_gui_loop_sender: mpsc::Sender<Box<dyn common::ToGuiLoopMessage>>,
    from_gui_loop_receiver: mpsc::Receiver<Box<dyn common::FromGuiLoopMessage>>,

    stuff: Rc<RefCell<Stuff>>,
}

pub struct UiEnum<T> {
    pub stuff: Rc<RefCell<Stuff>>,
    label: String,
    cache: T,
}

impl<
        T: std::fmt::Debug + ToString + strum::VariantNames + std::str::FromStr + PartialEq + Clone,
    > UiEnum<T>
{
    pub fn new(stuff: Rc<RefCell<Stuff>>, label: String, value: T) -> Self {
        let mut values_map = std::vec::Vec::new();
        for str in T::VARIANTS {
            let owned_str = str.to_string();
            values_map.push(owned_str);
        }

        stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::AddEnumStringRepr {
                label: label.clone(),
                value: value.to_string(),
                values: values_map.clone(),
            }));

        stuff.borrow_mut().components.insert(
            label.clone(),
            Box::new(common::EnumStringRepr {
                value: value.to_string(),
                values: values_map,
            }),
        );
        Self {
            stuff,
            label,
            cache: value,
        }
    }

    pub fn get_value(&mut self) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let string_repr = self
            .stuff
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

    pub fn get_new_value(&mut self) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let string_repr = self
            .stuff
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

pub struct UiVar<T> {
    pub stuff: Rc<RefCell<Stuff>>,
    label: String,
    cache: T,
}

impl UiVar<bool> {
    pub fn new(stuff: Rc<RefCell<Stuff>>, label: String, value: bool) -> Self {
        stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::AddVar::<bool> {
                label: label.clone(),
                value,
            }));
        stuff
            .borrow_mut()
            .components
            .insert(label.clone(), Box::new(common::Var::<bool> { value }));
        Self {
            stuff,
            label,
            cache: value,
        }
    }

    pub fn get_value(&mut self) -> bool {
        let value = self
            .stuff
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

    pub fn get_new_value(&mut self) -> Option<bool> {
        let value = self
            .stuff
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

impl<T: common::Numbers> UiVar<T> {
    pub fn new(stuff: Rc<RefCell<Stuff>>, label: String, value: T) -> Self {
        stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::AddVar::<T> {
                label: label.clone(),
                value,
            }));
        stuff
            .borrow_mut()
            .components
            .insert(label.clone(), Box::new(common::Var::<T> { value }));
        Self {
            stuff,
            label,
            cache: value,
        }
    }

    pub fn get_value(&mut self) -> T {
        let value = self
            .stuff
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

    pub fn get_new_value(&mut self) -> Option<T> {
        let value = self
            .stuff
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

pub struct UiRangedVar<T> {
    pub stuff: Rc<RefCell<Stuff>>,
    label: String,
    cache: T,
}

impl<T: common::Numbers> UiRangedVar<T> {
    pub fn new(stuff: Rc<RefCell<Stuff>>, label: String, value: T, min: T, max: T) -> Self {
        stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::AddRangedVar::<T> {
                label: label.clone(),
                value,
                min,
                max,
            }));
        stuff.borrow_mut().components.insert(
            label.clone(),
            Box::new(common::RangedVar::<T> { value, min, max }),
        );
        Self {
            stuff,
            label,
            cache: value,
        }
    }

    pub fn get_value(&mut self) -> T {
        let value = self
            .stuff
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

    pub fn get_new_value(&mut self) -> Option<T> {
        let value = self
            .stuff
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

pub struct UiWidget3 {
    pub label: String,
    pub stuff: Rc<RefCell<Stuff>>,
}

impl UiWidget3 {
    pub fn new(stuff: Rc<RefCell<Stuff>>, label: String) -> Self {
        stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::AddWidget3 {
                label: label.clone(),
            }));

        Self { label, stuff }
    }

    pub fn place_entity(&self, label: String, entity: common::Entity3) {
        self.stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::PlaceEntity {
                widget_label: self.label.clone(),
                named_entity: common::NamedEntity3 {
                    label,
                    entity,
                    scene_pose_entity: nalgebra::Isometry3::<f32>::identity(),
                },
            }));
    }

    pub fn place_entity_at(
        &self,
        label: String,
        entity: common::Entity3,
        scene_pose_entity: nalgebra::Isometry3<f32>,
    ) {
        self.stuff
            .borrow_mut()
            .message_queue
            .push_back(Box::new(common::PlaceEntity {
                widget_label: self.label.clone(),
                named_entity: common::NamedEntity3 {
                    label,
                    entity,
                    scene_pose_entity,
                },
            }));
    }
}

impl Manager {
    pub fn new(
        to_gui_loop_sender: mpsc::Sender<Box<dyn common::ToGuiLoopMessage>>,
        from_gui_loop_receiver: mpsc::Receiver<Box<dyn common::FromGuiLoopMessage>>,
    ) -> Self {
        Self {
            to_gui_loop_sender,
            from_gui_loop_receiver,
            stuff: Rc::new(RefCell::new(Stuff::default())),
        }
    }

    pub fn add_bool(&self, label: String, value: bool) -> UiVar<bool> {
        UiVar::<bool>::new(self.stuff.clone(), label, value)
    }

    pub fn add_i32(&self, label: String, value: i32) -> UiVar<i32> {
        UiVar::<i32>::new(self.stuff.clone(), label, value)
    }

    pub fn add_i64(&self, label: String, value: i64) -> UiVar<i64> {
        UiVar::<i64>::new(self.stuff.clone(), label, value)
    }

    pub fn add_f32(&self, label: String, value: f32) -> UiVar<f32> {
        UiVar::<f32>::new(self.stuff.clone(), label, value)
    }

    pub fn add_f64(&self, label: String, value: f64) -> UiVar<f64> {
        UiVar::<f64>::new(self.stuff.clone(), label, value)
    }

    pub fn add_ranged_i32(
        &self,
        label: String,
        value: i32,
        min: i32,
        max: i32,
    ) -> UiRangedVar<i32> {
        UiRangedVar::<i32>::new(self.stuff.clone(), label, value, min, max)
    }

    pub fn add_ranged_i64(
        &self,
        label: String,
        value: i64,
        min: i64,
        max: i64,
    ) -> UiRangedVar<i64> {
        UiRangedVar::<i64>::new(self.stuff.clone(), label, value, min, max)
    }
    pub fn add_ranged_f32(
        &self,
        label: String,
        value: f32,
        min: f32,
        max: f32,
    ) -> UiRangedVar<f32> {
        UiRangedVar::<f32>::new(self.stuff.clone(), label, value, min, max)
    }

    pub fn add_ranged_f64(
        &self,
        label: String,
        value: f64,
        min: f64,
        max: f64,
    ) -> UiRangedVar<f64> {
        UiRangedVar::<f64>::new(self.stuff.clone(), label, value, min, max)
    }

    pub fn add_enum<
        T: Clone + std::fmt::Debug + ToString + strum::VariantNames + std::str::FromStr + PartialEq,
    >(
        &self,
        label: String,
        value: T,
    ) -> UiEnum<T> {
        UiEnum::<T>::new(self.stuff.clone(), label, value)
    }

    pub fn add_widget3(&self, label: String) -> UiWidget3 {
        UiWidget3::new(self.stuff.clone(), label)
    }

    pub fn sync_with_gui(&mut self) {
        loop {
            let maybe_front = self.stuff.borrow_mut().message_queue.pop_front();
            if maybe_front.is_none() {
                break;
            }
            self.to_gui_loop_sender.send(maybe_front.unwrap()).unwrap();
        }

        for m in self.from_gui_loop_receiver.try_iter() {
            m.update(&mut self.stuff.borrow_mut().components);
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}
