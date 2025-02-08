pub mod element;

use core::{
    cell::RefCell,
    ffi::CStr,
    num::NonZeroU32,
    pin::{pin, Pin},
};
use std::ffi::CString;

use element::Element;
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
use glutin_winit::{finalize_window, DisplayBuilder, GlWindow};
use pin_project::pin_project;
use skia_safe::{
    gpu::{
        self, backend_render_targets,
        gl::{FramebufferInfo, Interface},
        DirectContext, SurfaceOrigin,
    },
    Color, ColorType
};
use winit::{
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    event_loop::handler::{EventHandler, HandlerKey},
    state::StateRefCell,
};

#[derive(Debug)]
#[pin_project]
pub struct SkiaWindow<'a, Root> {
    attr: WindowAttributes,
    state: RefCell<WindowState>,
    window: Pin<&'a StateRefCell<Option<Window>>>,
    root: Pin<&'a Root>,
}

impl<'a, Root> SkiaWindow<'a, Root>
where
    Root: for<'b> Element<'b>,
{
    fn new(
        builder: DisplayBuilder,
        attr: WindowAttributes,
        window: Pin<&'a StateRefCell<Option<Window>>>,
        root: Pin<&'a Root>,
    ) -> Self {
        Self {
            state: RefCell::new(WindowState::Uninit {
                builder: builder.with_window_attributes(Some(attr.clone())),
            }),
            attr,
            window,
            root,
        }
    }

    pub async fn render(f: impl FnOnce(Pin<&StateRefCell<Option<Window>>>) -> Pin<&'a Root>) {
        let window = pin!(StateRefCell::new(None));
        let window = window.into_ref();

        let root = f(window);

        let this = pin!(SkiaWindow::new(
            DisplayBuilder::new(),
            WindowAttributes::default(),
            window,
            root
        ));
        let this = this.into_ref();

        HandlerKey::register(this, async move {
            root.setup().await;
        })
        .await;
    }
}

impl<Root: for<'a> Element<'a>> EventHandler for SkiaWindow<'_, Root> {
    fn request_redraw(self: Pin<&Self>) {
        if let Some(window) = &*self.project_ref().window.get_untracked() {
            window.request_redraw();
        }
    }

    fn resumed(self: Pin<&Self>, el: &ActiveEventLoop) {
        let this = self.project_ref();

        // TODO:: error handling
        let Some((window, cx)) = (match this.state.replace(WindowState::Invalid) {
            WindowState::Invalid => {
                println!("GlState is invalid");
                el.exit();
                return;
            }

            state => state.resume(el, self.attr.clone()),
        }) else {
            println!("Window creation failed");
            el.exit();

            return;
        };

        this.state.replace(WindowState::Init(cx));
        this.window.set(Some(window));
    }

    fn suspended(self: Pin<&Self>, _el: &ActiveEventLoop) {
        let this = self.project_ref();
        this.window.set(None);

        this.state
            .replace(this.state.replace(WindowState::Invalid).suspend());
    }

    fn on_window_event(
        self: Pin<&Self>,
        el: &ActiveEventLoop,
        window_id: WindowId,
        event: &mut WindowEvent,
    ) {
        let this = self.project_ref();

        let WindowState::Init(Context {
            id,
            gl_cx,
            gr_cx,
            stencil_size,
            num_samples,
            fb_info,
            gl_surface,
            skia_surface,
            ..
        }) = &mut *this.state.borrow_mut()
        else {
            return;
        };

        if *id != window_id {
            return;
        }

        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                gl_surface.resize(
                    gl_cx,
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap(),
                );
                *skia_surface = create_skia_surface(
                    (size.width as _, size.height as _),
                    *fb_info,
                    gr_cx,
                    *num_samples as _,
                    *stencil_size as _,
                );
            }

            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                el.exit();
            }

            WindowEvent::RedrawRequested => {
                let canvas = skia_surface.canvas();
                canvas.clear(Color::BLACK);

                this.root.draw(canvas);

                gr_cx.flush_and_submit();
                gl_surface.swap_buffers(gl_cx).unwrap();
            }

            _ => {}
        }

        this.root.on_event(el, window_id, event);
    }
}

#[derive(Debug)]
struct Context {
    id: WindowId,
    gl_cx: PossiblyCurrentContext,
    gr_cx: DirectContext,

    num_samples: u8,
    stencil_size: u8,
    fb_info: FramebufferInfo,

    gl_surface: glutin::surface::Surface<WindowSurface>,
    skia_surface: skia_safe::Surface,
}

#[derive(Debug)]
enum WindowState {
    Uninit { builder: DisplayBuilder },
    Init(Context),
    Suspended { cx: NotCurrentContext },
    Invalid,
}

impl WindowState {
    // TODO:: error handling
    pub fn resume(self, el: &ActiveEventLoop, attr: WindowAttributes) -> Option<(Window, Context)> {
        let (window, gl_config, gl_cx) = match self {
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
                match finalize_window(el, attr, &gl_config) {
                    Ok(window) => (window, gl_config, cx),
                    Err(_err) => {
                        return None;
                    }
                }
            }

            _ => {
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
        Some((
            window,
            Context {
                id,
                gl_cx,
                gr_cx,
                num_samples,
                stencil_size,
                fb_info,
                gl_surface,
                skia_surface,
            },
        ))
    }

    pub fn suspend(self) -> Self {
        if let WindowState::Init(cx) = self {
            WindowState::Suspended {
                cx: cx.gl_cx.make_not_current().unwrap(),
            }
        } else {
            self
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
