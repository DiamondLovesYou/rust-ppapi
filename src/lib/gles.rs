// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(missing_docs)]

/// A module to wrap OpenGLES 2.0 functions from the PPAPI.
/// At some point I'd like to renovate this. For example by using borrowing
/// semantics to prevent a bound program from being overridden.

use std::borrow::{Cow, ToOwned};
use std::mem::{size_of, uninitialized};
use std::{ptr};
use std::default::Default;
use std::fmt;

use libc;
use libc::c_void;
use super::{Resource, CallbackArgs, Code, Rect};
use super::ppb;
use super::ppb::get_gles2;
use ffi;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Context3d(ffi::PP_Resource);

impl_clone_drop_for!(Context3d);

impl super::ContextResource for Context3d {
    fn get_device(&self) -> ffi::PP_Resource {
        self.unwrap()
    }
}

// for debugging purposes:
impl fmt::Display for Context3d {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[allow(missing_docs)] pub mod types {
    use super::super::ffi;
    pub type Enum    = ffi::GLenum;
    pub type UInt    = ffi::GLuint;
    pub type Int     = ffi::GLint;
    pub type Size    = ffi::GLsizei;
    pub type SizePtr = ffi::GLsizeiptr;
    pub type Float   = ffi::GLfloat;
    pub type Boolean = ffi::GLboolean;
    pub type Byte    = ffi::GLbyte;
    pub type ClampF  = ffi::GLclampf;
    pub type ClampI  = ffi::GLclampx;
    pub type Fixed   = ffi::GLfixed;
    pub type IntPtr  = ffi::GLintptr;
    pub type Short   = ffi::GLshort;
    pub type UByte   = ffi::GLubyte;
    pub type UShort  = ffi::GLushort;
    pub type Void    = ffi::GLvoid;
}
#[allow(missing_docs)] pub mod consts {
    use libc::{c_uint, c_uchar};
    /* BeginMode */
    pub const POINTS:         c_uint = 0x0000 as c_uint;
    pub const LINES:          c_uint = 0x0001 as c_uint;
    pub const LINE_LOOP:      c_uint = 0x0002 as c_uint;
    pub const LINE_STRIP:     c_uint = 0x0003 as c_uint;
    pub const TRIANGLES:      c_uint = 0x0004 as c_uint;
    pub const TRIANGLE_STRIP: c_uint = 0x0005 as c_uint;
    pub const TRIANGLE_FAN:   c_uint = 0x0006 as c_uint;

    pub const DEPTH_BUFFER_BIT:   c_uint = 0x00000100 as c_uint;
    pub const STENCIL_BUFFER_BIT: c_uint = 0x00000400 as c_uint;
    pub const COLOR_BUFFER_BIT:   c_uint = 0x00004000 as c_uint;

    /* BlendingFactorDest */
    pub const ZERO:                     c_uint = 0      as c_uint;
    pub const ONE:                      c_uint = 1      as c_uint;
    pub const SRC_COLOR:                c_uint = 0x0300 as c_uint;
    pub const ONE_MINUS_SRC_COLOR:      c_uint = 0x0301 as c_uint;
    pub const SRC_ALPHA:                c_uint = 0x0302 as c_uint;
    pub const ONE_MINUS_SRC_ALPHA:      c_uint = 0x0303 as c_uint;
    pub const DST_ALPHA:                c_uint = 0x0304 as c_uint;
    pub const ONE_MINUS_DST_ALPHA:      c_uint = 0x0305 as c_uint;

    /* BlendingFactorSrc */
    pub const DST_COLOR:                c_uint = 0x0306 as c_uint;
    pub const ONE_MINUS_DST_COLOR:      c_uint = 0x0307 as c_uint;
    pub const SRC_ALPHA_SATURATE:       c_uint = 0x0308 as c_uint;

    /* Boolean */
    pub const TRUE:                     c_uchar = 1 as c_uchar;
    pub const FALSE:                    c_uchar = 0 as c_uchar;

    /* BlendEquationSeparate */
    pub const FUNC_ADD:                 c_uint = 0x8006 as c_uint;
    pub const BLEND_EQUATION:           c_uint = 0x8009 as c_uint;
    pub const BLEND_EQUATION_RGB:       c_uint = 0x8009 as c_uint;
    pub const BLEND_EQUATION_ALPHA:     c_uint = 0x883D as c_uint;

    /* BlendSubtract */
    pub const FUNC_SUBTRACT:            c_uint = 0x800A as c_uint;
    pub const FUNC_REVERSE_SUBTRACT:    c_uint = 0x800B as c_uint;

    /* Separate Blend Functions */
    pub const BLEND_DST_RGB:            c_uint = 0x80C8 as c_uint;
    pub const BLEND_SRC_RGB:            c_uint = 0x80C9 as c_uint;
    pub const BLEND_DST_ALPHA:          c_uint = 0x80CA as c_uint;
    pub const BLEND_SRC_ALPHA:          c_uint = 0x80CB as c_uint;
    pub const CONSTANT_COLOR:           c_uint = 0x8001 as c_uint;
    pub const ONE_MINUS_CONSTANT_COLOR: c_uint = 0x8002 as c_uint;
    pub const CONSTANT_ALPHA:           c_uint = 0x8003 as c_uint;
    pub const ONE_MINUS_CONSTANT_ALPHA: c_uint = 0x8004 as c_uint;
    pub const BLEND_COLOR:              c_uint = 0x8005 as c_uint;

    /* Errors. */
    pub const NO_ERROR: c_uint = 0 as c_uint;
    pub const INVALID_ENUM: c_uint = 0x0500 as c_uint;
    pub const INVALID_VALUE: c_uint = 0x0501 as c_uint;
    pub const INVALID_OPERATION: c_uint = 0x0502 as c_uint;
    pub const STACK_OVERFLOW: c_uint = 0x0503 as c_uint;
    pub const STACK_UNDERFLOW: c_uint = 0x0504 as c_uint;
    pub const OUT_OF_MEMORY: c_uint = 0x0505 as c_uint;
    pub const INVALID_FRAMEBUFFER_OPERATION: c_uint = 0x0506 as c_uint;

    /* DataType */
    pub const BYTE:           c_uint = 0x1400 as c_uint;
    pub const UNSIGNED_BYTE:  c_uint = 0x1401 as c_uint;
    pub const SHORT:          c_uint = 0x1402 as c_uint;
    pub const UNSIGNED_SHORT: c_uint = 0x1403 as c_uint;
    pub const INT:            c_uint = 0x1404 as c_uint;
    pub const UNSIGNED_INT:   c_uint = 0x1405 as c_uint;
    pub const FLOAT:          c_uint = 0x1406 as c_uint;
    pub const FIXED:          c_uint = 0x140C as c_uint;

    /* EnableCap */
    pub const TEXTURE_2D:               c_uint = 0x0DE1 as c_uint;
    pub const CULL_FACE:                c_uint = 0x0B44 as c_uint;
    pub const BLEND:                    c_uint = 0x0BE2 as c_uint;
    pub const DITHER:                   c_uint = 0x0BD0 as c_uint;
    pub const STENCIL_TEST:             c_uint = 0x0B90 as c_uint;
    pub const DEPTH_TEST:               c_uint = 0x0B71 as c_uint;
    pub const SCISSOR_TEST:             c_uint = 0x0C11 as c_uint;
    pub const POLYGON_OFFSET_FILL:      c_uint = 0x8037 as c_uint;
    pub const SAMPLE_ALPHA_TO_COVERAGE: c_uint = 0x809E as c_uint;
    pub const SAMPLE_COVERAGE:          c_uint = 0x80A0 as c_uint;

    /* Polygons */
    pub const POINT: c_uint = 0x1B00 as c_uint;
    pub const LINE: c_uint = 0x1B01 as c_uint;
    pub const FILL: c_uint = 0x1B02 as c_uint;
    pub const CW:  c_uint = 0x0900 as c_uint;
    pub const CCW: c_uint = 0x0901 as c_uint;
    pub const POLYGON_MODE: c_uint = 0x0B40 as c_uint;
    pub const POLYGON_SMOOTH: c_uint = 0x0B41 as c_uint;
    pub const POLYGON_STIPPLE: c_uint = 0x0B42 as c_uint;
    pub const EDGE_FLAG: c_uint = 0x0B43 as c_uint;

    /* GetPName */
    pub const LINE_WIDTH:                    c_uint = 0x0B21 as c_uint;
    pub const ALIASED_POINT_SIZE_RANGE:      c_uint = 0x846D as c_uint;
    pub const ALIASED_LINE_WIDTH_RANGE:      c_uint = 0x846E as c_uint;
    pub const CULL_FACE_MODE:                c_uint = 0x0B45 as c_uint;
    pub const FRONT_FACE:                    c_uint = 0x0B46 as c_uint;
    pub const DEPTH_RANGE:                   c_uint = 0x0B70 as c_uint;
    pub const DEPTH_WRITEMASK:               c_uint = 0x0B72 as c_uint;
    pub const DEPTH_CLEAR_VALUE:             c_uint = 0x0B73 as c_uint;
    pub const DEPTH_FUNC:                    c_uint = 0x0B74 as c_uint;
    pub const STENCIL_CLEAR_VALUE:           c_uint = 0x0B91 as c_uint;
    pub const STENCIL_FUNC:                  c_uint = 0x0B92 as c_uint;
    pub const STENCIL_FAIL:                  c_uint = 0x0B94 as c_uint;
    pub const STENCIL_PASS_DEPTH_FAIL:       c_uint = 0x0B95 as c_uint;
    pub const STENCIL_PASS_DEPTH_PASS:       c_uint = 0x0B96 as c_uint;
    pub const STENCIL_REF:                   c_uint = 0x0B97 as c_uint;
    pub const STENCIL_VALUE_MASK:            c_uint = 0x0B93 as c_uint;
    pub const STENCIL_WRITEMASK:             c_uint = 0x0B98 as c_uint;
    pub const STENCIL_BACK_FUNC:             c_uint = 0x8800 as c_uint;
    pub const STENCIL_BACK_FAIL:             c_uint = 0x8801 as c_uint;
    pub const STENCIL_BACK_PASS_DEPTH_FAIL:  c_uint = 0x8802 as c_uint;
    pub const STENCIL_BACK_PASS_DEPTH_PASS:  c_uint = 0x8803 as c_uint;
    pub const STENCIL_BACK_REF:              c_uint = 0x8CA3 as c_uint;
    pub const STENCIL_BACK_VALUE_MASK:       c_uint = 0x8CA4 as c_uint;
    pub const STENCIL_BACK_WRITEMASK:        c_uint = 0x8CA5 as c_uint;
    pub const VIEWPORT:                      c_uint = 0x0BA2 as c_uint;
    pub const SCISSOR_BOX:                   c_uint = 0x0C10 as c_uint;
    /*      SCISSOR_TEST */
    pub const COLOR_CLEAR_VALUE:             c_uint = 0x0C22 as c_uint;
    pub const COLOR_WRITEMASK:               c_uint = 0x0C23 as c_uint;
    pub const UNPACK_ALIGNMENT:              c_uint = 0x0CF5 as c_uint;
    pub const PACK_ALIGNMENT:                c_uint = 0x0D05 as c_uint;
    pub const MAX_TEXTURE_SIZE:              c_uint = 0x0D33 as c_uint;
    pub const MAX_VIEWPORT_DIMS:             c_uint = 0x0D3A as c_uint;
    pub const SUBPIXEL_BITS:                 c_uint = 0x0D50 as c_uint;
    pub const RED_BITS:                      c_uint = 0x0D52 as c_uint;
    pub const GREEN_BITS:                    c_uint = 0x0D53 as c_uint;
    pub const BLUE_BITS:                     c_uint = 0x0D54 as c_uint;
    pub const ALPHA_BITS:                    c_uint = 0x0D55 as c_uint;
    pub const DEPTH_BITS:                    c_uint = 0x0D56 as c_uint;
    pub const STENCIL_BITS:                  c_uint = 0x0D57 as c_uint;
    pub const POLYGON_OFFSET_UNITS:          c_uint = 0x2A00 as c_uint;
    /*      POLYGON_OFFSET_FILL */
    pub const POLYGON_OFFSET_FACTOR:         c_uint = 0x8038 as c_uint;
    pub const TEXTURE_BINDING_2D:            c_uint = 0x8069 as c_uint;
    pub const SAMPLE_BUFFERS:                c_uint = 0x80A8 as c_uint;
    pub const SAMPLES:                       c_uint = 0x80A9 as c_uint;
    pub const SAMPLE_COVERAGE_VALUE:         c_uint = 0x80AA as c_uint;
    pub const SAMPLE_COVERAGE_INVERT:        c_uint = 0x80AB as c_uint;

    /* GetTarget */
    pub const UNPACK_ROW_LENGTH: c_uint = 0x0CF2 as c_uint;

    /* PixelFormat */
    pub const DEPTH_COMPONENT: c_uint = 0x1902 as c_uint;
    pub const RED:             c_uint = 0x1903 as c_uint;
    pub const GREEN:           c_uint = 0x1904 as c_uint;
    pub const BLUE:            c_uint = 0x1905 as c_uint;
    pub const ALPHA:           c_uint = 0x1906 as c_uint;
    pub const RGB:             c_uint = 0x1907 as c_uint;
    pub const RGBA:            c_uint = 0x1908 as c_uint;

    pub const BGRA:            c_uint = 0x80e1 as c_uint;   // NB: Not OpenGL ES!
    pub const RGBA8:           c_uint = 0x8058 as c_uint;   // NB: Not OpenGL ES!

    /* Packed Pixels */
    pub const UNSIGNED_INT_8_8_8_8_REV: c_uint = 0x8367 as c_uint; // NB: Not OpenGL ES!

    /* Shaders */
    pub const FRAGMENT_SHADER:                  c_uint = 0x8B30 as c_uint;
    pub const VERTEX_SHADER:                    c_uint = 0x8B31 as c_uint;
    pub const MAX_VERTEX_ATTRIBS:               c_uint = 0x8869 as c_uint;
    pub const MAX_VERTEX_UNIFORM_VECTORS:       c_uint = 0x8DFB as c_uint;
    pub const MAX_VARYING_VECTORS:              c_uint = 0x8DFC as c_uint;
    pub const MAX_COMBINED_TEXTURE_IMAGE_UNITS: c_uint = 0x8B4D as c_uint;
    pub const MAX_VERTEX_TEXTURE_IMAGE_UNITS:   c_uint = 0x8B4C as c_uint;
    pub const MAX_TEXTURE_IMAGE_UNITS:          c_uint = 0x8872 as c_uint;
    pub const MAX_FRAGMENT_UNIFORM_VECTORS:     c_uint = 0x8DFD as c_uint;
    pub const SHADER_TYPE:                      c_uint = 0x8B4F as c_uint;
    pub const DELETE_STATUS:                    c_uint = 0x8B80 as c_uint;
    pub const LINK_STATUS:                      c_uint = 0x8B82 as c_uint;
    pub const VALIDATE_STATUS:                  c_uint = 0x8B83 as c_uint;
    pub const ATTACHED_SHADERS:                 c_uint = 0x8B85 as c_uint;
    pub const ACTIVE_UNIFORMS:                  c_uint = 0x8B86 as c_uint;
    pub const ACTIVE_UNIFORM_MAX_LENGTH:        c_uint = 0x8B87 as c_uint;
    pub const ACTIVE_ATTRIBUTES:                c_uint = 0x8B89 as c_uint;
    pub const ACTIVE_ATTRIBUTE_MAX_LENGTH:      c_uint = 0x8B8A as c_uint;
    pub const SHADING_LANGUAGE_VERSION:         c_uint = 0x8B8C as c_uint;
    pub const CURRENT_PROGRAM:                  c_uint = 0x8B8D as c_uint;

    pub const MAX_RENDER_BUFFER_SIZE:           c_uint = 0x84E8 as c_uint;
    pub const MAX_CUBE_MAP_TEXTURE_SIZE:        c_uint = 0x851C as c_uint;

    /* StencilFunction */
    pub const NEVER:    c_uint = 0x0200 as c_uint;
    pub const LESS:     c_uint = 0x0201 as c_uint;
    pub const EQUAL:    c_uint = 0x0202 as c_uint;
    pub const LEQUAL:   c_uint = 0x0203 as c_uint;
    pub const GREATER:  c_uint = 0x0204 as c_uint;
    pub const NOTEQUAL: c_uint = 0x0205 as c_uint;
    pub const GEQUAL:   c_uint = 0x0206 as c_uint;
    pub const ALWAYS:   c_uint = 0x0207 as c_uint;

    pub const VENDOR:     c_uint = 0x1F00 as c_uint;
    pub const RENDERER:   c_uint = 0x1F01 as c_uint;
    pub const VERSION:    c_uint = 0x1F02 as c_uint;
    pub const EXTENSIONS: c_uint = 0x1F03 as c_uint;

    /* Shader Source */
    pub const COMPILE_STATUS:       c_uint = 0x8B81 as c_uint;
    pub const INFO_LOG_LENGTH:      c_uint = 0x8B84 as c_uint;
    pub const SHADER_SOURCE_LENGTH: c_uint = 0x8B88 as c_uint;
    pub const SHADER_COMPILER:      c_uint = 0x8DFA as c_uint;

    /* Buffer Objects */
    pub const ARRAY_BUFFER:                 c_uint = 0x8892 as c_uint;
    pub const ELEMENT_ARRAY_BUFFER:         c_uint = 0x8893 as c_uint;
    pub const ARRAY_BUFFER_BINDING:         c_uint = 0x8894 as c_uint;
    pub const ELEMENT_ARRAY_BUFFER_BINDING: c_uint = 0x8895 as c_uint;

    pub const STREAM_DRAW:  c_uint = 0x88E0 as c_uint;
    pub const STATIC_DRAW:  c_uint = 0x88E4 as c_uint;
    pub const DYNAMIC_DRAW: c_uint = 0x88E8 as c_uint;

    /* CullFaceMode */
    pub const FRONT: c_uint =           0x0404 as c_uint;
    pub const BACK: c_uint =            0x0405 as c_uint;
    pub const FRONT_AND_BACK: c_uint =  0x0408 as c_uint;

    /* TextureMagFilter */
    pub const NEAREST: c_uint = 0x2600 as c_uint;
    pub const LINEAR:  c_uint = 0x2601 as c_uint;

    /* TextureParameterName */
    pub const TEXTURE_MAG_FILTER: c_uint = 0x2800 as c_uint;
    pub const TEXTURE_MIN_FILTER: c_uint = 0x2801 as c_uint;
    pub const TEXTURE_WRAP_S:     c_uint = 0x2802 as c_uint;
    pub const TEXTURE_WRAP_T:     c_uint = 0x2803 as c_uint;

    /* TextureUnit */
    pub const TEXTURE0:       c_uint = 0x84C0 as c_uint;
    pub const TEXTURE1:       c_uint = 0x84C1 as c_uint;
    pub const TEXTURE2:       c_uint = 0x84C2 as c_uint;
    pub const TEXTURE3:       c_uint = 0x84C3 as c_uint;
    pub const TEXTURE4:       c_uint = 0x84C4 as c_uint;
    pub const TEXTURE5:       c_uint = 0x84C5 as c_uint;
    pub const TEXTURE6:       c_uint = 0x84C6 as c_uint;
    pub const TEXTURE7:       c_uint = 0x84C7 as c_uint;
    pub const TEXTURE8:       c_uint = 0x84C8 as c_uint;
    pub const TEXTURE9:       c_uint = 0x84C9 as c_uint;
    pub const TEXTURE10:      c_uint = 0x84CA as c_uint;
    pub const TEXTURE11:      c_uint = 0x84CB as c_uint;
    pub const TEXTURE12:      c_uint = 0x84CC as c_uint;
    pub const TEXTURE13:      c_uint = 0x84CD as c_uint;
    pub const TEXTURE14:      c_uint = 0x84CE as c_uint;
    pub const TEXTURE15:      c_uint = 0x84CF as c_uint;
    pub const TEXTURE16:      c_uint = 0x84D0 as c_uint;
    pub const TEXTURE17:      c_uint = 0x84D1 as c_uint;
    pub const TEXTURE18:      c_uint = 0x84D2 as c_uint;
    pub const TEXTURE19:      c_uint = 0x84D3 as c_uint;
    pub const TEXTURE20:      c_uint = 0x84D4 as c_uint;
    pub const TEXTURE21:      c_uint = 0x84D5 as c_uint;
    pub const TEXTURE22:      c_uint = 0x84D6 as c_uint;
    pub const TEXTURE23:      c_uint = 0x84D7 as c_uint;
    pub const TEXTURE24:      c_uint = 0x84D8 as c_uint;
    pub const TEXTURE25:      c_uint = 0x84D9 as c_uint;
    pub const TEXTURE26:      c_uint = 0x84DA as c_uint;
    pub const TEXTURE27:      c_uint = 0x84DB as c_uint;
    pub const TEXTURE28:      c_uint = 0x84DC as c_uint;
    pub const TEXTURE29:      c_uint = 0x84DD as c_uint;
    pub const TEXTURE30:      c_uint = 0x84DE as c_uint;
    pub const TEXTURE31:      c_uint = 0x84DF as c_uint;
    pub const ACTIVE_TEXTURE: c_uint = 0x84E0 as c_uint;

    /* TextureWrapMode */
    pub const REPEAT:          c_uint = 0x2901 as c_uint;
    pub const CLAMP_TO_EDGE:   c_uint = 0x812F as c_uint;
    pub const MIRRORED_REPEAT: c_uint = 0x8370 as c_uint;

    pub const COLOR_ATTACHMENT0: c_uint = 0x8CE0 as c_uint;
    pub const MAX_COLOR_ATTACHMENTS: c_uint = 0x8CDF as c_uint;

    pub const FRAMEBUFFER_COMPLETE: c_uint = 0x8CD5 as c_uint;

    // Framebuffer Object
    pub const FRAMEBUFFER:  c_uint = 0x8D40 as c_uint;
    pub const RENDERBUFFER: c_uint = 0x8D41 as c_uint;
}
macro_rules! call_gl_fun(
    ($expr:expr => $fun:ident => ( $ctxt:expr, $($arg:expr),* ) ) => ({
        #[inline(never)] fn failure() -> ! {
            panic!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let mut s = "".to_string();
        let v = vec!($((stringify!($arg), format!("{:?}", $arg))),*);
        for (i, j) in v.into_iter() {
            s = format!("{:}, {:} = {:}", s, i, j);
        }
        println!("{}({:?}{})", stringify!($fun), $ctxt, s);
        let e = $expr.$fun;
        let f = if e.is_none() { failure() }
                else { e.unwrap() };
        f( $ctxt.unwrap(), $($arg),* )
    });
    ($expr:expr => $fun:ident => ( $ctxt:expr )) => ({
        #[inline(never)] fn failure() -> ! {
            panic!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let e = $expr.$fun;
        let f = if e.is_none() { failure() }
                else { e.unwrap() };
        println!("{}({:?})", stringify!($fun), $ctxt);
        f( $ctxt.unwrap() )
    })
);
pub mod traits {
    use super::super::{Resource};
    use super::Context3d;
    use super::{types, consts};
    use super::super::ppb::get_gles2;
    use super::{BufferType, BoundBuffer, BufferObject,
                VertexBuffer, IndexBuffer, TextureBuffer, FrameBuffer,
                RenderBuffer};
    use std::clone::Clone;
    use std::borrow::Cow;
    use std::borrow::ToOwned;
    use std::mem;
    use libc::c_void;

    pub trait Buffer {
        fn unwrap(&self) -> types::UInt;
        fn get_type(&self) -> BufferType;
        fn is_default(&self) -> bool {
            self.unwrap() == 0
        }
        fn to_object(&self) -> BufferObject;
    }
    impl Buffer for VertexBuffer {
        fn unwrap(&self) -> types::UInt {
            let &VertexBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { BufferType::Vertex }
        fn to_object(&self) -> BufferObject { BufferObject::Vertex((*self).clone()) }
    }
    impl Buffer for IndexBuffer {
        fn unwrap(&self) -> types::UInt {
            let &IndexBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { BufferType::Index }
        fn to_object(&self) -> BufferObject { BufferObject::Index((*self).clone()) }
    }
    impl Buffer for TextureBuffer {
        fn unwrap(&self) -> types::UInt {
            let &TextureBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { BufferType::Texture }
        fn to_object(&self) -> BufferObject { BufferObject::Texture((*self).clone()) }
    }
    impl Buffer for FrameBuffer {
        fn unwrap(&self) -> types::UInt {
            let &FrameBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { BufferType::Frame }
        fn to_object(&self) -> BufferObject { BufferObject::Frame((*self).clone()) }
    }
    impl Buffer for RenderBuffer {
        fn unwrap(&self) -> types::UInt {
            let &RenderBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { BufferType::Render }
        fn to_object(&self) -> BufferObject { BufferObject::Render((*self).clone()) }
    }
    impl Buffer for BufferObject {
        fn unwrap(&self) -> types::UInt {
            match self {
                &BufferObject::Vertex(ref inner)  => inner.unwrap(),
                &BufferObject::Index(ref inner)   => inner.unwrap(),
                &BufferObject::Texture(ref inner) => inner.unwrap(),
                &BufferObject::Frame(ref inner)   => inner.unwrap(),
                &BufferObject::Render(ref inner)  => inner.unwrap(),
            }
        }
        fn get_type(&self) -> BufferType {
            match self {
                &BufferObject::Vertex(ref inner)  => inner.get_type(),
                &BufferObject::Index(ref inner)   => inner.get_type(),
                &BufferObject::Texture(ref inner) => inner.get_type(),
                &BufferObject::Frame(ref inner)   => inner.get_type(),
                &BufferObject::Render(ref inner)  => inner.get_type(),
            }
        }
        fn to_object(&self) -> BufferObject { (*self).clone() }
    }
    impl<T: Buffer> Buffer for BoundBuffer<T> {
        fn unwrap(&self) -> types::UInt {
            let &BoundBuffer(ref inner) = self;
            inner.unwrap()
        }
        fn get_type(&self) -> BufferType {
            let &BoundBuffer(ref inner) = self;
            inner.get_type()
        }
        fn to_object(&self) -> BufferObject {
            let &BoundBuffer(ref inner) = self;
            inner.to_object()
        }
    }

    pub trait BindableTargetBuffer
        where <Self as BindableTargetBuffer>::Target: Buffer,
    {
        type Target; type TargetArg = types::Enum;
        fn bind(&self, ctxt: &mut Context3d, target: Self::TargetArg) -> Self::Target;
    }

    pub trait BindableBuffer
        where <Self as BindableBuffer>::Target: Buffer,
    {
        type Target = BoundBuffer<Self>;
        fn bind(&self, ctxt: &mut Context3d) -> Self::Target;
    }
    macro_rules! std_buffer_bind(
        ($ty:ty => $fun:ident($target:expr)) => {
            impl BindableBuffer for $ty {
                fn bind(&self, ctxt: &mut Context3d) -> BoundBuffer<$ty> {
                    call_gl_fun!(get_gles2() => $fun => (ctxt,
                                                         $target,
                                                         self.unwrap()));
                    BoundBuffer(self.to_owned())
                }
            }
        }
    );
    std_buffer_bind!(VertexBuffer => BindBuffer(consts::ARRAY_BUFFER));
    std_buffer_bind!(IndexBuffer  => BindBuffer(consts::ELEMENT_ARRAY_BUFFER));
    std_buffer_bind!(RenderBuffer => BindFramebuffer(consts::RENDERBUFFER));

    impl BindableBuffer for FrameBuffer {
        fn bind(&self, ctxt: &mut Context3d) -> BoundBuffer<FrameBuffer> {
            call_gl_fun!(get_gles2() => BindFramebuffer => (ctxt,
                                                            consts::FRAMEBUFFER,
                                                            self.unwrap()));
            ctxt.clear(super::consts::COLOR_BUFFER_BIT);
            BoundBuffer(self.to_owned())
        }
    }

    pub trait GenBuffer {
        fn gen_single(ctxt: &Context3d) -> Self;
        fn gen_multiple(ctxt: &Context3d, count: usize) -> Vec<Self>;
    }

    macro_rules! impl_gen_buffer(
        ($ty:ty, $gen_fun:ident) => {
            impl GenBuffer for $ty {
                fn gen_single(ctxt: &Context3d) -> $ty {
                    use std::mem;
                    let count = 1i32;
                    let mut buffers: [types::UInt; 1] = unsafe { mem::uninitialized() };
                    (get_gles2().$gen_fun.unwrap())(ctxt.unwrap(),
                                                    count,
                                                    buffers.as_mut_ptr());
                    From::from(buffers[0])
                }
                fn gen_multiple(ctxt: &Context3d, count: usize) -> Vec<$ty> {
                    let mut buffers: Vec<types::UInt> =
                        Vec::with_capacity(count);
                    (get_gles2().$gen_fun.unwrap())(ctxt.unwrap(),
                                                    count as i32,
                                                    buffers.as_mut_ptr());
                    buffers.map_in_place(|b| From::from(b) )
                }
            }
        }
    );
    impl_gen_buffer!(VertexBuffer,  GenBuffers);
    impl_gen_buffer!(IndexBuffer,   GenBuffers);
    impl_gen_buffer!(TextureBuffer, GenTextures);
    impl_gen_buffer!(FrameBuffer,   GenFramebuffers);
    impl_gen_buffer!(RenderBuffer,  GenRenderbuffers);

    pub trait DropBuffer {
        // This is unsafe because there is no way for us to guarantee
        // with a reasonable level of practicality that a buffer has
        // no more references or live uses at time of call.
        unsafe fn drop_buffer(self, ctxt: &Context3d);
    }
    impl DropBuffer for BufferObject {
        unsafe fn drop_buffer(self, ctxt: &Context3d) {
            match self {
                BufferObject::Vertex(inner)   => inner.drop_buffer(ctxt),
                BufferObject::Index(inner)    => inner.drop_buffer(ctxt),
                BufferObject::Texture(inner)    => inner.drop_buffer(ctxt),
                BufferObject::Frame(inner)  => inner.drop_buffer(ctxt),
                BufferObject::Render(inner) => inner.drop_buffer(ctxt),
            }
        }
    }
    macro_rules! drop_buffer(
        ($ty:ty, $del_fun:ident) => {
            impl DropBuffer for $ty {
                unsafe fn drop_buffer(self, ctxt: &Context3d) {
                    let inner = self.unwrap();
                    call_gl_fun!(get_gles2() => $del_fun => (ctxt,
                                                             1i32,
                                                             &inner as *const types::UInt))
                }
            }
            impl DropBuffer for Vec<$ty> {
                unsafe fn drop_buffer(self, ctxt: &Context3d) {
                    call_gl_fun!(get_gles2() => $del_fun => (ctxt,
                                                             self.len() as i32,
                                                             self.as_ptr() as *const types::UInt))
                }
            }
        }
    );
    drop_buffer!(VertexBuffer,  DeleteBuffers);
    drop_buffer!(IndexBuffer,   DeleteBuffers);
    drop_buffer!(TextureBuffer, DeleteTextures);
    drop_buffer!(FrameBuffer,   DeleteFramebuffers);
    drop_buffer!(RenderBuffer,  DeleteRenderbuffers);

    pub trait Usage {
        fn get_usage_enum(&self) -> types::Enum;
    }
    impl Usage for super::StaticBufferUsage {
        fn get_usage_enum(&self) -> types::Enum {
            consts::STATIC_DRAW
        }
    }
    impl Usage for super::StreamBufferUsage {
        fn get_usage_enum(&self) -> types::Enum {
            consts::STREAM_DRAW
        }
    }
    impl Usage for super::DynamicBufferUsage {
        fn get_usage_enum(&self) -> types::Enum {
            consts::DYNAMIC_DRAW
        }
    }
    /// utility trait for CompileShader below.
    pub trait GenShader {
        fn gen_single(ctxt: &Context3d) -> Self;
    }
    impl GenShader for super::VertexShader {
        fn gen_single(ctxt: &Context3d) -> super::VertexShader {
            ctxt.gen_vert_shader()
        }
    }
    impl GenShader for super::FragmentShader {
        fn gen_single(ctxt: &Context3d) -> super::FragmentShader {
            ctxt.gen_frag_shader()
        }
    }
    pub trait CompileShader: GenShader + super::ShaderUnwrap + Sized {
        fn new<'a>(ctxt: &Context3d, src: &[Cow<'a, str>])
                   -> super::CompilingShader<Self>
        {
            use libc::c_char;
            let this: Self = GenShader::gen_single(ctxt);
            let mut src_ptrs: Vec<*const c_char> = src.iter()
                .map(|s| s.as_ref().as_ptr() as *const c_char )
                .collect();
            let src_lens: Vec<types::Int> = src.iter()
                .map(|s| s.len() as types::Int)
                .collect();
            call_gl_fun!(get_gles2() => ShaderSource => (ctxt,
                                                         this.unwrap(),
                                                         src.len() as types::Int,
                                                         src_ptrs.as_mut_ptr(),
                                                         src_lens.as_ptr()));
            super::CompilingShader(this)
        }
    }
    impl CompileShader for super::VertexShader { }
    impl CompileShader for super::FragmentShader { }

    pub trait BufferElementType: Into<types::Enum> + Copy
        where [Self::Target]: ToOwned,
    {
        type Target;
        fn element_size(&self) -> usize { mem::size_of::<Self::Target>() }
    }

    pub trait GeometryMode: Into<types::Enum> + Copy { }
    pub trait VertexAttribType: BufferElementType { }
    pub trait IndexElementType: BufferElementType
        where [<Self as BufferElementType>::Target]: ToOwned,
    {
        fn ptr_offset(&self, offset: usize) -> *const c_void {
            (offset * self.element_size()) as *const c_void
        }
    }
    pub trait FrameBufferReadPixelsType: BufferElementType { }

    pub trait FrameBufferTextureAttachment: Into<types::Enum> + Copy { }
}
#[doc(hidden)]
impl From<types::UInt> for VertexBuffer {
    fn from(id: types::UInt) -> VertexBuffer {
        VertexBuffer(id)
    }
}
#[doc(hidden)]
impl From<types::UInt> for IndexBuffer {
    fn from(id: types::UInt) -> IndexBuffer {
        IndexBuffer(id)
    }
}
#[doc(hidden)]
impl From<types::UInt> for TextureBuffer {
    fn from(id: types::UInt) -> TextureBuffer {
        TextureBuffer(id)
    }
}
#[doc(hidden)]
impl From<types::UInt> for FrameBuffer {
    fn from(id: types::UInt) -> FrameBuffer {
        FrameBuffer(id)
    }
}
#[doc(hidden)]
impl From<types::UInt> for RenderBuffer {
    fn from(id: types::UInt) -> RenderBuffer {
        RenderBuffer(id)
    }
}

#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct VertexBuffer(types::UInt);
#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct IndexBuffer(types::UInt);
#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct TextureBuffer(types::UInt);
#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct FrameBuffer(types::UInt);
#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct RenderBuffer(types::UInt);

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum BufferObject {
    Vertex(VertexBuffer),
    Index(IndexBuffer),
    Texture(TextureBuffer),
    Frame(FrameBuffer),
    Render(RenderBuffer),
}

macro_rules! impl_default(
    ($ty:ty, $expr:expr) => {
        impl Default for $ty {
            fn default() -> $ty {
                $expr
            }
        }
    }
);
impl_default!(VertexBuffer,    VertexBuffer(0));
impl_default!(IndexBuffer,     IndexBuffer(0));
impl_default!(TextureBuffer,   TextureBuffer(0));
impl_default!(FrameBuffer,     FrameBuffer(0));
impl_default!(RenderBuffer,    RenderBuffer(0));

#[derive(Eq, PartialEq, Clone, Hash, Copy)]
pub enum BufferType {
    Vertex,
    Index,
    Texture,
    Frame,
    Render,
}
#[derive(Clone)]
pub enum BufferData<'a, T: 'a>
    where [T]: ToOwned,
{
    Fill(Cow<'a, [T]>),
    /// Empty(element_count)
    Empty(usize),
}
impl<'a, T> BufferData<'a, T>
    where [T]: ToOwned,
{
    pub fn byte_len(&self) -> usize {
        match self {
            &BufferData::Fill(ref buf) => buf.as_ref().len() * size_of::<T>(),
            &BufferData::Empty(len) => len * size_of::<T>(),
        }
    }
    fn as_void_ptr(&self) -> *const c_void {
        use std::ptr;
        match self {
            &BufferData::Fill(ref buf) => buf.as_ref().as_ptr() as *const c_void,
            &BufferData::Empty(_)   => ptr::null(),
        }
    }

    pub fn map_raw_type<U>(&self) -> BufferData<'a, U>
        where [U]: ToOwned, U: Sized, <[T] as ToOwned>::Owned: AsRef<[T]>,
    {
        use std::slice::from_raw_parts;
        let new_len = self.byte_len() / size_of::<U>();
        match self {
            &BufferData::Fill(Cow::Borrowed(slice)) => {
                let new = unsafe {
                    from_raw_parts(slice.as_ptr() as *const U, new_len)
                };
                BufferData::Fill(Cow::Borrowed(new))
            },
            &BufferData::Fill(Cow::Owned(ref vec)) => {
                let new = unsafe {
                    from_raw_parts(vec.as_ref().as_ptr() as *const U, new_len)
                };
                BufferData::Fill(Cow::Borrowed(new))
            },
            &BufferData::Empty(size) => BufferData::Empty(size),
        }
    }
}

trait OptPointerOffset {
    fn to_ptr_offset(&self) -> *const libc::c_void;
}
impl OptPointerOffset for Option<usize> {
    fn to_ptr_offset(&self) -> *const libc::c_void {
        use libc::c_char;
        match self {
            &Some(sb) => {
                let p: *const c_char = ptr::null();
                unsafe {
                    p.offset(sb as isize) as *const c_void
                }
            }
            &None => ptr::null(),
        }
    }
}
impl OptPointerOffset for usize {
    fn to_ptr_offset(&self) -> *const libc::c_void {
        use libc::c_char;
        let p: *const c_char = ptr::null();
        unsafe {
            p.offset(*self as isize) as *const libc::c_void
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct BoundBuffer<T>(T);
pub type BoundVertBuffer = BoundBuffer<VertexBuffer>;
pub type BoundIdxBuffer  = BoundBuffer<IndexBuffer>;

// Geo modes
#[derive(Copy, Clone)] pub struct PointsGeometryMode;
#[derive(Copy, Clone)] pub struct LineStripGeometryMode;
#[derive(Copy, Clone)] pub struct LineLoopGeometryMode;
#[derive(Copy, Clone)] pub struct LinesGeometryMode;
#[derive(Copy, Clone)] pub struct TriangleStripGeometryMode;
#[derive(Copy, Clone)] pub struct TriangleFanGeometryMode;
#[derive(Copy, Clone)] pub struct TrianglesGeometryMode;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub enum GeometryMode {
    Points,
    LineStrip,
    LineLoop,
    Lines,
    TriangleStrip,
    TriangleFan,
    Triangle,
}
impl Into<types::Enum> for GeometryMode {
    fn into(self) -> types::Enum {
        match self {
            GeometryMode::Points        => consts::POINTS,
            GeometryMode::LineStrip     => consts::LINE_STRIP,
            GeometryMode::LineLoop      => consts::LINE_LOOP,
            GeometryMode::Lines         => consts::LINES,
            GeometryMode::TriangleStrip => consts::TRIANGLE_STRIP,
            GeometryMode::TriangleFan   => consts::TRIANGLE_FAN,
            GeometryMode::Triangle      => consts::TRIANGLES,
        }
    }
}
impl traits::GeometryMode for GeometryMode { }
impl Default for GeometryMode {
    fn default() -> GeometryMode {
        GeometryMode::Triangle
    }
}
macro_rules! impl_geo_mode(
    ($ty:ty, $expr:expr) => {
        impl traits::GeometryMode for $ty { }
        impl Into<types::Enum> for $ty {
            fn into(self) -> types::Enum {
                $expr
            }
        }
    }
);
impl_geo_mode!(PointsGeometryMode,        consts::POINTS);
impl_geo_mode!(LineStripGeometryMode,     consts::LINE_STRIP);
impl_geo_mode!(LineLoopGeometryMode,      consts::LINE_LOOP);
impl_geo_mode!(LinesGeometryMode,         consts::LINES);
impl_geo_mode!(TriangleStripGeometryMode, consts::TRIANGLE_STRIP);
impl_geo_mode!(TriangleFanGeometryMode,   consts::TRIANGLE_FAN);
impl_geo_mode!(TrianglesGeometryMode,     consts::TRIANGLES);
// /Geo modes

impl BoundBuffer<VertexBuffer> {
    pub fn buffer_vertex_data<'a, T>(&self,
                                     ctxt: &Context3d,
                                     buf: BufferData<'a, u8>,
                                     usage: T)
        where T: traits::Usage,
    {
        call_gl_fun!(get_gles2() => BufferData => (ctxt,
                                                   consts::ARRAY_BUFFER,
                                                   buf.byte_len() as types::SizePtr,
                                                   buf.as_void_ptr(),
                                                   usage.get_usage_enum()))
    }
    pub fn enable_vertex_attrib_array(&self, ctxt: &Context3d,
                                      locus: AttrLocus) {
        call_gl_fun!(get_gles2() => EnableVertexAttribArray => (ctxt, locus.0));
    }
    pub fn vertex_attribute<T>(&self, ctxt: &Context3d, locus: AttrLocus,
                               count: usize, ty: T, normalize: bool,
                               stride: usize, offset: usize)
        where T: traits::VertexAttribType + Copy,
    {
        call_gl_fun!(get_gles2() => VertexAttribPointer => (ctxt,
                                                            locus.0 as types::UInt,
                                                            count   as types::Int,
                                                            ty.into(),
                                                            normalize as types::Boolean,
                                                            stride as types::Size,
                                                            offset.to_ptr_offset()))
    }

    pub fn draw_slice<T>(&self, ctxt: &Context3d, mode: T,
                         slice_start: usize, slice_len: usize)
        where T: traits::GeometryMode + Copy,
    {
        call_gl_fun!(get_gles2() => DrawArrays => (ctxt,
                                                   mode.into(),
                                                   slice_start as types::Int,
                                                   slice_len as types::Size))

    }
}

#[derive(Copy, Clone)] pub struct ByteType;
#[derive(Copy, Clone)] pub struct UByteType;
#[derive(Copy, Clone)] pub struct ShortType;
#[derive(Copy, Clone)] pub struct UShortType;
// Omitted: FIXED. Isn't recommended by Chrome.
#[derive(Copy, Clone)] pub struct FloatType;

impl traits::BufferElementType for ByteType { type Target = i8; }
impl traits::BufferElementType for UByteType { type Target = u8; }
impl traits::BufferElementType for ShortType { type Target = i16; }
impl traits::BufferElementType for UShortType { type Target = u16; }
impl traits::BufferElementType for FloatType { type Target = f32; }

macro_rules! impl_into_for_type(
    ($ty:ty, $expr:expr) => {
        impl Into<types::Enum> for $ty {
            fn into(self) -> types::Enum { $expr }
        }
    }
);
impl_into_for_type!(ByteType,   consts::BYTE);
impl_into_for_type!(UByteType,  consts::UNSIGNED_BYTE);
impl_into_for_type!(ShortType,  consts::SHORT);
impl_into_for_type!(UShortType, consts::UNSIGNED_SHORT);
impl_into_for_type!(FloatType,  consts::FLOAT);

// For use as a value type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub enum VertexAttribType {
    Byte,
    UByte,
    Short,
    UShort,
    Float,
}
impl Into<types::Enum> for VertexAttribType {
    fn into(self) -> types::Enum {
        match self {
            VertexAttribType::Byte =>   consts::BYTE,
            VertexAttribType::UByte =>  consts::UNSIGNED_BYTE,
            VertexAttribType::Short =>  consts::SHORT,
            VertexAttribType::UShort => consts::UNSIGNED_SHORT,
            VertexAttribType::Float =>  consts::FLOAT,
        }
    }
}
impl traits::BufferElementType for VertexAttribType {
    type Target = u8;
    fn element_size(&self) -> usize {
        match self {
            &VertexAttribType::Byte =>   {ByteType}.element_size(),
            &VertexAttribType::UByte =>  {UByteType}.element_size(),
            &VertexAttribType::Short =>  {ShortType}.element_size(),
            &VertexAttribType::UShort => {UShortType}.element_size(),
            &VertexAttribType::Float =>  {FloatType}.element_size(),
        }
    }
}
impl traits::VertexAttribType for VertexAttribType { }

macro_rules! impl_vert_attrib_type(
    ($ty:ty) => {
        impl traits::VertexAttribType for $ty { }
    }
);
impl_vert_attrib_type!(ByteType);
impl_vert_attrib_type!(UByteType);
impl_vert_attrib_type!(ShortType);
impl_vert_attrib_type!(UShortType);
impl_vert_attrib_type!(FloatType);

macro_rules! impl_idx_elem_type(
    ($ty:ty) => {
        impl traits::IndexElementType for $ty { }
    }
);
impl_idx_elem_type!(UByteType);
impl_idx_elem_type!(UShortType);

impl BoundBuffer<IndexBuffer> {
    pub fn buffer_index_data<'a, U>(&self, ctxt: &Context3d,
                                    buf: BufferData<'a, u8>,
                                    usage: U)
        where U: traits::Usage,
    {
        call_gl_fun!(get_gles2() => BufferData => (ctxt,
                                                   consts::ELEMENT_ARRAY_BUFFER,
                                                   buf.byte_len() as types::SizePtr,
                                                   buf.as_void_ptr(),
                                                   usage.get_usage_enum()))
    }
    pub fn draw_elements<T, U>(&self, ctxt: &Context3d, mode: T, ty: U,
                               slice_start: usize, slice_len: usize)
        where T: traits::GeometryMode, U: traits::IndexElementType,
              [<U as traits::BufferElementType>::Target]: ToOwned,
    {
        call_gl_fun!(get_gles2() => DrawElements => (ctxt,
                                                     mode.into(),
                                                     slice_len as types::Size,
                                                     ty.into(),
                                                     ty.ptr_offset(slice_start)))
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Copy)]
pub enum TexFormat {
    Alpha,
    //Luminance,
    //LuminanceAlpha,
    Rgb,
    Rgba,
}
impl TexFormat {
    fn to_ffi(&self) -> types::UInt {
        match self {
            &TexFormat::Alpha => consts::ALPHA,
            //&TexFormat::Luminance =>,
            //&TexFormat::LuminanceAlpha =>,
            &TexFormat::Rgb => consts::RGB,
            &TexFormat::Rgba => consts::RGBA,
        }
    }
    pub fn elements_len(&self) -> usize {
        match self {
            &TexFormat::Alpha => 1,
            //&TexFormat::Luminance =>,
            //&TexFormat::LuminanceAlpha =>,
            &TexFormat::Rgb => 3,
            &TexFormat::Rgba => 4,
        }
    }
}

impl traits::BindableTargetBuffer for TextureBuffer {
    type Target = BoundTexBuffer;
    fn bind(&self, ctxt: &mut Context3d, target: types::Enum) -> BoundTexBuffer {
        let bound = BoundTexBuffer {
            tex: self.clone(),
            target: target,
        };
        bound.rebind(ctxt);
        bound
    }
}
pub struct BoundTexBuffer {
    tex: TextureBuffer,
    target: types::Enum,
}
impl BoundTexBuffer {
    pub fn rebind(&self, ctxt: &mut Context3d) {
        use self::traits::Buffer;
        call_gl_fun!(get_gles2() => BindTexture => (ctxt,
                                                    self.target,
                                                    self.tex.unwrap()))
    }
    pub fn pixel_store(&self, ctxt: &Context3d,
                       pname: types::Enum, param: types::Int) {
        call_gl_fun!(get_gles2() => PixelStorei => (ctxt,
                                                    pname,
                                                    param))
    }
    pub fn image_2d(&self,
                    ctxt: &Context3d,
                    mip_lvl: types::Int,
                    internal_format: TexFormat,
                    format: TexFormat,
                    size: super::Size,
                    type_: types::UInt,
                    buf: Option<&[u8]>) {
        use std::ptr::null;
        let buf_ptr = buf.map_or(null(), |buf| buf.as_ptr() ) as *const c_void;
        call_gl_fun!(get_gles2() => TexImage2D => (ctxt,
                                                   self.target,
                                                   mip_lvl,
                                                   internal_format.to_ffi() as i32,
                                                   size.width as types::Int,
                                                   size.height as types::Int,
                                                   0i32,
                                                   format.to_ffi(),
                                                   type_,
                                                   buf_ptr))
    }
}
impl traits::Buffer for BoundTexBuffer {
    fn unwrap(&self) -> types::UInt { self.tex.unwrap() }
    fn get_type(&self) -> BufferType { BufferType::Texture }
    fn to_object(&self) -> BufferObject { BufferObject::Texture(self.tex.clone()) }
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub struct ColorAttachment(pub types::Enum);
impl Into<types::Enum> for ColorAttachment {
    fn into(self) -> types::Enum {
        consts::COLOR_ATTACHMENT0 + self.0
    }
}
impl traits::FrameBufferTextureAttachment for ColorAttachment { }

/*#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub struct DepthAttachment;
impl Into<types::Enum> for DepthAttachment {
    fn into(self) -> types::Enum {
        consts::DEPTH_ATTACHMENT
    }
}
impl FrameBufferTextureAttachment for DepthAttachment { }

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub struct StencilAttachment;
impl Into<types::Enum> for StencilAttachment {
    fn into(self) -> types::Enum {
        consts::STENCIL_ATTACHMENT
    }
}
impl FrameBufferTextureAttachment for StencilAttachment { }*/

impl BoundBuffer<FrameBuffer> {
    pub fn attach_tex2d<T>(&mut self,
                           ctxt: &Context3d,
                           attachment: T,
                           tex: TextureBuffer,
                           mip_lvl: types::Int)
        where T: traits::FrameBufferTextureAttachment,
    {
        use self::traits::Buffer;
        call_gl_fun!(get_gles2() => FramebufferTexture2D => (ctxt,
                                                             consts::FRAMEBUFFER,
                                                             attachment.into(),
                                                             consts::TEXTURE_2D,
                                                             tex.unwrap(),
                                                             mip_lvl))
    }

    pub fn check_status(&self, ctxt: &Context3d) -> types::Enum {
        call_gl_fun!(get_gles2() => CheckFramebufferStatus => (ctxt, consts::FRAMEBUFFER))
    }

    /// If there was an error, the contents of the return value are undefined.
    pub fn read_pixels<T>(&self, ctxt: &Context3d, rect: super::Rect, fmt: TexFormat,
                          ty: T) -> Vec<<T as traits::BufferElementType>::Target>
        where T: traits::FrameBufferReadPixelsType
    {
        let len = (rect.size.width * rect.size.height * fmt.elements_len()) as usize;
        let mut dest: Vec<<T as traits::BufferElementType>::Target> =
            Vec::with_capacity(len);

        call_gl_fun!(get_gles2() => ReadPixels => (ctxt, rect.point.x as types::Int,
                                                   rect.point.y as types::Int,
                                                   rect.size.width as types::Int,
                                                   rect.size.height as types::Int,
                                                   fmt.to_ffi(), ty.into(),
                                                   dest.as_mut_ptr() as *mut libc::c_void));

        unsafe { dest.set_len(len); }
        dest
    }
}

#[derive(Copy, Clone)] pub struct StaticBufferUsage;
#[derive(Copy, Clone)] pub struct StreamBufferUsage;
#[derive(Copy, Clone)] pub struct DynamicBufferUsage;

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum BlendingFun_ {
    BlendingFun(types::Enum,  // sfactor
                types::Enum), // dfactor
    BlendingFunSep(types::Enum,   // srcRGB
                   types::Enum,   // dstRGB
                   types::Enum,   // srcAlpha
                   types::Enum),  // dstAlpha
}
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum BlendingEq_ {
    BlendingEq(types::Enum), // mode
    BlendingEqSep(types::Enum,  // modeRGB
                  types::Enum), // modeAlpha
}
#[derive(Clone, Copy)]
pub struct Blending {
    pub color: Option<(types::ClampF, types::ClampF, types::ClampF, types::ClampF)>,
    pub fun:   Option<BlendingFun_>,
    pub eq:    Option<BlendingEq_>,
}
#[derive(Clone)]
pub struct CompileError<'ctxt, T: ShaderUnwrap> {
    shader: CompilingShader<T>,
    ctxt: &'ctxt Context3d,
}
impl<'ctxt, T: ShaderUnwrap> fmt::Display for CompileError<'ctxt, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.detail())
    }
}
impl<'ctxt, T: ShaderUnwrap> CompileError<'ctxt, T> {
    pub fn description(&self) -> &str {
        const DESC: &'static str = "OpenGL shader compile error";
        DESC
    }
    pub fn detail(&self) -> String {
        let info_len = self.ctxt.get_shader_param(&self.shader, consts::INFO_LOG_LENGTH);
        let mut info_buf: Vec<u8> = Vec::with_capacity(info_len as usize);
        let mut actual_len: types::Size = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetShaderInfoLog
                     => (self.ctxt,
                         self.shader.unwrap(),
                         info_buf.capacity() as types::Size,
                         &mut actual_len as *mut types::Size,
                         info_buf.as_mut_ptr() as *mut i8));
        let actual_len: usize = actual_len as usize;
        String::from_utf8_lossy(&info_buf[..actual_len])
            .to_string()
    }
}
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct CompilingShader<T>(T);
impl<T: Clone> CompilingShader<T> {
    pub unsafe fn get_shader(&self) -> T {
        let &CompilingShader(ref shader) = self;
        shader.clone()
    }
}
pub type CompilingVertexShader = CompilingShader<VertexShader>;
pub type CompilingFragmentShader = CompilingShader<FragmentShader>;
impl<T: traits::CompileShader + Clone + ShaderUnwrap + Send> CompilingShader<T> {
    pub fn results<'ctxt>(&self, ctxt: &'ctxt Context3d) -> Result<T, CompileError<'ctxt, T>> {
        let status = ctxt.get_shader_param(self, consts::COMPILE_STATUS);
        if status == consts::TRUE as i32 {
            let &CompilingShader(ref inner) = self;
            Ok(inner.clone())
        } else {
            Err(CompileError {
                shader: self.clone(),
                ctxt: ctxt,
            })
        }
    }
}

#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct VertexShader(types::UInt);
#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct FragmentShader(types::UInt);

#[allow(missing_copy_implementations)] #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct ShaderProgram(types::UInt);
pub struct BoundShaderProgram<'a>(&'a ShaderProgram);

#[derive(Eq, PartialEq, Hash)]
pub struct UnlinkedShaderProgram<'a>(ShaderProgram, &'a Context3d);
impl<'a> UnlinkedShaderProgram<'a> {
    pub unsafe fn get_program(&self) -> ShaderProgram {
        self.0.clone()
    }
}
impl<'a> Drop for UnlinkedShaderProgram<'a> {
    fn drop(&mut self) {
        self.1.mark_program_for_drop(&self.0);
    }
}

pub struct LinkError<'a, 'b>(&'a LinkingShaderProgram<'b>)
    where 'b: 'a;
impl<'a, 'b> LinkError<'a, 'b> {
    pub fn description(&self) -> &str {
        const DESC: &'static str = "OpenGL shader program linking error";
        DESC
    }
    pub fn detail(&self) -> String {
        let info_len = (self.0).1.get_program_param(&(self.0).0, consts::INFO_LOG_LENGTH);
        let mut info_buf: Vec<u8> = Vec::with_capacity(info_len as usize);
        let mut actual_len: types::Size = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetProgramInfoLog
                     => ((self.0).1,
                         unsafe { self.0.get_program().unwrap() },
                         info_buf.capacity() as types::Size,
                         &mut actual_len as *mut types::Size,
                         info_buf.as_mut_ptr() as *mut i8));
        let actual_len: usize = actual_len as usize;
        String::from_utf8_lossy(&info_buf[..actual_len])
            .to_string()
    }
}

// A program that is currently is the process of linking.
// Note there is no async access to the results (API level deficiency).
#[derive(Eq, PartialEq, Hash)]
pub struct LinkingShaderProgram<'a>(ShaderProgram, &'a Context3d);
impl<'a> LinkingShaderProgram<'a> {
    pub unsafe fn get_program(&self) -> ShaderProgram {
        self.0.clone()
    }

    pub fn results<'b>(&'b self) -> Result<ShaderProgram, LinkError<'a, 'b>> {
        let status = self.1.get_program_param(self, consts::LINK_STATUS);
        if status == consts::TRUE as i32 {
            Ok(self.inner().clone())
        } else {
            Err(LinkError(self))
        }
    }
}
impl<'a> Drop for LinkingShaderProgram<'a> {
    fn drop(&mut self) {
        self.1.mark_program_for_drop(&self.0);
    }
}
trait InnerProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram;
}
impl InnerProgram for ShaderProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram {
        self
    }
}
impl<'a> InnerProgram for UnlinkedShaderProgram<'a> {
    fn inner<'b>(&'b self) -> &'b ShaderProgram {
        let &UnlinkedShaderProgram(ref inner, _) = self;
        inner
    }
}
impl<'a> InnerProgram for LinkingShaderProgram<'a> {
    fn inner<'b>(&'b self) -> &'b ShaderProgram {
        let &LinkingShaderProgram(ref inner, _) = self;
        inner
    }
}

/// INTERNEL
pub trait ShaderUnwrap {
    fn unwrap(&self) -> types::UInt;
}
impl ShaderUnwrap for VertexShader {
    fn unwrap(&self) -> types::UInt {
        let &VertexShader(inner) = self;
        inner
    }
}
impl ShaderUnwrap for FragmentShader {
    fn unwrap(&self) -> types::UInt {
        let &FragmentShader(inner) = self;
        inner
    }
}
impl<T: ShaderUnwrap> ShaderUnwrap for CompilingShader<T> {
    fn unwrap(&self) -> types::UInt {
        let &CompilingShader(ref inner) = self;
        inner.unwrap()
    }
}
impl<'a> ShaderUnwrap for &'a VertexShader {
    fn unwrap(&self) -> types::UInt {
        let & &VertexShader(inner) = self;
        inner
    }
}
impl<'a> ShaderUnwrap for &'a FragmentShader {
    fn unwrap(&self) -> types::UInt {
        let & &FragmentShader(inner) = self;
        inner
    }
}
impl<'a, T: ShaderUnwrap> ShaderUnwrap for &'a CompilingShader<T> {
    fn unwrap(&self) -> types::UInt {
        let & &CompilingShader(ref inner) = self;
        inner.unwrap()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AttrLocus(types::UInt);
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UniformLocus(types::UInt);

/// A shader program object.
impl ShaderProgram {
    fn unwrap(&self) -> types::UInt {
        let &ShaderProgram(inner) = self;
        inner
    }

    pub fn new<'a>(ctxt: &'a Context3d) -> UnlinkedShaderProgram<'a> {
        UnlinkedShaderProgram(ctxt.gen_shader_program(), ctxt)
    }

    /// TODO: force null-termination on callers?
    pub fn uniform_locus(&self, ctxt: &Context3d, name: &str) -> Option<UniformLocus> {
        let name = format!("{}\0", name);
        let locus = call_gl_fun!(get_gles2() => GetUniformLocation => (ctxt,
                                                                       self.unwrap(),
                                                                       name.as_ptr() as *const i8));
        if locus == -1 {
            None
        } else {
            Some(UniformLocus(locus as types::UInt))
        }
    }
    /// TODO: force null-termination on callers?
    pub fn attr_locus(&self, ctxt: &Context3d, name: &str) -> Option<AttrLocus> {
        let name = format!("{}\0", name);
        let locus = call_gl_fun!(get_gles2() => GetAttribLocation => (ctxt,
                                                                      self.unwrap(),
                                                                      name.as_ptr() as *const i8));
        if locus == -1 {
            None
        } else {
            Some(AttrLocus(locus as types::UInt))
        }
    }

    pub fn use_program<'a>(&'a self, ctxt: &mut Context3d) -> BoundShaderProgram<'a> {
        call_gl_fun!(get_gles2() => UseProgram => (ctxt, self.unwrap()));
        BoundShaderProgram(self)
    }

    pub fn unlink<'a>(self, ctxt: &'a Context3d) -> UnlinkedShaderProgram<'a> {
        UnlinkedShaderProgram(self, ctxt)
    }
}


pub trait Uniform {
    fn uniform(&self, ctxt: &Context3d, locus: types::Int);
}
macro_rules! impl_uniform_fun_v(
    (($($ty:ty),*) -> $ident:ident) => {$(
        impl<'a> Uniform for &'a [$ty] {
            fn uniform(&self,
                       ctxt: &Context3d,
                       locus: types::Int) {
                let ptr = self.as_ptr();
                call_gl_fun!(get_gles2() => $ident => (ctxt,
                                                       locus,
                                                       self.len() as types::Int,
                                                       ptr))
            }
        }
    )*};
    (impl $gl_name:ident { $($arg:expr),+ }) => {
        fn uniform(&self,
                   ctxt: &Context3d,
                   locus: types::Int) {
            let this = self;
            call_gl_fun!(get_gles2() => $gl_name => (ctxt,
                                                     locus,
                                                     $($arg),*))
        }
    }
);
impl_uniform_fun_v!((types::Int)   -> Uniform1iv);
impl_uniform_fun_v!((types::Float) -> Uniform1fv);

impl<'a> BoundShaderProgram<'a> {
    fn unwrap(&self) -> &'a ShaderProgram {
        let &BoundShaderProgram(inner) = self;
        inner
    }
    pub fn uniform<TP: Uniform>(&mut self,
                                ctxt: &Context3d,
                                index: Option<UniformLocus>,
                                data: TP) {
        let index = index.map(|UniformLocus(index)| index as types::Int );
        let index = index.unwrap_or(-1);
        data.uniform(ctxt, index);
    }
}
impl<'a> UnlinkedShaderProgram<'a> {
    /// TODO: should we force null-termination on the user?
    pub fn bind_attrib_locus(&mut self,
                             ctxt: &Context3d,
                             index: types::UInt,
                             name: &str) ->
        AttrLocus
    {
        let name = format!("{}\0", name);
        call_gl_fun!(get_gles2() => BindAttribLocation => (ctxt,
                                                           self.inner().unwrap(),
                                                           index,
                                                           name.as_ptr() as *const i8));
        AttrLocus(index)
    }
    pub fn attach_shader<T: traits::CompileShader + ShaderUnwrap>(&mut self,
                                                                  ctxt: &Context3d,
                                                                  shader: &T) {
        call_gl_fun!(get_gles2() => AttachShader => (ctxt,
                                                     self.inner().unwrap(),
                                                     shader.unwrap()));
    }
    pub fn link(self) -> LinkingShaderProgram<'a> {
        let ctxt = self.1;
        let inner = self.0.clone();
        call_gl_fun!(get_gles2() => LinkProgram => (ctxt,
                                                    inner.unwrap()));
        ::std::mem::forget(self);
        LinkingShaderProgram(inner, ctxt)
    }
}

#[derive(Copy, Clone)] pub struct MaxVertexAttribs;
#[derive(Copy, Clone)] pub struct MaxVertexUniformVectors;
#[derive(Copy, Clone)] pub struct MaxVaryingVectors;
#[derive(Copy, Clone)] pub struct MaxCombinedTextureImageUnits;
#[derive(Copy, Clone)] pub struct MaxVertexImageUnits;
#[derive(Copy, Clone)] pub struct MaxTextureImageUnits;
#[derive(Copy, Clone)] pub struct MaxFragmentUniformVectors;
#[derive(Copy, Clone)] pub struct MaxCubeMapTextureSize;
#[derive(Copy, Clone)] pub struct MaxRenderBufferSize;
#[derive(Copy, Clone)] pub struct MaxTextureSize;
#[derive(Copy, Clone)] pub struct MaxColorAttachments;
#[derive(Copy, Clone)] pub struct Vendor;
#[derive(Copy, Clone)] pub struct Extensions;
#[derive(Copy, Clone)] pub struct Renderer;
#[derive(Copy, Clone)] pub struct Version;
#[derive(Copy, Clone)] pub struct ShadingLanguageVersion;

/// INTERNEL
pub trait GetQueryType {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> Self;
}
impl GetQueryType for types::Boolean {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Boolean {
        let mut ret: types::Boolean = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetBooleanv => (ctxt,
                                                    pname,
                                                    &mut ret as *mut types::Boolean));
        ret
    }
}
impl GetQueryType for types::Float {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Float {
        let mut ret: types::Float = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetFloatv => (ctxt,
                                                  pname,
                                                  &mut ret as *mut types::Float));
        ret
    }
}
impl GetQueryType for types::Int {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Int {
        let mut ret: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetIntegerv => (ctxt,
                                                    pname,
                                                    &mut ret as *mut types::Int));
        ret
    }
}
impl GetQueryType for &'static str {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> &'static str {
        use std::ffi::CStr;
        use std::str::from_utf8_unchecked;
        use std::mem::transmute;
        let str_ptr = call_gl_fun!(get_gles2() => GetString => (ctxt,
                                                                pname)) as *const i8;
        if str_ptr.is_null() {
            panic!("Got null when I queried for `{}`", pstr);
        }
        let str_buf: &'static [u8] = unsafe {
            let str = CStr::from_ptr(str_ptr);
            transmute(str.to_bytes())
        };
        unsafe { from_utf8_unchecked(str_buf) }
    }
}
impl GetQueryType for Vec<&'static str> {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> Vec<&'static str> {
        use std::ffi::CStr;
        use std::str::from_utf8_unchecked;
        use std::mem::transmute;
        let str_ptr = call_gl_fun!(get_gles2() => GetString => (ctxt,
                                                                pname)) as *const i8;
        if str_ptr.is_null() {
            panic!("Got null when I queried for `{}`", pstr);
        }

        let str_buf: &[u8] = unsafe {
            let str = CStr::from_ptr(str_ptr);
            transmute(str.to_bytes())
        };
        let str: &'static str = unsafe { transmute(from_utf8_unchecked(str_buf)) };
        str.split(' ')
            .collect()
    }
}
pub trait GetQuery<Ret: GetQueryType> {
    fn pname(&self) -> types::Enum;
    fn pstr(&self) -> &'static str;
    fn get(&self, ctxt: &Context3d) -> Ret {
        GetQueryType::get(ctxt, self.pname(), self.pstr())
    }
}

macro_rules! impl_get_query_ret_type(
    ($ty:ty => $pname:expr) => {
        impl_get_query_ret_type!($ty => $pname => types::Int);
    };
    ($ty:ty => $pname:expr => $ret:ty) => {
        impl GetQuery<$ret> for $ty {
            fn pname(&self) -> types::Enum {
                $pname
            }
            fn pstr(&self) -> &'static str {
                static PSTR: &'static str = stringify!($ty);
                PSTR
            }
        }
    };
);
impl_get_query_ret_type!(MaxVertexAttribs =>             consts::MAX_VERTEX_ATTRIBS);
impl_get_query_ret_type!(MaxVertexUniformVectors =>      consts::MAX_VERTEX_UNIFORM_VECTORS);
impl_get_query_ret_type!(MaxVaryingVectors =>            consts::MAX_VARYING_VECTORS);
impl_get_query_ret_type!(MaxCombinedTextureImageUnits => consts::MAX_COMBINED_TEXTURE_IMAGE_UNITS);
impl_get_query_ret_type!(MaxVertexImageUnits =>          consts::MAX_VERTEX_TEXTURE_IMAGE_UNITS);
impl_get_query_ret_type!(MaxTextureImageUnits =>         consts::MAX_TEXTURE_IMAGE_UNITS);
impl_get_query_ret_type!(MaxFragmentUniformVectors =>    consts::MAX_FRAGMENT_UNIFORM_VECTORS);
impl_get_query_ret_type!(MaxCubeMapTextureSize =>        consts::MAX_CUBE_MAP_TEXTURE_SIZE);
impl_get_query_ret_type!(MaxRenderBufferSize =>          consts::MAX_RENDER_BUFFER_SIZE);
impl_get_query_ret_type!(MaxTextureSize =>               consts::MAX_TEXTURE_SIZE);
impl_get_query_ret_type!(MaxColorAttachments =>          consts::MAX_COLOR_ATTACHMENTS);
impl_get_query_ret_type!(Vendor                 => consts::VENDOR                   => &'static str);
impl_get_query_ret_type!(Extensions             => consts::EXTENSIONS               => Vec<&'static str>);
impl_get_query_ret_type!(Renderer               => consts::RENDERER                 => &'static str);
impl_get_query_ret_type!(Version                => consts::VERSION                  => &'static str);
impl_get_query_ret_type!(ShadingLanguageVersion => consts::SHADING_LANGUAGE_VERSION => &'static str);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Context3dAttrib {
    Width(u32),
    Height(u32),
    AlphaSize(u32),
    RedSize(u32),
    BlueSize(u32),
    GreenSize(u32),
    DepthSize(u32),
    StencilSize(u32),
    Samples(u32),
    SampleBuffers(u32),
    SwapBehaviour(u32),
}

impl Context3dAttrib {
    pub fn to_ffi(&self) -> (u32, u32) {
        use self::Context3dAttrib::*;
        match self {
            &Width(v) => (ffi::PP_GRAPHICS3DATTRIB_WIDTH, v),
            &Height(v) => (ffi::PP_GRAPHICS3DATTRIB_HEIGHT, v),
            &AlphaSize(v) => (ffi::PP_GRAPHICS3DATTRIB_ALPHA_SIZE, v),
            &RedSize(v) => (ffi::PP_GRAPHICS3DATTRIB_RED_SIZE, v),
            &BlueSize(v) => (ffi::PP_GRAPHICS3DATTRIB_BLUE_SIZE, v),
            &GreenSize(v) => (ffi::PP_GRAPHICS3DATTRIB_GREEN_SIZE, v),
            &DepthSize(v) => (ffi::PP_GRAPHICS3DATTRIB_DEPTH_SIZE, v),
            &StencilSize(v) => (ffi::PP_GRAPHICS3DATTRIB_STENCIL_SIZE, v),
            &Samples(v) => (ffi::PP_GRAPHICS3DATTRIB_SAMPLES, v),
            &SampleBuffers(v) => (ffi::PP_GRAPHICS3DATTRIB_SAMPLE_BUFFERS, v),
            &SwapBehaviour(v) => (ffi::PP_GRAPHICS3DATTRIB_SWAP_BEHAVIOR, v),
        }
    }
}

impl_resource_for!(Context3d, ResourceType::Graphics3D);

impl Context3d {
    pub fn mark_program_for_drop(&self, program: &ShaderProgram) {
        call_gl_fun!(get_gles2() => DeleteProgram => (self, program.unwrap()));
    }

    fn gen_vert_shader(&self) -> VertexShader {
        let handle = call_gl_fun!(get_gles2() => CreateShader => (self, consts::VERTEX_SHADER));
        VertexShader(handle)
    }
    fn gen_frag_shader(&self) -> FragmentShader {
        let handle = call_gl_fun!(get_gles2() => CreateShader => (self, consts::FRAGMENT_SHADER));
        FragmentShader(handle)
    }
    fn gen_shader_program(&self) -> ShaderProgram {
        let handle = call_gl_fun!(get_gles2() => CreateProgram => (self));
        ShaderProgram(handle)
    }
    fn get_shader_param<T: ShaderUnwrap>(&self, shader: &T, pname: types::Enum) -> types::Int {
        let mut param: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetShaderiv => (self,
                                                    shader.unwrap(),
                                                    pname,
                                                    &mut param as *mut types::Int));
        param
    }
    fn get_program_param<T: InnerProgram>(&self, program: &T, pname: types::Enum) -> types::Int {
        let mut param: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() => GetProgramiv => (self,
                                                     program.inner().unwrap(),
                                                     pname,
                                                     &mut param as *mut types::Int));
        param
    }

    pub fn get<TRet: GetQueryType, T: GetQuery<TRet>>(&self, enum_: T) -> TRet {
        enum_.get(self)
    }

    /// `slot` is added to `consts::TEXTURE0`.
    pub fn activate_tex_slot(&self,
                             slot: types::Enum) {
        call_gl_fun!(get_gles2() => ActiveTexture => (self,
                                                      slot + consts::TEXTURE0))
    }
    pub fn blend(&self, blending: &Blending) {
        match blending.color {
            Some((r, g, b, a)) => {
                call_gl_fun!(get_gles2() => BlendColor => (self,
                                                           r, g, b, a))
            }
            None => (),
        }
        match blending.fun {
            Some(BlendingFun_::BlendingFun(sfactor, dfactor)) => {
                call_gl_fun!(get_gles2() => BlendFunc => (self,
                                                          sfactor,
                                                          dfactor))
            }
            Some(BlendingFun_::BlendingFunSep(src_rgb, dst_rgb,
                                src_alpha, dst_alpha)) => {
                call_gl_fun!(get_gles2() => BlendFuncSeparate => (self,
                                                                  src_rgb,
                                                                  dst_rgb,
                                                                  src_alpha,
                                                                  dst_alpha))
            }
            None => (),
        }
        match blending.eq {
            Some(BlendingEq_::BlendingEq(mode)) => {
                call_gl_fun!(get_gles2() => BlendEquation => (self,
                                                              mode))
            }
            Some(BlendingEq_::BlendingEqSep(mode_rgb,
                               mode_alpha)) => {
                call_gl_fun!(get_gles2() => BlendEquationSeparate => (self,
                                                                      mode_rgb,
                                                                      mode_alpha))
            }
            None => (),
        }
    }

    pub fn viewport(&self, rect: Rect) {
        call_gl_fun!(get_gles2() => Viewport => (self, rect.point.x as types::Int,
                                                 rect.point.y as types::Int,
                                                 rect.size.width as types::Int,
                                                 rect.size.height as types::Int));
    }

    pub fn clear(&self, mask: libc::c_uint) {
        call_gl_fun!(get_gles2() => Clear => (self,
                                              mask))
    }
    // Out of memory errors will be provided on the next swap_buffers callback.
    pub fn resize_buffers(&self, width: u32, height: u32) -> super::Code {
        use libc;
        use ppb::Graphics3DIf;
        let iface = ppb::get_graphics_3d();
        iface.resize_buffers(self.unwrap(),
                             width as libc::int32_t,
                             height as libc::int32_t)
    }
    pub fn swap_buffers<F>(&self, next_frame: CallbackArgs<F, ()>) ->
        Code<()> where F: FnOnce(Code<()>),
    {
        use ppb::Graphics3DIf;
        let interface = ppb::get_graphics_3d();

        let cc = next_frame.to_ffi_callback((), Default::default());
        let r = interface.swap_buffers(self.unwrap(), cc.cc());
        cc.drop_with_code(r)
    }
}
