//! Window sizing and the browser loading overlay.
use winit::window::Window;

/// Dismisses the loading overlay once the first world frame is on screen.
#[cfg(target_arch = "wasm32")]
pub(crate) fn loaded() {
    call_overlay(&[1.0.into()]);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn loaded() {}

/// Calls `window.aoaProgress(...)` when the page defines it.
#[cfg(target_arch = "wasm32")]
pub(crate) fn call_overlay(args: &[wasm_bindgen::JsValue]) {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(function) = js_sys::Reflect::get(&window, &"aoaProgress".into()) else {
        return;
    };
    if let Some(function) = function.dyn_ref::<js_sys::Function>() {
        let _ = function.apply(&window, &args.iter().collect::<js_sys::Array>());
    }
}

/// The drawable size in device pixels. In the browser winit can leave the
/// canvas at its CSS size on high-DPI screens while pointer events arrive in
/// device pixels, so the canvas is sized here from its layout box instead.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn physical_size(window: &Window) -> (u32, u32) {
    let size = window.inner_size();
    (size.width.max(1), size.height.max(1))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn physical_size(window: &Window) -> (u32, u32) {
    use winit::platform::web::WindowExtWebSys;
    let Some(canvas) = window.canvas() else {
        let size = window.inner_size();
        return (size.width.max(1), size.height.max(1));
    };
    let scale = window.scale_factor();
    let width = ((canvas.client_width() as f64 * scale).round() as u32).max(1);
    let height = ((canvas.client_height() as f64 * scale).round() as u32).max(1);
    if canvas.width() != width || canvas.height() != height {
        canvas.set_width(width);
        canvas.set_height(height);
    }
    (width, height)
}
