use crate::gpu::gl::{Extensions, UInt};
use crate::prelude::*;
use skia_bindings as sb;
use skia_bindings::{GrGLInterface, SkRefCntBase, GrGLenum};
use std::ffi::c_void;
use std::os::raw;

const GR_GL_FRAMEBUFFER_BINDING: GrGLenum = 0x8CA6;

pub type Interface = RCHandle<GrGLInterface>;

impl NativeRefCountedBase for GrGLInterface {
    type Base = SkRefCntBase;
}

impl RCHandle<GrGLInterface> {
    pub fn new_native() -> Option<Interface> {
        Self::from_ptr(unsafe { sb::C_GrGLInterface_MakeNativeInterface() as _ })
    }

    pub fn new_load_with<F>(loadfn: F) -> Option<Interface>
        where
            F: FnMut(&str) -> *const c_void,
    {
        Self::from_ptr(unsafe {
            sb::C_GrGLInterface_MakeAssembledInterface(
                &loadfn as *const _ as *mut c_void,
                Some(gl_get_proc_fn_wrapper::<F>),
            ) as _
        })
    }
}

unsafe extern "C" fn gl_get_proc_fn_wrapper<F>(
    ctx: *mut c_void,
    name: *const raw::c_char,
) -> *const c_void
    where
        F: FnMut(&str) -> *const c_void,
{
    (*(ctx as *mut F))(std::ffi::CStr::from_ptr(name).to_str().unwrap())
}

impl RCHandle<GrGLInterface> {
    pub fn validate(&self) -> bool {
        unsafe { self.native().validate() }
    }

    pub fn extensions(&self) -> &Extensions {
        Extensions::from_native_ref(unsafe {
            &*sb::C_GrGLInterface_extensions(self.native_mut_force())
        })
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        Extensions::from_native_ref_mut(unsafe {
            &mut *sb::C_GrGLInterface_extensions(self.native_mut())
        })
    }

    pub fn has_extension(&self, extension: impl AsRef<str>) -> bool {
        self.extensions().has(extension)
    }

    fn get_integer_value(&mut self, pname: GrGLenum) -> UInt {
        unsafe { sb::C_GrGLInterface_GetIntegerv(self.native_mut(), pname) as UInt }
    }

    pub fn get_framebuffer_binding(&mut self) -> UInt {
        self.get_integer_value(GR_GL_FRAMEBUFFER_BINDING)
    }
}
