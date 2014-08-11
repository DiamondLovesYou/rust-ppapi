use super::{KeyboardInputEvent,
            MouseInputEvent,
            WheelInputEvent,
            TouchInputEvent,
            IMEInputEvent};
use super::{Resource, ResourceType, Ticks, FloatPoint,
            StringVar, Point, TouchPoint};
use super::{ppb, ffi};
use collections::enum_set;
use collections::enum_set::{CLike, EnumSet};
use std::{collections, iter, intrinsics, clone};

#[deriving(Clone, Show)]
pub enum Class {
    KeyboardClass(Event<super::KeyboardInputEvent, KeyboardEvent>),
    MouseClass   (Event<super::MouseInputEvent,    MouseEvent>),
    WheelClass   (Event<super::WheelInputEvent,    WheelEvent>),
    TouchClass   (Event<super::TouchInputEvent,    super::TouchInputEvent>),
    IMEClass     (Event<super::IMEInputEvent,      super::IMEInputEvent>),
}

pub type KeyboardClassEvent = Event<super::KeyboardInputEvent, KeyboardEvent>;
pub type MouseClassEvent    = Event<super::MouseInputEvent,    MouseEvent>;
pub type WheelClassEvent    = Event<super::WheelInputEvent,    WheelEvent>;
pub type TouchClassEvent    = Event<super::TouchInputEvent,    super::TouchInputEvent>;
pub type IMEClassEvent      = Event<super::IMEInputEvent,      super::IMEInputEvent>;

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
                MouseClass(Event {
                    event: MousePress(DownPress,
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
                MouseClass(Event {
                    event: MousePress(UpPress,
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
                MouseClass(Event {
                    event: MouseMove(MoveMove,
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
                MouseClass(Event {
                    event: MouseMove(EnterMove,
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
                MouseClass(Event {
                    event: MouseMove(LeaveMove,
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
                KeyboardClass(Event {
                    event: KeyPress(DownPress,
                                    kb_event.key_code(&res.unwrap()) as i32),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            ffi::PP_INPUTEVENT_TYPE_KEYUP => {
                KeyboardClass(Event {
                    event: KeyPress(UpPress,
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
                KeyboardClass(Event {
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                    event: KeyChar(str.as_slice().char_at(0)),
                })
            }
            ffi::PP_INPUTEVENT_TYPE_CONTEXTMENU => {
                MouseClass(Event {
                    event: MouseContextMenu(MouseClickEvent {
                        point: mouse_event.point(&res.unwrap()),
                        button: RightMouseButton,
                        click_count: mouse_event.click_count(&res.unwrap()),
                    }),
                    res: cast_to_expected(res),
                    timestamp: ticks,
                    mods: modifiers,
                })
            }
            _ => {
                fail!("invalid input event type")
            }
        }
    }
}
impl Resource for Class {
    fn unwrap(&self) -> ffi::PP_Resource {
        match self {
            &KeyboardClass(ref e) => e.res.unwrap(),
            &MouseClass   (ref e) => e.res.unwrap(),
            &WheelClass   (ref e) => e.res.unwrap(),
            &TouchClass   (ref e) => e.res.unwrap(),
            &IMEClass     (ref e) => e.res.unwrap(),
        }
    }
    fn type_of(&self) -> ResourceType {
        match self {
            &KeyboardClass(ref e) => e.res.type_of(),
            &MouseClass   (ref e) => e.res.type_of(),
            &WheelClass   (ref e) => e.res.type_of(),
            &TouchClass   (ref e) => e.res.type_of(),
            &IMEClass     (ref e) => e.res.type_of(),
        }
    }
}
impl InputEvent for Class {
    fn modifiers(&self) -> Modifiers {
        match self {
            &KeyboardClass(Event { mods: ref m, .. }) |
            &MouseClass   (Event { mods: ref m, .. }) |
            &WheelClass   (Event { mods: ref m, .. }) |
            &TouchClass   (Event { mods: ref m, .. }) |
            &IMEClass     (Event { mods: ref m, .. }) => m.clone(),
        }
    }
    fn timestamp(&self) -> Ticks {
        match self {
            &KeyboardClass(Event { timestamp: ts, .. }) |
            &MouseClass   (Event { timestamp: ts, .. }) |
            &WheelClass   (Event { timestamp: ts, .. }) |
            &TouchClass   (Event { timestamp: ts, .. }) |
            &IMEClass     (Event { timestamp: ts, .. }) => ts,
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
#[deriving(Clone, Hash, Eq, PartialEq, Show)]
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
            &ShiftKey         => 0,
            &ControlKey       => 1,
            &AltKey           => 2,
            &MetaKey          => 3,
            &IsKeyPad         => 4,
            &IsAutoRepeat     => 5,
            &LeftButtonDown   => 6,
            &MiddleButtonDown => 7,
            &RightButtonDown  => 8,
            &CapsLockKey      => 9,
            &NumLockKey       => 10,
            &IsLeft           => 11,
            &IsRight          => 12,
        }
    }
    fn from_uint(v: uint) -> Modifiers_ {
        match v {
            0  => ShiftKey,
            1  => ControlKey,
            2  => AltKey,
            3  => MetaKey,
            4  => IsKeyPad,
            5  => IsAutoRepeat,
            6  => LeftButtonDown,
            7  => MiddleButtonDown,
            8  => RightButtonDown,
            9  => CapsLockKey,
            10 => NumLockKey,
            11 => IsLeft,
            12 => IsRight,
            _  => unreachable!(),
        }
    }
}
pub type Modifiers = EnumSet<Modifiers_>;
fn modifiers_from_bitset(set: u32) -> Modifiers {
    let mut e: Modifiers = enum_set::EnumSet::empty();
    if set & 0b0000000000001 != 0 { e.add(ShiftKey) }
    if set & 0b0000000000010 != 0 { e.add(ControlKey) }
    if set & 0b0000000000100 != 0 { e.add(AltKey) }
    if set & 0b0000000001000 != 0 { e.add(MetaKey) }
    if set & 0b0000000010000 != 0 { e.add(IsKeyPad) }
    if set & 0b0000000100000 != 0 { e.add(IsAutoRepeat) }
    if set & 0b0000001000000 != 0 { e.add(LeftButtonDown) }
    if set & 0b0000010000000 != 0 { e.add(MiddleButtonDown) }
    if set & 0b0000100000000 != 0 { e.add(RightButtonDown) }
    if set & 0b0001000000000 != 0 { e.add(CapsLockKey) }
    if set & 0b0010000000000 != 0 { e.add(NumLockKey) }
    if set & 0b0100000000000 != 0 { e.add(IsLeft) }
    if set & 0b1000000000000 != 0 { e.add(IsRight) }
    e
}
#[deriving(Clone, Hash, Eq, PartialEq, Show)]
pub enum Move {
    EnterMove,
    LeaveMove,
    MoveMove,
}

#[deriving(Clone, Hash, Eq, PartialEq, Show)]
pub enum Press {
    DownPress,
    UpPress,
}
#[deriving(Clone, Eq, PartialEq, Show)]
pub enum MouseEvent {
    MousePress       (Press, MouseClickEvent),
    MouseContextMenu (MouseClickEvent),
    MouseMove        (Move,  MouseMoveEvent),
}
impl MouseEvent {
    fn point(&self) -> FloatPoint {
        match self {
            &MousePress(_, MouseClickEvent {
                point: point,
                ..
            }) | &MouseContextMenu(MouseClickEvent {
                point: point,
                ..
            }) | &MouseMove(_, MouseMoveEvent {
                point: point,
                ..
            }) => point,
        }
    }
}
#[deriving(Clone, Eq, PartialEq, Show)]
pub enum MouseButton {
    LeftMouseButton,
    MiddleMouseButton,
    RightMouseButton,
}
impl MouseButton {
    fn from_ffi(v: i32) -> MouseButton {
        match v {
            ffi::PP_INPUTEVENT_MOUSEBUTTON_LEFT   => LeftMouseButton,
            ffi::PP_INPUTEVENT_MOUSEBUTTON_MIDDLE => MiddleMouseButton,
            ffi::PP_INPUTEVENT_MOUSEBUTTON_RIGHT  => RightMouseButton,
            _ => unreachable!(),
        }
    }
}
#[deriving(Clone, Eq, PartialEq, Show)]
pub struct MouseClickEvent {
    point: FloatPoint,
    button: MouseButton,
    click_count: i32,
}
#[deriving(Clone, Eq, PartialEq, Show)]
pub struct MouseMoveEvent {
    point: FloatPoint,
    delta: FloatPoint,
    click_count: i32,    
}

#[deriving(Clone, Eq, PartialEq, Show)]
pub enum KeyboardEvent {
    KeyPress(Press, i32),
    KeyChar(char),
}

#[deriving(Clone, Eq, PartialEq, Show)]
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
)

impl_input_event_for!(super::KeyboardInputEvent)
impl_input_event_for!(super::MouseInputEvent)
impl_input_event_for!(super::WheelInputEvent)
impl_input_event_for!(super::TouchInputEvent)
impl_input_event_for!(super::IMEInputEvent)

impl super::KeyboardInputEvent {
    pub fn get_key_code(&self) -> u32 {
        (ppb::get_keyboard_event().GetKeyCode.unwrap())(self.unwrap())
    }
    pub fn get_char_text(&self) -> StringVar {
        let var = (ppb::get_keyboard_event().GetCharacterText.unwrap())(self.unwrap());
        StringVar::new_from_var(var)
    }
}
impl super::MouseInputEvent {
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
impl super::WheelInputEvent {
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
#[deriving(Clone, Eq, PartialEq, Hash, Show)]
pub enum TouchListType {
    TouchesTouchListType,
    DeltaTouchListType,
    TargetTouchListType,
}
impl TouchListType {
    fn to_ffi(&self) -> ffi::PP_TouchListType {
        match self {
            &TouchesTouchListType => ffi::PP_TOUCHLIST_TYPE_TOUCHES,
            &DeltaTouchListType => ffi::PP_TOUCHLIST_TYPE_CHANGEDTOUCHES,
            &TargetTouchListType => ffi::PP_TOUCHLIST_TYPE_TARGETTOUCHES,
        }
    }
}
#[deriving(Clone)]
pub struct TouchList {
    event: super::TouchInputEvent, 
    list_type: TouchListType,
}
#[deriving(Clone)]
pub struct TouchListIterator {
    list: TouchList,
    current: uint,
    size: uint,
}
impl super::TouchInputEvent {
    pub fn get_touch_list(&self, list_type: TouchListType) -> TouchList {
        TouchList{
            event: self.clone(),
            list_type: list_type,
        }
    }
}
impl collections::Collection for TouchList {
    fn len(&self) -> uint {
        (ppb::get_touch_event().GetTouchCount.unwrap())
            (self.event.unwrap(),
             self.list_type.to_ffi()) as uint
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

impl super::IMEInputEvent {
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
        if index < self.len() {
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
    event: &'a super::IMEInputEvent,
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
impl collections::Collection for super::IMEInputEvent {
    fn len(&self) -> uint {
        self.segments_len()
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
