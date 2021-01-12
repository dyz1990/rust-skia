#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings;
pub use bindings::*;
unsafe impl Send for SkBitmap{}

mod defaults;
pub use defaults::*;

mod impls;
pub use impls::*;

#[cfg(feature = "textlayout")]
pub mod icu;
