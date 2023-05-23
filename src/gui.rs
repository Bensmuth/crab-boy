// STOLEN FROM https://github.com/parasyte/pixels/blob/main/examples/imgui-winit/src/gui.rs
use pixels::{wgpu, PixelsContext};
use std::time::Instant;
use crate::cpu;

/// Manages all state required for rendering Dear ImGui over `Pixels`.
pub(crate) struct Gui {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    last_cursor: Option<imgui::MouseCursor>,
    registers_open: bool,
    flow_control_open: bool,

}

impl Gui {
    /// Create Dear ImGui.
    pub(crate) fn new(window: &winit::window::Window, pixels: &pixels::Pixels) -> Self {
        // Create Dear ImGui context
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        // Initialize winit platform support
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        // Configure Dear ImGui fonts
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Create Dear ImGui WGPU renderer
        let device = pixels.device();
        let queue = pixels.queue();
        let config = imgui_wgpu::RendererConfig {
            texture_format: pixels.render_texture_format(),
            ..Default::default()
        };
        let renderer = imgui_wgpu::Renderer::new(&mut imgui, device, queue, config);

        // Return GUI context
        Self {
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            last_cursor: None,
            registers_open: false,
            flow_control_open: false,
        }
    }

    /// Prepare Dear ImGui.
    pub(crate) fn prepare(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(), winit::error::ExternalError> {
        // Prepare Dear ImGui
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;
        self.platform.prepare_frame(self.imgui.io_mut(), window)
    }

    /// Render Dear ImGui.
    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
        cpu: &mut cpu::Cpu,
        run: &mut bool,
    ) -> imgui_wgpu::RendererResult<()> {
        // Start a new Dear ImGui frame and update the cursor
        let ui = self.imgui.frame();

        let mouse_cursor = ui.mouse_cursor();
        if self.last_cursor != mouse_cursor {
            self.last_cursor = mouse_cursor;
            self.platform.prepare_render(&ui, window);
        }


        // Draw windows and GUI elements here
        let mut registers_open = false;
        let mut flow_control_open = false;
        ui.main_menu_bar(|| {
            ui.menu("Debug", || {
                registers_open = imgui::MenuItem::new("Registers").build(&ui);
                flow_control_open = imgui::MenuItem::new("Flow Control").build(&ui);
            });
        });
        if registers_open {
            self.registers_open = true;
        }

        if self.registers_open {
            imgui::Window::new("Registers")
                .opened(&mut self.registers_open)
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(cpu.get_register_debug_string());
                });

        }

        if flow_control_open{
            self.flow_control_open = true;
        }

        let mut step = false;
        let mut torun = false;
        if self.flow_control_open {
            imgui::Window::new("Flow Control")
                .opened(&mut self.flow_control_open)
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    step = ui.button("Step");
                    torun = ui.button("Pause/Run")
                });

        }

        if step{
            cpu.tick();
        }
        if torun{
            *run = !*run;
        }


        // Render Dear ImGui with WGPU
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), &context.queue, &context.device, &mut rpass)
    }

    /// Handle any outstanding events.
    pub(crate) fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }
}
