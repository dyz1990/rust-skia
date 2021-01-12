#[cfg(feature = "d3d")]
use super::d3d;
#[cfg(feature = "gl")]
use super::gl;
#[cfg(feature = "vulkan")]
use super::vk;
use crate::gpu::{
    BackendFormat, DirectContext,
    Mipmapped,
};
use crate::prelude::*;
use crate::{image, Image};
use skia_bindings as sb;
use skia_bindings::{GrContext_Base, GrDirectContext, GrRecordingContext, SkRefCntBase};
use std::{
    ops::{Deref, DerefMut},
};

pub type Context = RCHandle<GrContext_Base>;

impl NativeRefCountedBase for GrContext_Base {
    type Base = SkRefCntBase;
}

impl From<RCHandle<GrDirectContext>> for RCHandle<GrContext_Base> {
    fn from(direct_context: RCHandle<GrDirectContext>) -> Self {
        unsafe { std::mem::transmute(direct_context) }
    }
}

impl Deref for RCHandle<GrRecordingContext> {
    type Target = RCHandle<GrContext_Base>;

    fn deref(&self) -> &Self::Target {
        unsafe { transmute_ref(self) }
    }
}

impl DerefMut for RCHandle<GrRecordingContext> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute_ref_mut(self) }
    }
}

impl RCHandle<GrContext_Base> {
    #[cfg(feature = "gl")]
    pub fn new_gl(interface: impl Into<Option<gl::Interface>>) -> Option<Context> {
        DirectContext::new_gl(interface, None).map(|c| c.into())
    }

    #[cfg(feature = "vulkan")]
    pub fn new_vulkan(backend_context: &vk::BackendContext) -> Option<Context> {
        DirectContext::new_vulkan(backend_context, None).map(|c| c.into())
    }

    /// # Safety
    /// This function is unsafe because `device` and `queue` are untyped handles which need to exceed the
    /// lifetime of the context returned.
    #[cfg(feature = "metal")]
    pub unsafe fn new_metal(
        device: *mut std::ffi::c_void,
        queue: *mut std::ffi::c_void,
    ) -> Option<Context> {
        DirectContext::new_metal(device, queue, None).map(|c| c.into())
    }

    // TODO: support variant with GrContextOptions
    #[cfg(feature = "d3d")]
    pub unsafe fn new_d3d(backend_context: &d3d::BackendContext) -> Option<Context> {
        DirectContext::new_d3d(backend_context, None).map(|c| c.into())
    }

    // TODO: threadSafeProxy()


    pub fn abandon(&mut self) -> &mut Self {
        unsafe {
            // self.native_mut().abandonContext()
            sb::GrImageContext_abandonContext(self.native_mut() as *mut _ as _)
        }
        self
    }

    #[cfg(feature = "vulkan")]
    pub fn store_vk_pipeline_cache_data(&mut self) -> &mut Self {
        unsafe {
            self.native_mut().storeVkPipelineCacheData();
        }
        self
    }

    pub fn compute_image_size(
        image: impl AsRef<Image>,
        mipmapped: Mipmapped,
        use_next_pow2: impl Into<Option<bool>>,
    ) -> usize {
        unsafe {
            sb::C_GrContext_ComputeImageSize(
                image.as_ref().clone().into_ptr(),
                mipmapped,
                use_next_pow2.into().unwrap_or_default(),
            )
        }
    }

    // TODO: wrap createBackendTexture (several variants)
    //       introduced in m76, m77, and m79
    //       extended in m84 with finishedProc and finishedContext

    // TODO: wrap updateBackendTexture (several variants)
    //       introduced in m84

    pub fn compressed_backend_format(&self, compression: image::CompressionType) -> BackendFormat {
        let mut backend_format = BackendFormat::default();
        unsafe {
            sb::C_GrContext_compressedBackendFormat(
                self.native(),
                compression,
                backend_format.native_mut(),
            )
        };
        backend_format
    }

    // TODO: wrap createCompressedBackendTexture (several variants)
    //       introduced in m81
    //       extended in m84 with finishedProc and finishedContext

    // TODO: wrap updateCompressedBackendTexture (two variants)
    //       introduced in m86


}
