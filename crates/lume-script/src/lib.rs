//! Adaptador de scripts de LumeArt.
//!
//! Ejecuta JavaScript con QuickJS (`rquickjs`, quickjs-ng enlazado
//! estáticamente) sin acceso a archivos, red ni reloj y con límite de tiempo
//! por llamada, como hacía goja en pxcli. [`anim`] implementa el puerto
//! `AnimEngine` (`anim.js` con `prelude.js`) y [`vec`], `VecEngine`
//! (`dibujo.js` con `lib.js`).
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

pub mod anim;
pub mod calls;
pub mod gomath;
pub mod vec;
mod vec_json;

pub use anim::QuickJsAnim;
pub use vec::QuickJsVec;
