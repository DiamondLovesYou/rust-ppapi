// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::mem::transmute;

use libc;

use ffi;
use ppb::{get_messaging_opt};

use {Code, MessageLoop, Instance, AnyVar, ToVar, Resource};

pub trait MessageHandling: Send + Sync {
    fn register<T>(&self, handler: T) -> Code<()>
        where T: MessageHandler;
    fn register_on_loop<T>(&self, handler: T, msg_loop: MessageLoop) -> Code<()>
        where T: MessageHandler + Send
    {
        msg_loop
            .post_work(move |_| {
                // TODO
                let _ = self.register(handler);
            }, 0)
            .map_ok(|_| () )
    }
    fn unregister(&self);
}
impl MessageHandling for Instance {
    fn register<T>(&self, handler: T) -> Code<()>
        where T: MessageHandler
    {
        let interface = if let Some(i) = get_messaging_opt() {
            i
        } else {
            return Code::NoInterface;
        };
        let interface = interface.RegisterMessageHandler.unwrap();

        let ffi = ffi::Struct_PPP_MessageHandler_0_2 {
            HandleMessage: Some(handle_message::<T>),
            HandleBlockingMessage: Some(handle_blocking_message::<T> ),
            Destroy: Some(destroy::<T>),
        };
        let internal = Handler {
            handler: handler,
            ffi: ffi,
        };
        let mut handler = Box::new(internal);

        handler.handler.registered();

        let ffi_ptr = unsafe {
            let ptr: *const *const ffi::Struct_PPP_MessageHandler_0_2 =
                transmute(&handler);
            *ptr
        };
        let handler_ptr = unsafe { transmute(handler) };

        let code = interface(Instance::current().unwrap(), handler_ptr, ffi_ptr,
                             MessageLoop::current().unwrap().unwrap());
        let code: Code = From::from(code);
        code.map_ok(|_| () )
    }
    fn unregister(&self) {
        let interface = if let Some(i) = get_messaging_opt() {
            i
        } else {
            return;
        };
        let interface = interface.UnregisterMessageHandler.unwrap();
        interface(self.unwrap());
    }
}

pub trait MessageHandler {
    fn registered(&mut self);
    fn async_message(&mut self, msg: AnyVar);
    fn sync_message(&mut self, msg: AnyVar) -> AnyVar;
    fn unregistered(self);
}

struct Handler<T> {
    handler: T,
    ffi: ffi::Struct_PPP_MessageHandler_0_2,
}

thread_local!(static HANDLERS: Option<RefCell<Vec<Instance>>> = Default::default());

extern fn handle_message<T>(_instance: ffi::PP_Instance,
                            user_data: *mut libc::c_void,
                            message: *const ffi::Struct_PP_Var)
    where T: MessageHandler
{
    let handler: &mut Handler<T> = unsafe { transmute(user_data) };
    let var = unsafe { *message };
    let var = AnyVar::new_bumped(var); // TODO var refs.
    handler.handler.async_message(var);
}

extern fn handle_blocking_message<T>(_instance: ffi::PP_Instance,
                                     user_data: *mut libc::c_void,
                                     message: *const ffi::Struct_PP_Var,
                                     response: *mut ffi::Struct_PP_Var)
    where T: MessageHandler
{
    let handler: &mut Handler<T> = unsafe { transmute(user_data) };
    let var = unsafe { *message };
    let var = AnyVar::new_bumped(var); // TODO var refs.
    let ret = handler.handler.sync_message(var);

    unsafe {
        *response = ret.to_var();
    }
}

extern fn destroy<T>(_instance: ffi::PP_Instance,
                     user_data: *mut libc::c_void)
    where T: MessageHandler
{
    let handler: Box<Handler<T>> = unsafe { transmute(user_data) };
    handler.handler.unregistered();
}

/// Ensures the local handlers gets unregistered and cleaned up before the
/// message loop terminates.
#[doc(hidden)]
pub fn unregister_handlers(queue: &MessageLoop) {
    fn _unregister_handlers(_: Code<()>) {
        HANDLERS.with(|h| {
            let h = if let &Some(ref h) = h {
                h
            } else {
                return;
            };

            let mut b = h.borrow_mut();
            for instance in b.iter() {
                instance.unregister();
            }
            b.clear();
        })
    }
    queue.post_work(_unregister_handlers, 0)
        .unwrap();
}
