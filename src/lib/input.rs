// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Resource, ResourceType, Ticks, FloatPoint,
            StringVar, Point, TouchPoint};
use super::{ppb, ffi};
use ppb::{InputEventIf, KeyboardInputEventIf, MouseInputEventIf,
          TouchInputEventIf, WheelInputEventIf};
use collections::enum_set;
use collections::enum_set::{CLike, EnumSet};
use std::{iter, intrinsics, clone, hash};

#[deriving(Hash, Eq, PartialEq, Show)] pub struct KeyboardInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct MouseInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct WheelInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct TouchInputEvent(ffi::PP_Resource);
#[deriving(Eq, Show)]
pub struct IMEInputEvent {
    res: ffi::PP_Resource,
    pub string: String,
    segments_len: uint,
}

impl_resource_for!(TouchInputEvent ResourceType::TouchInputEventRes);
impl_resource_for!(WheelInputEvent ResourceType::WheelInputEventRes);
impl_resource_for!(MouseInputEvent ResourceType::MouseInputEventRes);
impl_resource_for!(KeyboardInputEvent ResourceType::KeyboardInputEventRes);

impl IMEInputEvent {
    pub fn new(res: ffi::PP_Resource) -> IMEInputEvent {
        let var = (ppb::get_ime_event().GetText.unwrap())(res);
        let string = StringVar::new_from_var(var).to_string();
        let seg_len = (ppb::get_ime_event().GetSegmentNumber.unwrap())(res);
        IMEInputEvent {
            res: res,
            string: string,
            segments_len: seg_len as uint,
        }
    }
}
impl Resource for IMEInputEvent {
    fn unwrap(&self) -> ffi::PP_Resource {
        self.res
    }
    fn type_of(&self) -> ResourceType {
        super::ResourceType::IMEInputEventRes
    }
}
impl PartialEq for IMEInputEvent {
    fn eq(&self, rhs: &IMEInputEvent) -> bool {
        self.res == rhs.res
    }
}
impl<T: hash::Writer> hash::Hash<T> for IMEInputEvent {
    fn hash(&self, s: &mut T) {
        self.res.hash(s)
    }
}

#[deriving(Clone, Show)]
pub enum Class {
    Keyboard(Event<KeyboardInputEvent, KeyboardEvent>),
    Mouse   (Event<MouseInputEvent,    MouseEvent>),
    Wheel   (Event<WheelInputEvent,    WheelEvent>),
    Touch   (Event<TouchInputEvent,    TouchInputEvent>),
    IME     (Event<IMEInputEvent,      IMEInputEvent>),
}

pub type KeyboardClassEvent = Event<KeyboardInputEvent, KeyboardEvent>;
pub type MouseClassEvent    = Event<MouseInputEvent,    MouseEvent>;
pub type WheelClassEvent    = Event<WheelInputEvent,    WheelEvent>;
pub type TouchClassEvent    = Event<TouchInputEvent,    TouchInputEvent>;
pub type IMEClassEvent      = Event<IMEInputEvent,      IMEInputEvent>;

impl Class {
    pub fn new<T: InputEvent + Resource + 'static>(res: T) -> Class {
        let ticks = res.timestamp();
        let modifiers = res.modifiers();

        // Wraps a (safe) hack saving us from cloning res.
        // This is safe because it will only get called when T && U
        // are actually the same. We need this so as to bypass transmute
        // on value types (which fails when the types aren't the same
        // size).
        fn cast_to_expected<T: 'static, U: 'static>(mut res: T) -> U {
            use core::mem::{replace, uninitialized};
            use core::mem::transmute;
            use std::intrinsics::type_id;
            unsafe {
                // This *should* never fail, but it's here just in case:
                assert_eq!(type_id::<T>(), type_id::<U>());
                let res_ptr: &mut U = transmute(&mut res);
                replace(res_ptr, uninitialized())
            }
        }

        let input_event = ppb::get_input_event();
        let mouse_event = ppb::get_mouse_event();
        let kb_event = ppb::get_keyboard_event();
        //let wheel_event = ppb::get_wheel_event();
        //let ime_event = ppb::get_ime_event();
        //let touch_event = ppb::get_touch_event();

        match input_event.type_of(&res.unwrap()) {
            ffi::PP_INPUTEVENT_TYPE_MOUSEDOWN => {
                Class::Mouse(Event {
                    event: MouseEvent::Press(Press::Down,
                                      MouseClickEvent {
                                          point: mouse_event.point(&res.unwrap()),
                                          button: MouseButton::from_ffi
                                              (mouse_event.button(&res.unwrap())),
                                          click_count: mouse_event.click_count(&res.unwrap()),
                                      }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_MOUSEUP => {
                Class::Mouse(Event {
                    event: MouseEvent::Press(Press::Up,
                                      MouseClickEvent {
                                          point: mouse_event.point(&res.unwrap()),
                                          button: MouseButton::from_ffi
                                              (mouse_event.button(&res.unwrap())),
                                          click_count: mouse_event.click_count(&res.unwrap()),
                                      }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_MOUSEMOVE => {
                Class::Mouse(Event {
                    event: MouseEvent::Move(Move::Move,
                                     MouseMoveEvent {
                                         point: mouse_event.point(&res.unwrap()),
                                         delta: mouse_event.delta(&res.unwrap()),
                                         click_count: mouse_event.click_count(&res.unwrap()),
                                     }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_MOUSEENTER => {
                Class::Mouse(Event {
                    event: MouseEvent::Move(Move::Enter,
                                     MouseMoveEvent {
                                         point: mouse_event.point(&res.unwrap()),
                                         delta: mouse_event.delta(&res.unwrap()),
                                         click_count: mouse_event.click_count(&res.unwrap()),
                                     }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_MOUSELEAVE => {
                Class::Mouse(Event {
                    event: MouseEvent::Move(Move::Leave,
                                     MouseMoveEvent {
                                         point: mouse_event.point(&res.unwrap()),
                                         delta: mouse_event.delta(&res.unwrap()),
                                         click_count: mouse_event.click_count(&res.unwrap()),
                                     }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_WHEEL => {
                unreachable!()
            }
            ffi::PP_INPUTEVENT_TYPE_KEYDOWN => {
                Class::Keyboard(Event {
                    event: KeyboardEvent::Press(Press::Down,
                                    kb_event.key_code(&res.unwrap()) as i32),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_KEYUP => {
                Class::Keyboard(Event {
                    event: KeyboardEvent::Press(Press::Up,
                                    kb_event.key_code(&res.unwrap()) as i32),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_CHAR => {
                let char_var = StringVar((unsafe {
                    ffi::id_from_var(kb_event.text(&res.unwrap()))
                }) as i64);
                let str = char_var.to_string();
                if str.len() != 1 {
                    warn!("character input event does not have a length of one: \
                          \"{}\"", str)
                }
                Class::Keyboard(Event {
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                    event: KeyboardEvent::Char(str.as_slice().char_at(0)),
                })
            }
            ffi::PP_INPUTEVENT_TYPE_CONTEXTMENU => {
                Class::Mouse(Event {
                    event: MouseEvent::ContextMenu(MouseClickEvent {
                        point: mouse_event.point(&res.unwrap()),
                        button: MouseButton::Right,
                        click_count: mouse_event.click_count(&res.unwrap()),
                    }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            _ => {
                panic!("invalid input event type")
            }
        }
    }
}
impl Resource for Class {
    fn unwrap(&self) -> ffi::PP_Resource {
        match self {
            &Class::Keyboard(ref e) => e.res.unwrap(),
            &Class::Mouse   (ref e) => e.res.unwrap(),
            &Class::Wheel   (ref e) => e.res.unwrap(),
            &Class::Touch   (ref e) => e.res.unwrap(),
            &Class::IME     (ref e) => e.res.unwrap(),
        }
    }
    fn type_of(&self) -> ResourceType {
        match self {
            &Class::Keyboard(ref e) => e.res.type_of(),
            &Class::Mouse   (ref e) => e.res.type_of(),
            &Class::Wheel   (ref e) => e.res.type_of(),
            &Class::Touch   (ref e) => e.res.type_of(),
            &Class::IME     (ref e) => e.res.type_of(),
        }
    }
}
impl InputEvent for Class {
    fn modifiers(&self) -> Modifiers {
        match self {
            &Class::Keyboard(Event { mods: ref m, .. }) |
            &Class::Mouse   (Event { mods: ref m, .. }) |
            &Class::Wheel   (Event { mods: ref m, .. }) |
            &Class::Touch   (Event { mods: ref m, .. }) |
            &Class::IME     (Event { mods: ref m, .. }) => m.clone(),
        }
    }
    fn timestamp(&self) -> Ticks {
        match self {
            &Class::Keyboard(Event { timestamp: ts, .. }) |
            &Class::Mouse   (Event { timestamp: ts, .. }) |
            &Class::Wheel   (Event { timestamp: ts, .. }) |
            &Class::Touch   (Event { timestamp: ts, .. }) |
            &Class::IME     (Event { timestamp: ts, .. }) => ts,
        }
    }
}
#[deriving(Clone, Show)]
pub struct Event<Res, Class> {
    pub res: Res,
    pub timestamp: Ticks,
    pub mods: Modifiers,
    pub event: Class,
}
impl<Res: Resource, Class> Resource for Event<Res, Class> {
    fn unwrap(&self) -> ffi::PP_Resource {
        self.res.unwrap()
    }
    fn type_of(&self) -> ResourceType {
        self.res.type_of()
    }
}
impl<Res: Resource, Class> InputEvent for Event<Res, Class> {
    fn modifiers(&self) -> Modifiers {
        self.mods.clone()
    }
    fn timestamp(&self) -> Ticks {
        self.timestamp
    }
}
#[deriving(Clone, Hash, Eq, PartialEq, Show, Copy)]
pub enum Modifiers_ {
    ShiftKey,
    ControlKey,
    AltKey,
    MetaKey,
    IsKeyPad,
    IsAutoRepeat,
    LeftButtonDown,
    MiddleButtonDown,
    RightButtonDown,
    CapsLockKey,
    NumLockKey,
    IsLeft,
    IsRight,
}
impl CLike for Modifiers_ {
    fn to_uint(&self) -> uint {
        match self {
            &Modifiers_::ShiftKey         => 0,
            &Modifiers_::ControlKey       => 1,
            &Modifiers_::AltKey           => 2,
            &Modifiers_::MetaKey          => 3,
            &Modifiers_::IsKeyPad         => 4,
            &Modifiers_::IsAutoRepeat     => 5,
            &Modifiers_::LeftButtonDown   => 6,
            &Modifiers_::MiddleButtonDown => 7,
            &Modifiers_::RightButtonDown  => 8,
            &Modifiers_::CapsLockKey      => 9,
            &Modifiers_::NumLockKey       => 10,
            &Modifiers_::IsLeft           => 11,
            &Modifiers_::IsRight          => 12,
        }
    }
    fn from_uint(v: uint) -> Modifiers_ {
        match v {
            0  => Modifiers_::ShiftKey,
            1  => Modifiers_::ControlKey,
            2  => Modifiers_::AltKey,
            3  => Modifiers_::MetaKey,
            4  => Modifiers_::IsKeyPad,
            5  => Modifiers_::IsAutoRepeat,
            6  => Modifiers_::LeftButtonDown,
            7  => Modifiers_::MiddleButtonDown,
            8  => Modifiers_::RightButtonDown,
            9  => Modifiers_::CapsLockKey,
            10 => Modifiers_::NumLockKey,
            11 => Modifiers_::IsLeft,
            12 => Modifiers_::IsRight,
            _  => unreachable!(),
        }
    }
}
pub type Modifiers = EnumSet<Modifiers_>;
fn modifiers_from_bitset(set: u32) -> Modifiers {
    let mut e: Modifiers = enum_set::EnumSet::new();
    if set & 0b0000000000001 != 0 { e.insert(Modifiers_::ShiftKey); }
    if set & 0b0000000000010 != 0 { e.insert(Modifiers_::ControlKey); }
    if set & 0b0000000000100 != 0 { e.insert(Modifiers_::AltKey); }
    if set & 0b0000000001000 != 0 { e.insert(Modifiers_::MetaKey); }
    if set & 0b0000000010000 != 0 { e.insert(Modifiers_::IsKeyPad); }
    if set & 0b0000000100000 != 0 { e.insert(Modifiers_::IsAutoRepeat); }
    if set & 0b0000001000000 != 0 { e.insert(Modifiers_::LeftButtonDown); }
    if set & 0b0000010000000 != 0 { e.insert(Modifiers_::MiddleButtonDown); }
    if set & 0b0000100000000 != 0 { e.insert(Modifiers_::RightButtonDown); }
    if set & 0b0001000000000 != 0 { e.insert(Modifiers_::CapsLockKey); }
    if set & 0b0010000000000 != 0 { e.insert(Modifiers_::NumLockKey); }
    if set & 0b0100000000000 != 0 { e.insert(Modifiers_::IsLeft); }
    if set & 0b1000000000000 != 0 { e.insert(Modifiers_::IsRight); }
    e
}
#[deriving(Clone, Hash, Eq, PartialEq, Show, Copy)]
pub enum Move {
    Enter,
    Leave,
    Move,
}

#[deriving(Clone, Hash, Eq, PartialEq, Show, Copy)]
pub enum Press {
    Down,
    Up,
}
#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub enum MouseEvent {
    Press       (Press, MouseClickEvent),
    ContextMenu (MouseClickEvent),
    Move        (Move,  MouseMoveEvent),
}
impl MouseEvent {
    fn point(&self) -> FloatPoint {
        match self {
            &MouseEvent::Press(_, MouseClickEvent {
                point, ..
            }) | &MouseEvent::ContextMenu(MouseClickEvent {
                point, ..
            }) | &MouseEvent::Move(_, MouseMoveEvent {
                point, ..
            }) => point,
        }
    }
}
#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
impl MouseButton {
    fn from_ffi(v: i32) -> MouseButton {
        match v {
            ffi::PP_INPUTEVENT_MOUSEBUTTON_LEFT   => MouseButton::Left,
            ffi::PP_INPUTEVENT_MOUSEBUTTON_MIDDLE => MouseButton::Middle,
            ffi::PP_INPUTEVENT_MOUSEBUTTON_RIGHT  => MouseButton::Right,
            _ => unreachable!(),
        }
    }
}
#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub struct MouseClickEvent {
    point: FloatPoint,
    button: MouseButton,
    click_count: i32,
}
#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub struct MouseMoveEvent {
    point: FloatPoint,
    delta: FloatPoint,
    click_count: i32,
}

#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub enum KeyboardEvent {
    Press(Press, i32),
    Char(char),
}

#[deriving(Clone, Eq, PartialEq, Show, Copy)]
pub struct WheelEvent {
    delta: FloatPoint,
    ticks: FloatPoint,
    by_page: bool,
}

pub trait InputEvent {
    fn modifiers(&self) -> Modifiers;
    fn timestamp(&self) -> Ticks;
}

macro_rules! impl_input_event_for(
    ($ty:ty) => (
        impl InputEvent for $ty {
            fn modifiers(&self) -> Modifiers {
                let bitset = ppb::get_input_event().modifiers(&self.unwrap());
                modifiers_from_bitset(bitset)
            }
            fn timestamp(&self) -> Ticks {
                ppb::get_input_event().timestamp(&self.unwrap())
            }
        }
    )
);

impl_input_event_for!(KeyboardInputEvent);
impl_input_event_for!(MouseInputEvent);
impl_input_event_for!(WheelInputEvent);
impl_input_event_for!(TouchInputEvent);
impl_input_event_for!(IMEInputEvent);

impl KeyboardInputEvent {
    pub fn get_key_code(&self) -> u32 {
        (ppb::get_keyboard_event().GetKeyCode.unwrap())(self.unwrap())
    }
    pub fn get_char_text(&self) -> StringVar {
        let var = (ppb::get_keyboard_event().GetCharacterText.unwrap())(self.unwrap());
        StringVar::new_from_var(var)
    }
}
impl MouseInputEvent {
    pub fn get_button(&self) -> ffi::PP_InputEvent_MouseButton {
        (ppb::get_mouse_event().GetButton.unwrap())(self.unwrap())
    }
    pub fn get_position(&self) -> Point {
        (ppb::get_mouse_event().GetPosition.unwrap())(self.unwrap())
    }
    pub fn get_click_count(&self) -> i32 {
        (ppb::get_mouse_event().GetClickCount.unwrap())(self.unwrap())
    }
    pub fn get_movement(&self) -> Point {
        (ppb::get_mouse_event().GetMovement.unwrap())(self.unwrap())
    }
}
impl WheelInputEvent {
    pub fn get_delta(&self) -> FloatPoint {
        (ppb::get_wheel_event().GetDelta.unwrap())(self.unwrap())
    }
    pub fn get_ticks(&self) -> FloatPoint {
        (ppb::get_wheel_event().GetTicks.unwrap())(self.unwrap())
    }
    pub fn get_scroll_by_page(&self) -> bool {
        (ppb::get_wheel_event().GetScrollByPage.unwrap())(self.unwrap()) != ffi::PP_FALSE
    }
}
#[deriving(Clone, Eq, PartialEq, Hash, Show, Copy)]
pub enum TouchListType {
    Touches,
    Delta,
    Target,
}
impl TouchListType {
    fn to_ffi(&self) -> ffi::PP_TouchListType {
        match self {
            &TouchListType::Touches => ffi::PP_TOUCHLIST_TYPE_TOUCHES,
            &TouchListType::Delta => ffi::PP_TOUCHLIST_TYPE_CHANGEDTOUCHES,
            &TouchListType::Target => ffi::PP_TOUCHLIST_TYPE_TARGETTOUCHES,
        }
    }
}
#[deriving(Clone)]
pub struct TouchList {
    event: TouchInputEvent,
    list_type: TouchListType,
}
#[deriving(Clone)]
pub struct TouchListIterator {
    list: TouchList,
    current: uint,
    size: uint,
}
impl TouchInputEvent {
    pub fn get_touch_list(&self, list_type: TouchListType) -> TouchList {
        TouchList{
            event: self.clone(),
            list_type: list_type,
        }
    }
}
impl TouchList {
    pub fn get(&self, index: uint) -> TouchPoint {
        ppb::get_touch_event().by_index
            (&self.event.unwrap(),
             self.list_type.to_ffi(),
             index as u32)
    }
    pub fn iter(&self) -> TouchListIterator {
        TouchListIterator{
            list: self.clone(),
            current: 0,
            size: self.len(),
        }
    }
    pub fn find_id(&self, id: u32) -> Option<TouchPoint> {
        if id < self.len() as u32 {
            Some(ppb::get_touch_event().by_id
                 (&self.event.unwrap(),
                  self.list_type.to_ffi(),
                  id))
        } else {
            None
        }
    }
    pub fn len(&self) -> uint {
        (ppb::get_touch_event().GetTouchCount.unwrap())
            (self.event.unwrap(),
             self.list_type.to_ffi()) as uint
    }
}
impl iter::Iterator<TouchPoint> for TouchListIterator {
    fn next(&mut self) -> Option<TouchPoint> {
        if self.current >= self.size { None }
        else                         { self.current += 1;
                                       Some(self.list.get(self.current - 1)) }
    }
}
impl iter::RandomAccessIterator<TouchPoint> for TouchListIterator {
    fn indexable(&self) -> uint {
        self.size
    }
    fn idx(&mut self, index: uint) -> Option<TouchPoint> {
        if index < self.size { Some(self.list.get(index)) }
        else                 { None }
    }
}

impl IMEInputEvent {
    pub fn segments_len(&self) -> uint {
        self.segments_len
    }
    pub fn segment_offset(&self, index: uint) -> Option<(uint, uint)> {
        if index < self.segments_len() {
            let interface = ppb::get_ime_event();
            let start = (interface.GetSegmentOffset.unwrap())
                (self.unwrap(),
                 index as u32);
            let end = (interface.GetSegmentOffset.unwrap())
                (self.unwrap(),
                 (index + 1) as u32);
            Some((start as uint, end as uint))
        } else {
            None
        }
    }
    pub fn segment_str<'a>(&'a self, index: uint) -> Option<&'a str> {
        if index < self.segments_len() {
            let segment = self.segment_offset(index);
            let (start, end) = segment.expect("WAT. #1");
            Some(self.string.as_slice().slice(start, end))
        } else {
            None
        }
    }
    pub fn segment_iter<'a>(&'a self) -> IMESegmentIterator<'a> {
        IMESegmentIterator{
            event: self,
            current: 0,
        }
    }
    pub fn selection_offset(&self) -> (u32, u32) {
        let mut start: u32 = unsafe { intrinsics::uninit() };
        let mut end: u32 = unsafe { intrinsics::uninit() };


        let start_ptr: *mut u32 = &mut start as *mut u32;
        let end_ptr: *mut u32 = &mut end as *mut u32;
        (ppb::get_ime_event().GetSelection.unwrap())
            (self.unwrap(),
             start_ptr,
             end_ptr);

        let start = start;
        let end   = end;

        (start, end)
    }
    pub fn selection_str<'a>(&'a self) -> &'a str {
        let (start, end) = self.selection_offset();
        self.string.as_slice().slice(start as uint, end as uint)
    }
    pub fn target_segment_index(&self) -> Option<uint> {
        match (ppb::get_ime_event().GetTargetSegment.unwrap())
            (self.unwrap()) {
            -1 => None,
            index => Some(index as uint),
        }
    }
    pub fn target_segment_str<'a>(&'a self) -> Option<&'a str> {
        match self.target_segment_index() {
            Some(index) => self.segment_str(index as uint),
            None => None,
        }
    }
}
pub struct IMESegmentIterator<'a> {
    event: &'a IMEInputEvent,
    current: uint,
}
impl<'a> clone::Clone for IMESegmentIterator<'a> {
    fn clone(&self) -> IMESegmentIterator<'a> {
        IMESegmentIterator {
            event: self.event,
            current: self.current,
        }
    }
}
impl<'a> iter::Iterator<&'a str> for IMESegmentIterator<'a> {
    fn next(&mut self) -> Option<&'a str> {
        self.current += 1;
        self.event.segment_str(self.current - 1)
    }
}
impl<'a> iter::RandomAccessIterator<&'a str> for IMESegmentIterator<'a> {
    fn indexable(&self) -> uint {
        self.event.segments_len()
    }
    fn idx(&mut self, index: uint) -> Option<&'a str> {
        self.event.segment_str(index)
    }
}
