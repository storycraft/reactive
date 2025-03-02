use core::{ffi::CStr, mem, num::NonZeroU32};
use std::ffi::CString;

use gl::types::GLint;
use glutin::{
    config::{Config, ConfigTemplateBuilder, GetGlConfig, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
    },
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow, finalize_window};
use reactivity_winit::winit::{
    event_loop::ActiveEventLoop,
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes, WindowId},
};
use skia_safe::{
    Canvas, ColorType,
    gpu::{
        self, DirectContext, SurfaceOrigin, backend_render_targets,
        gl::{FramebufferInfo, Interface},
    },
};

#[derive(Debug)]
pub struct Context {
    pub id: WindowId,

    skia_surface: skia_safe::Surface,
    gr_cx: DirectContext,

    num_samples: u8,
    stencil_size: u8,
    fb_info: FramebufferInfo,

    // https://github.com/rust-skia/rust-skia/issues/476
    // Skia hangs if gl contexts drop before skia context
    gl_cx: PossiblyCurrentContext,
    gl_surface: glutin::surface::Surface<WindowSurface>,
}

impl Context {
    pub fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.gl_surface.resize(&self.gl_cx, width, height);
        self.skia_surface = create_skia_surface(
            (width.get() as _, height.get() as _),
            self.fb_info,
            &mut self.gr_cx,
            self.num_samples as _,
            self.stencil_size as _,
        );
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.skia_surface.canvas()
    }

    pub fn render(&mut self) {
        self.gr_cx.flush_and_submit();
        self.gl_surface.swap_buffers(&self.gl_cx).unwrap();
    }
}

#[derive(Debug)]
pub enum WindowState {
    Uninit { builder: DisplayBuilder },
    Init(Context),
    Suspended { cx: NotCurrentContext },
    Invalid,
}

impl WindowState {
    pub const fn new(builder: DisplayBuilder) -> Self {
        Self::Uninit { builder }
    }

    // TODO:: error handling
    pub fn create_window(
        &mut self,
        el: &ActiveEventLoop,
        attr: &WindowAttributes,
    ) -> Option<Window> {
        let (window, gl_config, gl_cx) = match mem::replace(self, WindowState::Invalid) {
            WindowState::Uninit { builder } => {
                let template = ConfigTemplateBuilder::new().with_alpha_size(8);

                let Ok((Some(window), gl_config)) = builder.build(el, template, gl_config_picker)
                else {
                    return None;
                };

                println!("Picked a config with {} samples", gl_config.num_samples());

                let gl_cx = create_gl_context(&window, &gl_config);
                (window, gl_config, gl_cx)
            }

            WindowState::Suspended { cx } => {
                println!("Recreating window in `resumed`");
                // Pick the config which we already use for the context.
                let gl_config = cx.config();
                match finalize_window(el, attr.clone(), &gl_config) {
                    Ok(window) => (window, gl_config, cx),
                    Err(_err) => {
                        return None;
                    }
                }
            }

            state @ WindowState::Init(_) => {
                *self = state;
                return None;
            }

            WindowState::Invalid => {
                return None;
            }
        };

        let Ok(attrs) = window.build_surface_attributes(Default::default()) else {
            return None;
        };
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_cx = gl_cx.make_current(&gl_surface).unwrap();

        gl::load_with(|s| {
            gl_config
                .display()
                .get_proc_address(CString::new(s).unwrap().as_c_str())
        });
        let interface = Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            gl_config
                .display()
                .get_proc_address(CString::new(name).unwrap().as_c_str())
        })
        .expect("Could not create interface");

        let mut gr_cx = gpu::direct_contexts::make_gl(interface, None)
            .expect("Could not create direct context");

        let fb_info = {
            let mut fboid: GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let num_samples = gl_config.num_samples();
        let stencil_size = gl_config.stencil_size();

        let size = window.inner_size();
        let skia_surface = create_skia_surface(
            (size.width as _, size.height as _),
            fb_info,
            &mut gr_cx,
            num_samples as _,
            stencil_size as _,
        );

        println!(
            "Running on {}",
            (unsafe { CStr::from_ptr(gl::GetString(gl::RENDERER).cast()) }).to_string_lossy()
        );
        println!(
            "OpenGL Version {}",
            (unsafe { CStr::from_ptr(gl::GetString(gl::VERSION).cast()) }).to_string_lossy()
        );
        println!(
            "Shaders version on {}",
            (unsafe { CStr::from_ptr(gl::GetString(gl::SHADING_LANGUAGE_VERSION).cast()) })
                .to_string_lossy()
        );

        let id = window.id();
        *self = WindowState::Init(Context {
            id,
            gl_cx,
            gr_cx,
            num_samples,
            stencil_size,
            fb_info,
            gl_surface,
            skia_surface,
        });

        Some(window)
    }

    pub fn suspend(&mut self) {
        *self = match mem::replace(self, WindowState::Invalid) {
            WindowState::Init(cx) => WindowState::Suspended {
                cx: cx.gl_cx.make_not_current().unwrap(),
            },

            state => state,
        }
    }
}

fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() < accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    }
}

fn create_skia_surface(
    size: (i32, i32),
    fb_info: FramebufferInfo,
    gr_context: &mut DirectContext,
    num_samples: usize,
    stencil_size: usize,
) -> skia_safe::Surface {
    let backend_render_target =
        backend_render_targets::make_gl(size, num_samples, stencil_size, fb_info);

    gpu::surfaces::wrap_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("Could not create skia surface")
}
