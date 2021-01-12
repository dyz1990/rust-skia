#[cfg(feature = "d3d")]
use super::d3d;
#[cfg(feature = "gl")]
use super::gl;
#[cfg(feature = "vulkan")]
use super::vk;
use super::*;
use crate::{image, ColorType, Data, Image};
use crate::prelude::*;
use skia_bindings as sb;
use skia_bindings::{GrRecordingContext, GrDirectContext, SkRefCntBase};
use std::{
    ops::{Deref, DerefMut},
    ptr,
    time::Duration,
};
pub type DirectContext = RCHandle<GrDirectContext>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourceCacheLimits {
    pub max_resources: usize,
    pub max_resource_bytes: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourceCacheUsage {
    pub resource_count: usize,
    pub resource_bytes: usize,
}

impl NativeRefCountedBase for GrDirectContext {
    type Base = SkRefCntBase;
}

impl Deref for RCHandle<GrDirectContext> {
    type Target = RCHandle<GrRecordingContext>;

    fn deref(&self) -> &Self::Target {
        unsafe { transmute_ref(self) }
    }
}

impl DerefMut for RCHandle<GrDirectContext> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { transmute_ref_mut(self) }
    }
}

impl RCHandle<GrDirectContext> {
    #[cfg(feature = "gl")]
    pub fn new_gl<'a>(
        interface: impl Into<Option<gl::Interface>>,
        options: impl Into<Option<&'a ContextOptions>>,
    ) -> Option<DirectContext> {
        DirectContext::from_ptr(unsafe {
            sb::C_GrDirectContext_MakeGL(
                interface.into().into_ptr_or_null(),
                options.into().native_ptr_or_null(),
            )
        })
    }


    pub fn reset(&mut self, backend_state: Option<u32>) -> &mut Self {
        unsafe {
            self.native_mut()
                .resetContext(backend_state.unwrap_or(sb::kAll_GrBackendState))
        }
        self
    }

    pub fn reset_gl_texture_bindings(&mut self) -> &mut Self {
        unsafe { self.native_mut().resetGLTextureBindings() }
        self
    }

    pub fn oomed(&mut self) -> bool {
        unsafe { self.native_mut().oomed() }
    }

    pub fn resource_cache_limits(&self) -> ResourceCacheLimits {
        let mut resources = 0;
        let mut resource_bytes = 0;
        unsafe {
            self.native()
                .getResourceCacheLimits(&mut resources, &mut resource_bytes)
        }
        ResourceCacheLimits {
            max_resources: resources.try_into().unwrap(),
            max_resource_bytes: resource_bytes,
        }
    }


    pub fn resource_cache_limit(&self) -> usize {
        unsafe { self.native().getResourceCacheLimit() }
    }

    pub fn resource_cache_usage(&self) -> ResourceCacheUsage {
        let mut resource_count = 0;
        let mut resource_bytes = 0;
        unsafe {
            self.native()
                .getResourceCacheUsage(&mut resource_count, &mut resource_bytes)
        }
        ResourceCacheUsage {
            resource_count: resource_count.try_into().unwrap(),
            resource_bytes,
        }
    }

    pub fn resource_cache_purgeable_bytes(&self) -> usize {
        unsafe { self.native().getResourceCachePurgeableBytes() }
    }

    pub fn set_resource_cache_limits(&mut self, limits: ResourceCacheLimits) {
        unsafe {
            self.native_mut().setResourceCacheLimits(
                limits.max_resources.try_into().unwrap(),
                limits.max_resource_bytes,
            )
        }
    }

    pub fn set_resource_cache_limit(&mut self, max_resource_bytes: usize) {
        unsafe { self.native_mut().setResourceCacheLimit(max_resource_bytes) }
    }



    pub fn release_resources_and_abandon(&mut self) -> &mut Self {
        unsafe { sb::GrDirectContext_releaseResourcesAndAbandonContext(self.native_mut() as *mut _ as _) }
        self
    }

    pub fn free_gpu_resources(&mut self) -> &mut Self {
        unsafe { sb::GrDirectContext_freeGpuResources(self.native_mut() as *mut _ as _) }
        self
    }

    pub fn perform_deferred_cleanup(&mut self, not_used: Duration) -> &mut Self {
        unsafe {
            sb::C_GrContext_performDeferredCleanup(
                self.native_mut(),
                not_used.as_millis().try_into().unwrap(),
            )
        }
        self
    }

    pub fn purge_unlocked_resources(
        &mut self,
        bytes_to_purge: Option<usize>,
        prefer_scratch_resources: bool,
    ) -> &mut Self {
        unsafe {
            match bytes_to_purge {
                Some(bytes_to_purge) => self
                    .native_mut()
                    .purgeUnlockedResources(bytes_to_purge, prefer_scratch_resources),
                None => self
                    .native_mut()
                    .purgeUnlockedResources1(prefer_scratch_resources),
            }
        }
        self
    }


    // TODO: wait()

    pub fn flush_and_submit(&mut self) -> &mut Self {
        unsafe { sb::C_GrContext_flushAndSubmit(self.native_mut()) }
        self
    }

    pub fn flush_with_info(&mut self, info: &super::FlushInfo) -> super::SemaphoresSubmitted {
        unsafe { self.native_mut().flush(info.native()) }
    }

    #[deprecated(since = "0.30.0", note = "use flush_and_submit()")]
    pub fn flush(&mut self) -> &mut Self {
        self.flush_and_submit()
    }

    // TODO: flush(GrFlushInfo, ..)

    pub fn submit(&mut self, sync_cpu: impl Into<Option<bool>>) -> bool {
        unsafe { self.native_mut().submit(sync_cpu.into().unwrap_or(false)) }
    }

    pub fn check_async_work_completion(&mut self) {
        unsafe { self.native_mut().checkAsyncWorkCompletion() }
    }

    pub fn supports_distance_field_text(&self) -> bool {
        unsafe { self.native().supportsDistanceFieldText() }
    }
    // TODO: add variant with GpuFinishedProc / GpuFinishedContext
    pub fn set_backend_texture_state(
        &mut self,
        backend_texture: &super::BackendTexture,
        state: &BackendSurfaceMutableState,
    ) -> bool {
        unsafe {
            self.native_mut().setBackendTextureState(
                backend_texture.native(),
                state.native(),
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            )
        }
    }

    // TODO: add variant with GpuFinishedProc / GpuFinishedContext
    pub fn set_backend_render_target_state(
        &mut self,
        target: &super::BackendRenderTarget,
        state: &super::BackendSurfaceMutableState,

    ) -> bool {
        unsafe {
            self.native_mut().setBackendRenderTargetState(
                target.native(),
                state.native(),
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            )
        }
    }

    // TODO: wrap deleteBackendTexture(),

    pub fn precompile_shader(&mut self, key: &Data, data: &Data) -> bool {
        unsafe {
            self.native_mut()
                .precompileShader(key.native(), data.native())
        }
    }

    #[cfg(feature = "vulkan")]
    pub fn new_vulkan<'a>(
        backend_context: &vk::BackendContext,
        options: impl Into<Option<&'a ContextOptions>>,
    ) -> Option<DirectContext> {
        unsafe {
            let end_resolving = backend_context.begin_resolving();
            let context = DirectContext::from_ptr(sb::C_GrDirectContext_MakeVulkan(
                backend_context.native.as_ptr() as _,
                options.into().native_ptr_or_null(),
            ));
            drop(end_resolving);
            context
        }
    }

    /// # Safety
    /// This function is unsafe because `device` and `queue` are untyped handles which need to exceed the
    /// lifetime of the context returned.
    #[cfg(feature = "metal")]
    pub unsafe fn new_metal<'a>(
        device: *mut std::ffi::c_void,
        queue: *mut std::ffi::c_void,
        options: impl Into<Option<&'a ContextOptions>>,
    ) -> Option<DirectContext> {
        DirectContext::from_ptr(sb::C_GrContext_MakeMetal(
            device,
            queue,
            options.into().native_ptr_or_null(),
        ))
    }

    #[cfg(feature = "d3d")]
    pub unsafe fn new_d3d<'a>(
        backend_context: &d3d::BackendContext,
        options: impl Into<Option<&'a ContextOptions>>,
    ) -> Option<DirectContext> {
        DirectContext::from_ptr(sb::C_GrDirectContext_MakeDirect3D(
            backend_context.native(),
            options.into().native_ptr_or_null(),
        ))
    }
}
