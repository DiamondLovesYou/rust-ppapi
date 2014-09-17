// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// Rust PPApi is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rust PPApi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Rust PPApi. If not, see <http://www.gnu.org/licenses/>.

//! A module to wrap OpenGLES 2.0 functions from the PPAPI

#![allow(missing_doc)]

use std::mem::{size_of, uninitialized};
use std::{clone, ptr};
use std::default::Default;
use std::str::MaybeOwned;
use std::c_str::CString;
use libc;
use libc::c_void;
use super::{Resource, Callback};
use super::ppb;
use super::ppb::get_gles2;
use ffi;

#[deriving(Hash, Eq, PartialEq, Show)]
pub struct Context3d(ffi::PP_Resource);

#[allow(missing_doc)] pub mod types {
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
#[allow(missing_doc)] pub mod consts {
    use libc::{c_uint, c_uchar};
    /* BeginMode */
    pub static POINTS:         c_uint = 0x0000 as c_uint;
    pub static LINES:          c_uint = 0x0001 as c_uint;
    pub static LINE_LOOP:      c_uint = 0x0002 as c_uint;
    pub static LINE_STRIP:     c_uint = 0x0003 as c_uint;
    pub static TRIANGLES:      c_uint = 0x0004 as c_uint;
    pub static TRIANGLE_STRIP: c_uint = 0x0005 as c_uint;
    pub static TRIANGLE_FAN:   c_uint = 0x0006 as c_uint;

    pub static DEPTH_BUFFER_BIT:   c_uint = 0x00000100 as c_uint;
    pub static STENCIL_BUFFER_BIT: c_uint = 0x00000400 as c_uint;
    pub static COLOR_BUFFER_BIT:   c_uint = 0x00004000 as c_uint;

    /* BlendingFactorDest */
    pub static ZERO:                     c_uint = 0      as c_uint;
    pub static ONE:                      c_uint = 1      as c_uint;
    pub static SRC_COLOR:                c_uint = 0x0300 as c_uint;
    pub static ONE_MINUS_SRC_COLOR:      c_uint = 0x0301 as c_uint;
    pub static SRC_ALPHA:                c_uint = 0x0302 as c_uint;
    pub static ONE_MINUS_SRC_ALPHA:      c_uint = 0x0303 as c_uint;
    pub static DST_ALPHA:                c_uint = 0x0304 as c_uint;
    pub static ONE_MINUS_DST_ALPHA:      c_uint = 0x0305 as c_uint;

    /* BlendingFactorSrc */
    pub static DST_COLOR:                c_uint = 0x0306 as c_uint;
    pub static ONE_MINUS_DST_COLOR:      c_uint = 0x0307 as c_uint;
    pub static SRC_ALPHA_SATURATE:       c_uint = 0x0308 as c_uint;

    /* Boolean */
    pub static TRUE:                     c_uchar = 1 as c_uchar;
    pub static FALSE:                    c_uchar = 0 as c_uchar;

    /* BlendEquationSeparate */
    pub static FUNC_ADD:                 c_uint = 0x8006 as c_uint;
    pub static BLEND_EQUATION:           c_uint = 0x8009 as c_uint;
    pub static BLEND_EQUATION_RGB:       c_uint = 0x8009 as c_uint;
    pub static BLEND_EQUATION_ALPHA:     c_uint = 0x883D as c_uint;

    /* BlendSubtract */
    pub static FUNC_SUBTRACT:            c_uint = 0x800A as c_uint;
    pub static FUNC_REVERSE_SUBTRACT:    c_uint = 0x800B as c_uint;

    /* Separate Blend Functions */
    pub static BLEND_DST_RGB:            c_uint = 0x80C8 as c_uint;
    pub static BLEND_SRC_RGB:            c_uint = 0x80C9 as c_uint;
    pub static BLEND_DST_ALPHA:          c_uint = 0x80CA as c_uint;
    pub static BLEND_SRC_ALPHA:          c_uint = 0x80CB as c_uint;
    pub static CONSTANT_COLOR:           c_uint = 0x8001 as c_uint;
    pub static ONE_MINUS_CONSTANT_COLOR: c_uint = 0x8002 as c_uint;
    pub static CONSTANT_ALPHA:           c_uint = 0x8003 as c_uint;
    pub static ONE_MINUS_CONSTANT_ALPHA: c_uint = 0x8004 as c_uint;
    pub static BLEND_COLOR:              c_uint = 0x8005 as c_uint;

    /* Errors. */
    pub static NO_ERROR: c_uint = 0 as c_uint;
    pub static INVALID_ENUM: c_uint = 0x0500 as c_uint;
    pub static INVALID_VALUE: c_uint = 0x0501 as c_uint;
    pub static INVALID_OPERATION: c_uint = 0x0502 as c_uint;
    pub static STACK_OVERFLOW: c_uint = 0x0503 as c_uint;
    pub static STACK_UNDERFLOW: c_uint = 0x0504 as c_uint;
    pub static OUT_OF_MEMORY: c_uint = 0x0505 as c_uint;
    pub static INVALID_FRAMEBUFFER_OPERATION: c_uint = 0x0506 as c_uint;

    /* DataType */
    pub static BYTE:           c_uint = 0x1400 as c_uint;
    pub static UNSIGNED_BYTE:  c_uint = 0x1401 as c_uint;
    pub static SHORT:          c_uint = 0x1402 as c_uint;
    pub static UNSIGNED_SHORT: c_uint = 0x1403 as c_uint;
    pub static INT:            c_uint = 0x1404 as c_uint;
    pub static UNSIGNED_INT:   c_uint = 0x1405 as c_uint;
    pub static FLOAT:          c_uint = 0x1406 as c_uint;
    pub static FIXED:          c_uint = 0x140C as c_uint;

    /* EnableCap */
    pub static TEXTURE_2D:               c_uint = 0x0DE1 as c_uint;
    pub static CULL_FACE:                c_uint = 0x0B44 as c_uint;
    pub static BLEND:                    c_uint = 0x0BE2 as c_uint;
    pub static DITHER:                   c_uint = 0x0BD0 as c_uint;
    pub static STENCIL_TEST:             c_uint = 0x0B90 as c_uint;
    pub static DEPTH_TEST:               c_uint = 0x0B71 as c_uint;
    pub static SCISSOR_TEST:             c_uint = 0x0C11 as c_uint;
    pub static POLYGON_OFFSET_FILL:      c_uint = 0x8037 as c_uint;
    pub static SAMPLE_ALPHA_TO_COVERAGE: c_uint = 0x809E as c_uint;
    pub static SAMPLE_COVERAGE:          c_uint = 0x80A0 as c_uint;

    /* Polygons */
    pub static POINT: c_uint = 0x1B00 as c_uint;
    pub static LINE: c_uint = 0x1B01 as c_uint;
    pub static FILL: c_uint = 0x1B02 as c_uint;
    pub static CW:  c_uint = 0x0900 as c_uint;
    pub static CCW: c_uint = 0x0901 as c_uint;
    pub static POLYGON_MODE: c_uint = 0x0B40 as c_uint;
    pub static POLYGON_SMOOTH: c_uint = 0x0B41 as c_uint;
    pub static POLYGON_STIPPLE: c_uint = 0x0B42 as c_uint;
    pub static EDGE_FLAG: c_uint = 0x0B43 as c_uint;

    /* GetPName */
    pub static LINE_WIDTH:                    c_uint = 0x0B21 as c_uint;
    pub static ALIASED_POINT_SIZE_RANGE:      c_uint = 0x846D as c_uint;
    pub static ALIASED_LINE_WIDTH_RANGE:      c_uint = 0x846E as c_uint;
    pub static CULL_FACE_MODE:                c_uint = 0x0B45 as c_uint;
    pub static FRONT_FACE:                    c_uint = 0x0B46 as c_uint;
    pub static DEPTH_RANGE:                   c_uint = 0x0B70 as c_uint;
    pub static DEPTH_WRITEMASK:               c_uint = 0x0B72 as c_uint;
    pub static DEPTH_CLEAR_VALUE:             c_uint = 0x0B73 as c_uint;
    pub static DEPTH_FUNC:                    c_uint = 0x0B74 as c_uint;
    pub static STENCIL_CLEAR_VALUE:           c_uint = 0x0B91 as c_uint;
    pub static STENCIL_FUNC:                  c_uint = 0x0B92 as c_uint;
    pub static STENCIL_FAIL:                  c_uint = 0x0B94 as c_uint;
    pub static STENCIL_PASS_DEPTH_FAIL:       c_uint = 0x0B95 as c_uint;
    pub static STENCIL_PASS_DEPTH_PASS:       c_uint = 0x0B96 as c_uint;
    pub static STENCIL_REF:                   c_uint = 0x0B97 as c_uint;
    pub static STENCIL_VALUE_MASK:            c_uint = 0x0B93 as c_uint;
    pub static STENCIL_WRITEMASK:             c_uint = 0x0B98 as c_uint;
    pub static STENCIL_BACK_FUNC:             c_uint = 0x8800 as c_uint;
    pub static STENCIL_BACK_FAIL:             c_uint = 0x8801 as c_uint;
    pub static STENCIL_BACK_PASS_DEPTH_FAIL:  c_uint = 0x8802 as c_uint;
    pub static STENCIL_BACK_PASS_DEPTH_PASS:  c_uint = 0x8803 as c_uint;
    pub static STENCIL_BACK_REF:              c_uint = 0x8CA3 as c_uint;
    pub static STENCIL_BACK_VALUE_MASK:       c_uint = 0x8CA4 as c_uint;
    pub static STENCIL_BACK_WRITEMASK:        c_uint = 0x8CA5 as c_uint;
    pub static VIEWPORT:                      c_uint = 0x0BA2 as c_uint;
    pub static SCISSOR_BOX:                   c_uint = 0x0C10 as c_uint;
    /*      SCISSOR_TEST */
    pub static COLOR_CLEAR_VALUE:             c_uint = 0x0C22 as c_uint;
    pub static COLOR_WRITEMASK:               c_uint = 0x0C23 as c_uint;
    pub static UNPACK_ALIGNMENT:              c_uint = 0x0CF5 as c_uint;
    pub static PACK_ALIGNMENT:                c_uint = 0x0D05 as c_uint;
    pub static MAX_TEXTURE_SIZE:              c_uint = 0x0D33 as c_uint;
    pub static MAX_VIEWPORT_DIMS:             c_uint = 0x0D3A as c_uint;
    pub static SUBPIXEL_BITS:                 c_uint = 0x0D50 as c_uint;
    pub static RED_BITS:                      c_uint = 0x0D52 as c_uint;
    pub static GREEN_BITS:                    c_uint = 0x0D53 as c_uint;
    pub static BLUE_BITS:                     c_uint = 0x0D54 as c_uint;
    pub static ALPHA_BITS:                    c_uint = 0x0D55 as c_uint;
    pub static DEPTH_BITS:                    c_uint = 0x0D56 as c_uint;
    pub static STENCIL_BITS:                  c_uint = 0x0D57 as c_uint;
    pub static POLYGON_OFFSET_UNITS:          c_uint = 0x2A00 as c_uint;
    /*      POLYGON_OFFSET_FILL */
    pub static POLYGON_OFFSET_FACTOR:         c_uint = 0x8038 as c_uint;
    pub static TEXTURE_BINDING_2D:            c_uint = 0x8069 as c_uint;
    pub static SAMPLE_BUFFERS:                c_uint = 0x80A8 as c_uint;
    pub static SAMPLES:                       c_uint = 0x80A9 as c_uint;
    pub static SAMPLE_COVERAGE_VALUE:         c_uint = 0x80AA as c_uint;
    pub static SAMPLE_COVERAGE_INVERT:        c_uint = 0x80AB as c_uint;

    /* GetTarget */
    pub static UNPACK_ROW_LENGTH: c_uint = 0x0CF2 as c_uint;

    /* PixelFormat */
    pub static DEPTH_COMPONENT: c_uint = 0x1902 as c_uint;
    pub static RED:             c_uint = 0x1903 as c_uint;
    pub static GREEN:           c_uint = 0x1904 as c_uint;
    pub static BLUE:            c_uint = 0x1905 as c_uint;
    pub static ALPHA:           c_uint = 0x1906 as c_uint;
    pub static RGB:             c_uint = 0x1907 as c_uint;
    pub static RGBA:            c_uint = 0x1908 as c_uint;

    pub static BGRA:            c_uint = 0x80e1 as c_uint;   // NB: Not OpenGL ES!
    pub static RGBA8:           c_uint = 0x8058 as c_uint;   // NB: Not OpenGL ES!

    /* Packed Pixels */
    pub static UNSIGNED_INT_8_8_8_8_REV: c_uint = 0x8367 as c_uint; // NB: Not OpenGL ES!

    /* Shaders */
    pub static FRAGMENT_SHADER:                  c_uint = 0x8B30 as c_uint;
    pub static VERTEX_SHADER:                    c_uint = 0x8B31 as c_uint;
    pub static MAX_VERTEX_ATTRIBS:               c_uint = 0x8869 as c_uint;
    pub static MAX_VERTEX_UNIFORM_VECTORS:       c_uint = 0x8DFB as c_uint;
    pub static MAX_VARYING_VECTORS:              c_uint = 0x8DFC as c_uint;
    pub static MAX_COMBINED_TEXTURE_IMAGE_UNITS: c_uint = 0x8B4D as c_uint;
    pub static MAX_VERTEX_TEXTURE_IMAGE_UNITS:   c_uint = 0x8B4C as c_uint;
    pub static MAX_TEXTURE_IMAGE_UNITS:          c_uint = 0x8872 as c_uint;
    pub static MAX_FRAGMENT_UNIFORM_VECTORS:     c_uint = 0x8DFD as c_uint;
    pub static SHADER_TYPE:                      c_uint = 0x8B4F as c_uint;
    pub static DELETE_STATUS:                    c_uint = 0x8B80 as c_uint;
    pub static LINK_STATUS:                      c_uint = 0x8B82 as c_uint;
    pub static VALIDATE_STATUS:                  c_uint = 0x8B83 as c_uint;
    pub static ATTACHED_SHADERS:                 c_uint = 0x8B85 as c_uint;
    pub static ACTIVE_UNIFORMS:                  c_uint = 0x8B86 as c_uint;
    pub static ACTIVE_UNIFORM_MAX_LENGTH:        c_uint = 0x8B87 as c_uint;
    pub static ACTIVE_ATTRIBUTES:                c_uint = 0x8B89 as c_uint;
    pub static ACTIVE_ATTRIBUTE_MAX_LENGTH:      c_uint = 0x8B8A as c_uint;
    pub static SHADING_LANGUAGE_VERSION:         c_uint = 0x8B8C as c_uint;
    pub static CURRENT_PROGRAM:                  c_uint = 0x8B8D as c_uint;

    pub static MAX_RENDER_BUFFER_SIZE:           c_uint = 0x84E8 as c_uint;
    pub static MAX_CUBE_MAP_TEXTURE_SIZE:        c_uint = 0x851C as c_uint;

    /* StencilFunction */
    pub static NEVER:    c_uint = 0x0200 as c_uint;
    pub static LESS:     c_uint = 0x0201 as c_uint;
    pub static EQUAL:    c_uint = 0x0202 as c_uint;
    pub static LEQUAL:   c_uint = 0x0203 as c_uint;
    pub static GREATER:  c_uint = 0x0204 as c_uint;
    pub static NOTEQUAL: c_uint = 0x0205 as c_uint;
    pub static GEQUAL:   c_uint = 0x0206 as c_uint;
    pub static ALWAYS:   c_uint = 0x0207 as c_uint;

    pub static VENDOR:     c_uint = 0x1F00 as c_uint;
    pub static RENDERER:   c_uint = 0x1F01 as c_uint;
    pub static VERSION:    c_uint = 0x1F02 as c_uint;
    pub static EXTENSIONS: c_uint = 0x1F03 as c_uint;

    /* Shader Source */
    pub static COMPILE_STATUS:       c_uint = 0x8B81 as c_uint;
    pub static INFO_LOG_LENGTH:      c_uint = 0x8B84 as c_uint;
    pub static SHADER_SOURCE_LENGTH: c_uint = 0x8B88 as c_uint;
    pub static SHADER_COMPILER:      c_uint = 0x8DFA as c_uint;

    /* Buffer Objects */
    pub static ARRAY_BUFFER:                 c_uint = 0x8892 as c_uint;
    pub static ELEMENT_ARRAY_BUFFER:         c_uint = 0x8893 as c_uint;
    pub static ARRAY_BUFFER_BINDING:         c_uint = 0x8894 as c_uint;
    pub static ELEMENT_ARRAY_BUFFER_BINDING: c_uint = 0x8895 as c_uint;

    pub static STREAM_DRAW:  c_uint = 0x88E0 as c_uint;
    pub static STATIC_DRAW:  c_uint = 0x88E4 as c_uint;
    pub static DYNAMIC_DRAW: c_uint = 0x88E8 as c_uint;

    /* CullFaceMode */
    pub static FRONT: c_uint =           0x0404 as c_uint;
    pub static BACK: c_uint =            0x0405 as c_uint;
    pub static FRONT_AND_BACK: c_uint =  0x0408 as c_uint;

    /* TextureMagFilter */
    pub static NEAREST: c_uint = 0x2600 as c_uint;
    pub static LINEAR:  c_uint = 0x2601 as c_uint;

    /* TextureParameterName */
    pub static TEXTURE_MAG_FILTER: c_uint = 0x2800 as c_uint;
    pub static TEXTURE_MIN_FILTER: c_uint = 0x2801 as c_uint;
    pub static TEXTURE_WRAP_S:     c_uint = 0x2802 as c_uint;
    pub static TEXTURE_WRAP_T:     c_uint = 0x2803 as c_uint;

    /* TextureUnit */
    pub static TEXTURE0:       c_uint = 0x84C0 as c_uint;
    pub static TEXTURE1:       c_uint = 0x84C1 as c_uint;
    pub static TEXTURE2:       c_uint = 0x84C2 as c_uint;
    pub static TEXTURE3:       c_uint = 0x84C3 as c_uint;
    pub static TEXTURE4:       c_uint = 0x84C4 as c_uint;
    pub static TEXTURE5:       c_uint = 0x84C5 as c_uint;
    pub static TEXTURE6:       c_uint = 0x84C6 as c_uint;
    pub static TEXTURE7:       c_uint = 0x84C7 as c_uint;
    pub static TEXTURE8:       c_uint = 0x84C8 as c_uint;
    pub static TEXTURE9:       c_uint = 0x84C9 as c_uint;
    pub static TEXTURE10:      c_uint = 0x84CA as c_uint;
    pub static TEXTURE11:      c_uint = 0x84CB as c_uint;
    pub static TEXTURE12:      c_uint = 0x84CC as c_uint;
    pub static TEXTURE13:      c_uint = 0x84CD as c_uint;
    pub static TEXTURE14:      c_uint = 0x84CE as c_uint;
    pub static TEXTURE15:      c_uint = 0x84CF as c_uint;
    pub static TEXTURE16:      c_uint = 0x84D0 as c_uint;
    pub static TEXTURE17:      c_uint = 0x84D1 as c_uint;
    pub static TEXTURE18:      c_uint = 0x84D2 as c_uint;
    pub static TEXTURE19:      c_uint = 0x84D3 as c_uint;
    pub static TEXTURE20:      c_uint = 0x84D4 as c_uint;
    pub static TEXTURE21:      c_uint = 0x84D5 as c_uint;
    pub static TEXTURE22:      c_uint = 0x84D6 as c_uint;
    pub static TEXTURE23:      c_uint = 0x84D7 as c_uint;
    pub static TEXTURE24:      c_uint = 0x84D8 as c_uint;
    pub static TEXTURE25:      c_uint = 0x84D9 as c_uint;
    pub static TEXTURE26:      c_uint = 0x84DA as c_uint;
    pub static TEXTURE27:      c_uint = 0x84DB as c_uint;
    pub static TEXTURE28:      c_uint = 0x84DC as c_uint;
    pub static TEXTURE29:      c_uint = 0x84DD as c_uint;
    pub static TEXTURE30:      c_uint = 0x84DE as c_uint;
    pub static TEXTURE31:      c_uint = 0x84DF as c_uint;
    pub static ACTIVE_TEXTURE: c_uint = 0x84E0 as c_uint;

    /* TextureWrapMode */
    pub static REPEAT:          c_uint = 0x2901 as c_uint;
    pub static CLAMP_TO_EDGE:   c_uint = 0x812F as c_uint;
    pub static MIRRORED_REPEAT: c_uint = 0x8370 as c_uint;

    pub static COLOR_ATTACHMENT0: c_uint = 0x8CE0 as c_uint;
    pub static MAX_COLOR_ATTACHMENTS: c_uint = 0x8CDF as c_uint;

    pub static FRAMEBUFFER_COMPLETE: c_uint = 0x8CD5 as c_uint;

    // Framebuffer Object
    pub static FRAMEBUFFER:  c_uint = 0x8D40 as c_uint;
    pub static RENDERBUFFER: c_uint = 0x8D41 as c_uint;
}
macro_rules! call_gl_fun(
    ($expr:expr->$fun:ident => ( $ctxt:expr, $($arg:expr),* ) ) => ({
        #[inline(never)] fn failure() -> ! {
            fail!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = $expr.$fun.unwrap_or_else(failure);
        f( $ctxt.unwrap(), $($arg),* )
    });
    ($expr:expr->$fun:ident => ( $ctxt:expr )) => ({
        #[inline(never)] fn failure() -> ! {
            fail!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = $expr.$fun.unwrap_or_else(failure);
        f( $ctxt.unwrap() )
    })
)
pub mod traits {
    use super::super::{Resource};
    use super::Context3d;
    use super::{types, consts};
    use super::super::ppb::get_gles2;
    use super::{BufferType, BoundBuffer, Ctor,    BufferObject,
                VertexBuffer,  VertexBufferType,  VertBufObject,
                IndexBuffer,   IndexBufferType,   IdxBufObject,
                TextureBuffer, TextureBufferType, TexBufObject,
                FrameBuffer,   FrameBufferType,   FrameBufObject,
                RenderBuffer,  RenderBufferType,  RenderBufObject};
    use std::clone::Clone;
    use std::str::{MaybeOwned, Owned, Slice};
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
        fn get_type(&self) -> BufferType { VertexBufferType }
        fn to_object(&self) -> BufferObject { VertBufObject(self.clone()) }
    }
    impl Buffer for IndexBuffer {
        fn unwrap(&self) -> types::UInt {
            let &IndexBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { IndexBufferType }
        fn to_object(&self) -> BufferObject { IdxBufObject(self.clone()) }
    }
    impl Buffer for TextureBuffer {
        fn unwrap(&self) -> types::UInt {
            let &TextureBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { TextureBufferType }
        fn to_object(&self) -> BufferObject { TexBufObject(self.clone()) }
    }
    impl Buffer for FrameBuffer {
        fn unwrap(&self) -> types::UInt {
            let &FrameBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { FrameBufferType }
        fn to_object(&self) -> BufferObject { FrameBufObject(self.clone()) }
    }
    impl Buffer for RenderBuffer {
        fn unwrap(&self) -> types::UInt {
            let &RenderBuffer(inner) = self;
            inner
        }
        fn get_type(&self) -> BufferType { RenderBufferType }
        fn to_object(&self) -> BufferObject { RenderBufObject(self.clone()) }
    }
    impl Buffer for BufferObject {
        fn unwrap(&self) -> types::UInt {
            match self {
                &VertBufObject(inner)   => inner.unwrap(),
                &IdxBufObject(inner)    => inner.unwrap(),
                &TexBufObject(inner)    => inner.unwrap(),
                &FrameBufObject(inner)  => inner.unwrap(),
                &RenderBufObject(inner) => inner.unwrap(),
            }
        }
        fn get_type(&self) -> BufferType {
            match self {
                &VertBufObject(inner)   => inner.get_type(),
                &IdxBufObject(inner)    => inner.get_type(),
                &TexBufObject(inner)    => inner.get_type(),
                &FrameBufObject(inner)  => inner.get_type(),
                &RenderBufObject(inner) => inner.get_type(),
            }
        }
        fn to_object(&self) -> BufferObject { self.clone() }
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

    pub trait BindableBuffer {
        fn bind(&self, ctxt: &mut Context3d) -> BoundBuffer<Self>;
    }
    macro_rules! std_buffer_bind(
        ($ty:ty => $fun:ident($target:expr)) => {
            impl BindableBuffer for $ty {
                fn bind(&self, ctxt: &mut Context3d) -> BoundBuffer<$ty> {
                    call_gl_fun!(get_gles2()->$fun => (ctxt,
                                                       $target,
                                                       self.unwrap()));
                    BoundBuffer(self.clone())
                }
            }
        }
    )
    std_buffer_bind!(VertexBuffer => BindBuffer(consts::ARRAY_BUFFER))
    std_buffer_bind!(IndexBuffer  => BindBuffer(consts::ELEMENT_ARRAY_BUFFER))
    std_buffer_bind!(RenderBuffer => BindFramebuffer(consts::RENDERBUFFER))

    impl BindableBuffer for FrameBuffer {
        fn bind(&self, ctxt: &mut Context3d) -> BoundBuffer<FrameBuffer> {
            call_gl_fun!(get_gles2()->BindFramebuffer => (ctxt,
                                                          consts::FRAMEBUFFER,
                                                          self.unwrap()));
            ctxt.clear(super::consts::COLOR_BUFFER_BIT);
            BoundBuffer(self.clone())
        }
    }

    pub trait GenBuffer {
        fn gen_single(ctxt: &Context3d) -> Self;
        fn gen_multiple(ctxt: &Context3d, count: uint) -> Vec<Self>;
    }

    macro_rules! impl_gen_buffer(
        ($ty:ty, $gen_fun:ident) => {
            impl GenBuffer for $ty {
                fn gen_single(ctxt: &Context3d) -> $ty {
                    let mut v: Vec<$ty> = GenBuffer::gen_multiple(ctxt, 1);
                    v.shift().unwrap()
                }
                fn gen_multiple(ctxt: &Context3d, count: uint) -> Vec<$ty> {
                    use std::intrinsics::uninit;
                    let mut buffers: Vec<types::UInt> =
                        Vec::from_elem(count, unsafe { uninit() });
                    (get_gles2().$gen_fun.unwrap())(ctxt.unwrap(),
                                                    count as i32,
                                                    buffers.as_mut_ptr());
                    buffers
                        .move_iter()
                        .map(|b| Ctor::ctor(b) )
                        .collect()
                }
            }
        }
    )
    impl_gen_buffer!(VertexBuffer,  GenBuffers)
    impl_gen_buffer!(IndexBuffer,   GenBuffers)
    impl_gen_buffer!(TextureBuffer, GenTextures)
    impl_gen_buffer!(FrameBuffer,   GenFramebuffers)
    impl_gen_buffer!(RenderBuffer,  GenRenderbuffers)

    pub trait DropBuffer {
        // This is unsafe because there is no way for us to guarantee
        // with a reasonable level of practicality that a buffer has
        // no more references or live uses at time of call.
        unsafe fn drop_buffer(self, ctxt: &Context3d);
    }
    impl DropBuffer for BufferObject {
        unsafe fn drop_buffer(self, ctxt: &Context3d) {
            match self {
                VertBufObject(inner)   => inner.drop_buffer(ctxt),
                IdxBufObject(inner)    => inner.drop_buffer(ctxt),
                TexBufObject(inner)    => inner.drop_buffer(ctxt),
                FrameBufObject(inner)  => inner.drop_buffer(ctxt),
                RenderBufObject(inner) => inner.drop_buffer(ctxt),
            }
        }
    }
    macro_rules! drop_buffer(
        ($ty:ty, $del_fun:ident) => {
            impl DropBuffer for $ty {
                unsafe fn drop_buffer(self, ctxt: &Context3d) {
                    let inner = self.unwrap();
                    call_gl_fun!(get_gles2()->$del_fun => (ctxt,
                                                           1i32,
                                                           &inner as *const types::UInt))
                }
            }
            impl DropBuffer for Vec<$ty> {
                unsafe fn drop_buffer(self, ctxt: &Context3d) {
                    call_gl_fun!(get_gles2()->$del_fun => (ctxt,
                                                           self.len() as i32,
                                                           self.as_ptr() as *const types::UInt))
                }
            }
        }
    )
    drop_buffer!(VertexBuffer,  DeleteBuffers)
    drop_buffer!(IndexBuffer,   DeleteBuffers)
    drop_buffer!(TextureBuffer, DeleteTextures)
    drop_buffer!(FrameBuffer,   DeleteFramebuffers)
    drop_buffer!(RenderBuffer,  DeleteRenderbuffers)

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
    // utility trait for CompileShader below.
    trait GenShader {
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
    pub trait CompileShader: GenShader + super::ShaderUnwrap {
        fn new(ctxt: &Context3d, src: &Vec<MaybeOwned>) -> super::CompilingShader<Self> {
            use libc::c_char;
            let this: Self = GenShader::gen_single(ctxt);
            let mut src_ptrs: Vec<*const c_char> = src.iter()
                .map(|s| {
                    (match s {
                        &Owned(ref s) => s.as_slice().as_ptr(),
                        &Slice(s) => s.as_ptr(),
                    }) as *const c_char
                })
                .collect();
            let src_lens: Vec<types::Int> = src.iter()
                .map(|s| s.len() as types::Int)
                .collect();
            call_gl_fun!(get_gles2() -> ShaderSource => (ctxt,
                                                         this.unwrap(),
                                                         src.len() as types::Int,
                                                         src_ptrs.as_mut_ptr(),
                                                         src_lens.as_ptr()));
            super::CompilingShader(this)
        }
    }
    impl CompileShader for super::VertexShader { }
    impl CompileShader for super::FragmentShader { }

    pub trait GeometryMode {
        fn get_geo_mode_enum(&self) -> types::Enum;
    }
    pub trait VertexAttribType {
        fn get_vertex_attrib_type_enum(&self) -> types::Enum;
    }
    pub trait IndexElementType {
        fn get_index_element_type_enum(&self) -> types::Enum;
        fn ptr_offset(&self, offset: uint) -> *const c_void;
    }
}
trait Ctor {
    fn ctor(id: types::UInt) -> Self;
}
impl Ctor for VertexBuffer {
    fn ctor(id: types::UInt) -> VertexBuffer {
        VertexBuffer(id)
    }
}
impl Ctor for IndexBuffer {
    fn ctor(id: types::UInt) -> IndexBuffer {
        IndexBuffer(id)
    }
}
impl Ctor for TextureBuffer {
    fn ctor(id: types::UInt) -> TextureBuffer {
        TextureBuffer(id)
    }
}
impl Ctor for FrameBuffer {
    fn ctor(id: types::UInt) -> FrameBuffer {
        FrameBuffer(id)
    }
}
impl Ctor for RenderBuffer {
    fn ctor(id: types::UInt) -> RenderBuffer {
        RenderBuffer(id)
    }
}
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct VertexBuffer(types::UInt);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct IndexBuffer(types::UInt);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct TextureBuffer(types::UInt);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct FrameBuffer(types::UInt);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct RenderBuffer(types::UInt);

#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum BufferObject {
    VertBufObject(VertexBuffer),
    IdxBufObject(IndexBuffer),
    TexBufObject(TextureBuffer),
    FrameBufObject(FrameBuffer),
    RenderBufObject(RenderBuffer),
}

macro_rules! impl_default(
    ($ty:ty -> $expr:expr) => {
        impl Default for $ty {
            fn default() -> $ty {
                $expr
            }
        }
    }
)
impl_default!(VertexBuffer  -> VertexBuffer(0))
impl_default!(IndexBuffer   -> IndexBuffer(0))
impl_default!(TextureBuffer -> TextureBuffer(0))
impl_default!(FrameBuffer   -> FrameBuffer(0))
impl_default!(RenderBuffer  -> RenderBuffer(0))

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum BufferType {
    VertexBufferType,
    IndexBufferType,
    TextureBufferType,
    FrameBufferType,
    RenderBufferType,
}
pub enum BufferOption<'a, T: 'a> {
    BufferSome(&'a Vec<T>),
    BufferNone(uint),
}
impl<'a, T> clone::Clone for BufferOption<'a, T> {
    fn clone(&self) -> BufferOption<'a, T> {
        match self {
            &BufferSome(vec) => BufferSome(vec),
            &BufferNone(count) => BufferNone(count),
        }
    }
}
impl<'a, T> BufferOption<'a, T> {
    fn byte_len(&self) -> types::SizePtr {
        (match self {
            &BufferSome(buf) => buf.len() * size_of::<T>(),
            &BufferNone(len) => len * size_of::<T>(),
        }) as types::SizePtr
    }
    fn as_void_ptr(&self) -> *const c_void {
        use std::ptr;
        match self {
            &BufferSome(buf) => buf.as_ptr() as *const c_void,
            &BufferNone(_)   => ptr::null(),
        }
    }
}

trait OptPointerOffset {
    fn to_ptr_offset(&self) -> *const libc::c_void;
}
impl OptPointerOffset for Option<uint> {
    fn to_ptr_offset(&self) -> *const libc::c_void {
        use libc::c_char;
        match self {
            &Some(sb) => {
                let p: *const c_char = ptr::null();
                unsafe {
                    p.offset(sb as int) as *const c_void
                }
            }
            &None => ptr::null(),
        }
    }
}

#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct BoundBuffer<T>(T);
pub type BoundVertBuffer = BoundBuffer<VertexBuffer>;
pub type BoundIdxBuffer  = BoundBuffer<IndexBuffer>;

pub struct PointsGeometryMode;
pub struct LineStripGeometryMode;
pub struct LineLoopGeometryMode;
pub struct LinesGeometryMode;
pub struct TriangleStripGeometryMode;
pub struct TriangleFanGeometryMode;
pub struct TrianglesGeometryMode;

#[deriving(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Encodable, Decodable)]
pub enum GeometryMode {
    PointsGeoMode,
    LineStripGeoMode,
    LineLoopGeoMode,
    LinesGeoMode,
    TriangleStripGeoMode,
    TriangleFanGeoMode,
    TriangleGeoMode,
}
impl traits::GeometryMode for GeometryMode {
    fn get_geo_mode_enum(&self) -> types::Enum {
        match self {
            &PointsGeoMode        => consts::POINTS,
            &LineStripGeoMode     => consts::LINE_STRIP,
            &LineLoopGeoMode      => consts::LINE_LOOP,
            &LinesGeoMode         => consts::LINES,
            &TriangleStripGeoMode => consts::TRIANGLE_STRIP,
            &TriangleFanGeoMode   => consts::TRIANGLE_FAN,
            &TriangleGeoMode      => consts::TRIANGLES,
        }
    }
}
impl Default for GeometryMode {
    fn default() -> GeometryMode {
        TriangleGeoMode
    }
}
macro_rules! impl_geo_mode(
    ($ty:ty $expr:expr) => {
        impl traits::GeometryMode for $ty {
            fn get_geo_mode_enum(&self) -> types::Enum {
                $expr
            }
        }
    }
)
impl_geo_mode!(PointsGeometryMode consts::POINTS)
impl_geo_mode!(LineStripGeometryMode consts::LINE_STRIP)
impl_geo_mode!(LineLoopGeometryMode consts::LINE_LOOP)
impl_geo_mode!(LinesGeometryMode consts::LINES)
impl_geo_mode!(TriangleStripGeometryMode consts::TRIANGLE_STRIP)
impl_geo_mode!(TriangleFanGeometryMode consts::TRIANGLE_FAN)
impl_geo_mode!(TrianglesGeometryMode consts::TRIANGLES)

impl BoundBuffer<VertexBuffer> {
    pub fn buffer_vertex_data<'a, TUsage: traits::Usage>(&self,
                                                         ctxt: &mut Context3d,
                                                         buf: BufferOption<'a, u8>,
                                                         usage: TUsage) {
        call_gl_fun!(get_gles2() -> BufferData => (ctxt,
                                                   consts::ARRAY_BUFFER,
                                                   buf.byte_len(),
                                                   buf.as_void_ptr(),
                                                   usage.get_usage_enum()))
    }
    pub fn vertex_attribute<T: traits::VertexAttribType>(&self,
                                                         ctxt: &Context3d,
                                                         index: uint,
                                                         size: uint,
                                                         ty: T,
                                                         normalized: bool,
                                                         stride: uint,
                                                         offset: Option<uint>) {
        call_gl_fun!(get_gles2() -> VertexAttribPointer => (ctxt,
                                                            index as types::UInt,
                                                            size  as types::Int,
                                                            ty.get_vertex_attrib_type_enum(),
                                                            normalized as types::Boolean,
                                                            stride as types::Size,
                                                            offset.to_ptr_offset()))
    }

    pub fn draw_slice<T: traits::GeometryMode>(&self,
                                               ctxt: &Context3d,
                                               mode: T,
                                               slice_start: uint,
                                               slice_len: uint) {
        call_gl_fun!(get_gles2() -> DrawArrays => (ctxt,
                                                   mode.get_geo_mode_enum(),
                                                   slice_start as types::Int,
                                                   slice_len as types::Size))

    }
}
pub struct ByteType;
pub struct UByteType;
pub struct ShortType;
pub struct UShortType;
// Omitted: FIXED. Isn't recommended by Chrome.
pub struct FloatType;

// For use as a value type.
#[deriving(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VertexAttribType {
    ByteAttribType,
    UByteAttribType,
    ShortAttribType,
    UShortAttribType,
    FloatAttribType,
}
impl traits::VertexAttribType for VertexAttribType {
    fn get_vertex_attrib_type_enum(&self) -> types::Enum {
        match self {
            &ByteAttribType =>   consts::BYTE,
            &UByteAttribType =>  consts::UNSIGNED_BYTE,
            &ShortAttribType =>  consts::SHORT,
            &UShortAttribType => consts::UNSIGNED_SHORT,
            &FloatAttribType =>  consts::FLOAT,
        }
    }
}
macro_rules! impl_vert_attrib_type(
    ($ty:ty $expr:expr) => {
        impl traits::VertexAttribType for $ty {
            fn get_vertex_attrib_type_enum(&self) -> types::Enum {
                $expr
            }
        }
    }
)
impl_vert_attrib_type!(ByteType consts::BYTE)
impl_vert_attrib_type!(UByteType consts::UNSIGNED_BYTE)
impl_vert_attrib_type!(ShortType consts::SHORT)
impl_vert_attrib_type!(UShortType consts::UNSIGNED_SHORT)
impl_vert_attrib_type!(FloatType consts::FLOAT)

macro_rules! impl_idx_elem_type(
    ($ty:ty $expr:expr $bytes:expr) => {
        impl traits::IndexElementType for $ty {
            fn get_index_element_type_enum(&self) -> types::Enum {
                $expr
            }
            fn ptr_offset(&self, offset: uint) -> *const c_void {
                (offset * $bytes) as *const c_void
            }
        }
    }
)
impl_idx_elem_type!(UByteType consts::UNSIGNED_BYTE 1)
impl_idx_elem_type!(UShortType consts::UNSIGNED_SHORT 2)

impl BoundBuffer<IndexBuffer> {
    pub fn buffer_index_data<'a, TUsage: traits::Usage>(&self,
                                                        ctxt: &mut Context3d,
                                                        buf: BufferOption<'a, u16>,
                                                        usage: TUsage) {
        call_gl_fun!(get_gles2() -> BufferData => (ctxt,
                                                   consts::ELEMENT_ARRAY_BUFFER,
                                                   buf.byte_len(),
                                                   buf.as_void_ptr(),
                                                   usage.get_usage_enum()))
    }
    pub fn draw_elements<T: traits::GeometryMode, U: traits::IndexElementType>(&self,
                                                                               ctxt: &Context3d,
                                                                               mode: T,
                                                                               ty:   U,
                                                                               slice_start: uint,
                                                                               slice_len: uint) {
        call_gl_fun!(get_gles2() -> DrawElements => (ctxt,
                                                     mode.get_geo_mode_enum(),
                                                     slice_len as types::Size,
                                                     ty.get_index_element_type_enum(),
                                                     ty.ptr_offset(slice_start)))
    }
}

#[deriving(Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub enum TexFormat {
    AlphaTexFormat,
    //LuminanceTexFormat,
    //LuminanceAlphaTexFormat,
    RgbTexFormat,
    RgbaTexFormat,
}
impl TexFormat {
    fn to_ffi(&self) -> types::UInt {
        match self {
            &AlphaTexFormat => consts::ALPHA,
            //&LuminanceTexFormat =>,
            //&LuminanceAlphaTexFormat =>,
            &RgbTexFormat => consts::RGB,
            &RgbaTexFormat => consts::RGBA,
        }
    }
}

impl TextureBuffer {
    pub fn bind(&self, ctxt: &mut Context3d, target: types::Enum) -> BoundTexBuffer {
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
        call_gl_fun!(get_gles2() -> BindTexture => (ctxt,
                                                    self.target,
                                                    self.tex.unwrap()))
    }
    pub fn pixel_store(&self, ctxt: &Context3d,
                       pname: types::Enum, param: types::Int) {
        call_gl_fun!(get_gles2() -> PixelStorei => (ctxt,
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
                    buf: Option<&Vec<u8>>) {
        use std::ptr::null;
        call_gl_fun!(get_gles2() -> TexImage2D => (ctxt,
                                                   self.target,
                                                   mip_lvl,
                                                   internal_format.to_ffi() as i32,
                                                   size.width as types::Int,
                                                   size.height as types::Int,
                                                   0i32,
                                                   format.to_ffi(),
                                                   type_,
                                                   buf.map_or(null(),
                                                              |buf| buf.as_ptr())
                                                   as *const c_void))
    }
}
impl FrameBuffer {

}
impl BoundBuffer<FrameBuffer> {
    pub fn attach_tex2d(&mut self,
                        ctxt: &Context3d,
                        attachment: types::Enum,
                        tex: TextureBuffer,
                        mip_lvl: types::Int) {
        use self::traits::Buffer;
        call_gl_fun!(get_gles2() -> FramebufferTexture2D => (ctxt,
                                                             consts::FRAMEBUFFER,
                                                             attachment,
                                                             consts::TEXTURE_2D,
                                                             tex.unwrap(),
                                                             mip_lvl))
    }
}

pub struct StaticBufferUsage;
pub struct StreamBufferUsage;
pub struct DynamicBufferUsage;

#[deriving(Clone, PartialEq, Eq)]
pub enum BlendingFun_ {
    BlendingFun(types::Enum,  // sfactor
                types::Enum), // dfactor
    BlendingFunSep(types::Enum,   // srcRGB
                   types::Enum,   // dstRGB
                   types::Enum,   // srcAlpha
                   types::Enum),  // dstAlpha
}
#[deriving(Clone, PartialEq, Eq)]
pub enum BlendingEq_ {
    BlendingEq(types::Enum), // mode
    BlendingEqSep(types::Enum,  // modeRGB
                  types::Enum), // modeAlpha
}
#[deriving(Clone)]
pub struct Blending {
    pub color: Option<(types::ClampF, types::ClampF, types::ClampF, types::ClampF)>,
    pub fun:   Option<BlendingFun_>,
    pub eq:    Option<BlendingEq_>,
}

#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
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
    pub fn results(&self, ctxt: &Context3d) -> Result<T, proc(&Context3d): Send -> String> {
        use std::str;
        let status = ctxt.get_shader_param(self, consts::COMPILE_STATUS);
        if status == consts::TRUE as i32 {
            let &CompilingShader(ref inner) = self;
            Ok(inner.clone())
        } else {
            let this = self.clone();
            Err(proc(ctxt: &Context3d) -> String {
                let info_len = ctxt.get_shader_param(&this, consts::INFO_LOG_LENGTH);
                let mut info_buf: Vec<u8> = Vec::with_capacity(info_len as uint);
                let mut actual_len: types::Size = unsafe { uninitialized() };
                call_gl_fun!(get_gles2() -> GetShaderInfoLog
                             => (ctxt,
                                 this.unwrap(),
                                 info_buf.capacity() as types::Size,
                                 &mut actual_len as *mut types::Size,
                                 info_buf.as_mut_ptr() as *mut i8));
                str::from_utf8(info_buf.as_slice()).unwrap().to_string()
            })
        }
    }
}

#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct VertexShader(types::UInt);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct FragmentShader(types::UInt);

#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct ShaderProgram(types::UInt);
pub struct BoundShaderProgram<'a>(&'a ShaderProgram);
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct UnlinkedShaderProgram(ShaderProgram);
impl UnlinkedShaderProgram {
    pub unsafe fn get_program(&self) -> ShaderProgram {
        self.inner().clone()
    }
}

// A program that is currently is the process of linking.
// Note there is no async access to the results (API level deficiency).
#[deriving(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct LinkingShaderProgram(ShaderProgram);
impl LinkingShaderProgram {
    pub unsafe fn get_program(&self) -> ShaderProgram {
        self.inner().clone()
    }

    pub fn results(&self,
                   ctxt: &Context3d) -> Result<ShaderProgram, proc(ctxt: &Context3d): Send -> String> {
        use std::str;
        let status = ctxt.get_program_param(self, consts::LINK_STATUS);
        if status == consts::TRUE as i32 {
            Ok(self.inner().clone())
        } else {
            let this = self.clone();
            Err(proc(ctxt: &Context3d) -> String {
                let info_len = ctxt.get_program_param(&this, consts::INFO_LOG_LENGTH);
                let mut info_buf: Vec<u8> = Vec::with_capacity(info_len as uint);
                let mut actual_len: types::Size = unsafe { uninitialized() };
                call_gl_fun!(get_gles2() -> GetProgramInfoLog
                             => (ctxt,
                                 this.inner().unwrap(),
                                 info_buf.capacity() as types::Size,
                                 &mut actual_len as *mut types::Size,
                                 info_buf.as_mut_ptr() as *mut i8));
                str::from_utf8(info_buf.as_slice()).unwrap().to_string()
            })
        }
    }
}
trait InnerProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram;
}
impl InnerProgram for ShaderProgram {
    #[inline(always)] fn inner<'a>(&'a self) -> &'a ShaderProgram {
        self
    }
}
impl InnerProgram for UnlinkedShaderProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram {
        let &UnlinkedShaderProgram(ref inner) = self;
        inner
    }
}
impl<'a> InnerProgram for &'a UnlinkedShaderProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram {
        let & &UnlinkedShaderProgram(ref inner) = self;
        inner
    }
}
impl InnerProgram for LinkingShaderProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram {
        let &LinkingShaderProgram(ref inner) = self;
        inner
    }
}
impl<'a> InnerProgram for &'a LinkingShaderProgram {
    fn inner<'a>(&'a self) -> &'a ShaderProgram {
        let & &LinkingShaderProgram(ref inner) = self;
        inner
    }
}

trait ShaderUnwrap {
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

impl ShaderProgram {
    fn unwrap(&self) -> types::UInt {
        let &ShaderProgram(inner) = self;
        inner
    }

    pub fn new(ctxt: &Context3d) -> UnlinkedShaderProgram {
        UnlinkedShaderProgram(ctxt.gen_shader_program())
    }

    pub fn uniform_locale(&mut self,
                          ctxt: &Context3d,
                          name: &CString) -> Option<types::Int> {
        let locus = call_gl_fun!(get_gles2() -> GetUniformLocation => (ctxt,
                                                                       self.unwrap(),
                                                                       name.as_ptr()));
        if locus == -1 {
            None
        } else {
            Some(locus)
        }
    }

    pub fn use_program<'a>(&'a self, ctxt: &mut Context3d) -> BoundShaderProgram<'a> {
        call_gl_fun!(get_gles2() -> UseProgram => (ctxt, self.unwrap()));
        BoundShaderProgram(self)
    }

    pub fn unlink(self) -> UnlinkedShaderProgram {
        UnlinkedShaderProgram(self)
    }
}

trait UniformFun {
    fn uniform(&self, ctxt: &Context3d, locale: types::Int);
}
macro_rules! impl_uniform_fun_v(
    (($($ty:ty),*) -> $ident:ident) => {$(
        impl<'a> UniformFun for &'a [$ty] {
            fn uniform(&self,
                       ctxt: &Context3d,
                       locale: types::Int) {
                let ptr = self.as_ptr();
                call_gl_fun!(get_gles2() -> $ident => (ctxt,
                                                       locale,
                                                       self.len() as types::Int,
                                                       ptr))
            }
        }
    )*}
)
impl_uniform_fun_v!((types::Int)   -> Uniform1iv)
impl_uniform_fun_v!((types::Float) -> Uniform1fv)

impl<'a> BoundShaderProgram<'a> {
    fn unwrap(&self) -> &'a ShaderProgram {
        let &BoundShaderProgram(inner) = self;
        inner
    }
    pub fn uniform<TP: UniformFun>(&mut self,
                                   ctxt: &Context3d,
                                   index: Option<i32>,
                                   data: TP) {
        match index {
            Some(index) => {
                data.uniform(ctxt, index as types::Int);
            }
            None => {}
        }
    }
}
impl UnlinkedShaderProgram {
    pub fn bind_attrib_locale(&mut self,
                              ctxt: &Context3d,
                              index: uint,
                              name: &CString) {
        call_gl_fun!(get_gles2() -> BindAttribLocation => (ctxt,
                                                           self.inner().unwrap(),
                                                           index as types::UInt,
                                                           name.as_ptr()))
    }
    pub fn attach_shader<T: traits::CompileShader + ShaderUnwrap>(&mut self,
                                                                  ctxt: &Context3d,
                                                                  shader: &T) {
        call_gl_fun!(get_gles2() -> AttachShader => (ctxt,
                                                     self.inner().unwrap(),
                                                     shader.unwrap()));
    }
    pub fn link(self, ctxt: &Context3d) -> LinkingShaderProgram {
        let UnlinkedShaderProgram(inner) = self;
        call_gl_fun!(get_gles2() -> LinkProgram => (ctxt,
                                                    inner.unwrap()));
        LinkingShaderProgram(inner)
    }
}

pub struct MaxVertexAttribs;
pub struct MaxVertexUniformVectors;
pub struct MaxVaryingVectors;
pub struct MaxCombinedTextureImageUnits;
pub struct MaxVertexImageUnits;
pub struct MaxTextureImageUnits;
pub struct MaxFragmentUniformVectors;
pub struct MaxCubeMapTextureSize;
pub struct MaxRenderBufferSize;
pub struct MaxTextureSize;
pub struct MaxColorAttachments;
pub struct Vendor;
pub struct Extensions;
pub struct Renderer;
pub struct Version;
pub struct ShadingLanguageVersion;

trait GetQueryType {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> Self;
}
impl GetQueryType for types::Boolean {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Boolean {
        let mut ret: types::Boolean = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() -> GetBooleanv => (ctxt,
                                                    pname,
                                                    &mut ret as *mut types::Boolean));
        ret
    }
}
impl GetQueryType for types::Float {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Float {
        let mut ret: types::Float = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() -> GetFloatv => (ctxt,
                                                  pname,
                                                  &mut ret as *mut types::Float));
        ret
    }
}
impl GetQueryType for types::Int {
    fn get(ctxt: &Context3d, pname: types::Enum, _pstr: &'static str) -> types::Int {
        let mut ret: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() -> GetIntegerv => (ctxt,
                                                    pname,
                                                    &mut ret as *mut types::Int));
        ret
    }
}
impl GetQueryType for MaybeOwned<'static> {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> MaybeOwned<'static> {
        use std::c_str::CString;
        use std::str::from_utf8_lossy;
        use core::mem::transmute;
        let str_ptr = call_gl_fun!(get_gles2() -> GetString => (ctxt,
                                                                pname));
        if str_ptr.is_null() {
            fail!("Got null when I queried for `{}`", pstr);
        }

        unsafe {
            let str = CString::new(str_ptr as *const i8, false);
            from_utf8_lossy(transmute(str.as_bytes_no_nul()))
        }
    }
}
impl GetQueryType for Vec<MaybeOwned<'static>> {
    fn get(ctxt: &Context3d, pname: types::Enum, pstr: &'static str) -> Vec<MaybeOwned<'static>> {
        use std::c_str::{from_c_multistring, CString};
        use std::str::from_utf8_lossy;
        use core::mem::transmute;
        let str_ptr = call_gl_fun!(get_gles2() -> GetString => (ctxt,
                                                                pname));
        if str_ptr.is_null() {
            fail!("Got null when I queried for `{}`", pstr);
        }

        let mut exts = Vec::new();
        unsafe {
            from_c_multistring(str_ptr as *const i8, None, |cstr: &CString| {
                let buf: &'static [u8] = transmute(cstr.as_bytes_no_nul());
                exts.push(from_utf8_lossy(buf));
            });
        }
        exts
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
        impl_get_query_ret_type!($ty => $pname -> types::Int)
    };
    ($ty:ty => $pname:expr -> $ret:ty) => {
        impl GetQuery<$ret> for $ty {
            fn pname(&self) -> types::Enum {
                $pname
            }
            fn pstr(&self) -> &'static str {
                static PSTR: &'static str = stringify!($ty);
                PSTR
            }
        }
    }
)
impl_get_query_ret_type!(MaxVertexAttribs =>             consts::MAX_VERTEX_ATTRIBS)
impl_get_query_ret_type!(MaxVertexUniformVectors =>      consts::MAX_VERTEX_UNIFORM_VECTORS)
impl_get_query_ret_type!(MaxVaryingVectors =>            consts::MAX_VARYING_VECTORS)
impl_get_query_ret_type!(MaxCombinedTextureImageUnits => consts::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
impl_get_query_ret_type!(MaxVertexImageUnits =>          consts::MAX_VERTEX_TEXTURE_IMAGE_UNITS)
impl_get_query_ret_type!(MaxTextureImageUnits =>         consts::MAX_TEXTURE_IMAGE_UNITS)
impl_get_query_ret_type!(MaxFragmentUniformVectors =>    consts::MAX_FRAGMENT_UNIFORM_VECTORS)
impl_get_query_ret_type!(MaxCubeMapTextureSize =>        consts::MAX_CUBE_MAP_TEXTURE_SIZE)
impl_get_query_ret_type!(MaxRenderBufferSize =>          consts::MAX_RENDER_BUFFER_SIZE)
impl_get_query_ret_type!(MaxTextureSize =>               consts::MAX_TEXTURE_SIZE)
impl_get_query_ret_type!(MaxColorAttachments =>          consts::MAX_COLOR_ATTACHMENTS)

impl_get_query_ret_type!(Vendor                 => consts::VENDOR -> MaybeOwned<'static>)
impl_get_query_ret_type!(Extensions             => consts::EXTENSIONS -> Vec<MaybeOwned<'static>>)
impl_get_query_ret_type!(Renderer               => consts::RENDERER -> MaybeOwned<'static>)
impl_get_query_ret_type!(Version                => consts::VERSION -> MaybeOwned<'static>)
impl_get_query_ret_type!(ShadingLanguageVersion => consts::SHADING_LANGUAGE_VERSION -> MaybeOwned<'static>)

impl_resource_for!(Context3d Graphics3DRes)

impl Context3d {
    fn gen_vert_shader(&self) -> VertexShader {
        let handle = call_gl_fun!(get_gles2() -> CreateShader => (self, consts::VERTEX_SHADER));
        VertexShader(handle)
    }
    fn gen_frag_shader(&self) -> FragmentShader {
        let handle = call_gl_fun!(get_gles2() -> CreateShader => (self, consts::FRAGMENT_SHADER));
        FragmentShader(handle)
    }
    fn gen_shader_program(&self) -> ShaderProgram {
        let handle = call_gl_fun!(get_gles2() -> CreateProgram => (self));
        ShaderProgram(handle)
    }
    fn get_shader_param<T: ShaderUnwrap>(&self, shader: &T, pname: types::Enum) -> types::Int {
        let mut param: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() -> GetShaderiv => (self,
                                                    shader.unwrap(),
                                                    pname,
                                                    &mut param as *mut types::Int));
        param
    }
    fn get_program_param<T: InnerProgram>(&self, program: &T, pname: types::Enum) -> types::Int {
        let mut param: types::Int = unsafe { uninitialized() };
        call_gl_fun!(get_gles2() -> GetProgramiv => (self,
                                                     program.inner().unwrap(),
                                                     pname,
                                                     &mut param as *mut types::Int));
        param
    }

    pub fn get<TRet: GetQueryType, T: GetQuery<TRet>>(&self, enum_: T) -> TRet {
        enum_.get(self)
    }

    pub fn activate_tex_slot(&self,
                             // note this param is added to consts::TEXTURE0
                             slot: types::Enum) {
        call_gl_fun!(get_gles2() -> ActiveTexture => (self,
                                                      slot + consts::TEXTURE0))
    }
    pub fn blend(&self, blending: &Blending) {
        match blending.color {
            Some((r, g, b, a)) => {
                call_gl_fun!(get_gles2() -> BlendColor => (self,
                                                           r, g, b, a))
            }
            None => (),
        }
        match blending.fun {
            Some(BlendingFun(sfactor, dfactor)) => {
                call_gl_fun!(get_gles2() -> BlendFunc => (self,
                                                          sfactor,
                                                          dfactor))
            }
            Some(BlendingFunSep(src_rgb, dst_rgb,
                                src_alpha, dst_alpha)) => {
                call_gl_fun!(get_gles2() -> BlendFuncSeparate => (self,
                                                                  src_rgb,
                                                                  dst_rgb,
                                                                  src_alpha,
                                                                  dst_alpha))
            }
            None => (),
        }
        match blending.eq {
            Some(BlendingEq(mode)) => {
                call_gl_fun!(get_gles2() -> BlendEquation => (self,
                                                              mode))
            }
            Some(BlendingEqSep(mode_rgb,
                               mode_alpha)) => {
                call_gl_fun!(get_gles2() -> BlendEquationSeparate => (self,
                                                                      mode_rgb,
                                                                      mode_alpha))
            }
            None => (),
        }
    }
    pub fn clear(&self, mask: libc::c_uint) {
        call_gl_fun!(get_gles2() -> Clear => (self,
                                              mask))
    }
    pub fn swap_buffers<T: Callback>(&self, next_frame: T) {
        use super::{Callback, PostToSelf};
        use ppb::Graphics3DIf;
        let interface = ppb::get_graphics_3d();

        let cb = next_frame.to_ffi_callback();
        let r = interface.swap_buffers(self.unwrap(), cb);
        if !r.is_ok() {
            cb.post_to_self(r);
        }
    }
}
