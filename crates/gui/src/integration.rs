use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window;

/// Encapsulates egui context, winit platform integration, and wgpu rendering pass.
pub struct GuiIntegration {
    pub context: Context,
    pub winit_state: State,
    pub renderer: Renderer,
}

impl GuiIntegration {
    /// Creates a new GUI integration for the given window and wgpu device.
    pub fn new(
        window: &Window,
        device: &Device,
        surface_format: TextureFormat,
    ) -> Self {
        let context = Context::default();
        crate::theme::setup_minecraft_theme(&context);

        let winit_state = State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = Renderer::new(device, surface_format, None, 1, false);

        Self {
            context,
            winit_state,
            renderer,
        }
    }

    /// Feeds winit WindowEvents into egui.
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.winit_state.on_window_event(window, event);
        response.consumed
    }

    /// Prepares egui UI for the current frame.
    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.winit_state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }

    /// Finishes the egui UI frame, uploads buffers/textures to GPU, and records the render pass.
    pub fn end_frame_and_render(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        color_view: &TextureView,
        screen_descriptor: &ScreenDescriptor,
    ) {
        let full_output = self.context.end_pass();
        self.winit_state
            .handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self
            .context
            .tessellate(full_output.shapes, screen_descriptor.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &paint_jobs, screen_descriptor);

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve background
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Cast lifetime to match egui_wgpu render call
            let mut rpass: RenderPass<'static> = unsafe { std::mem::transmute(render_pass) };
            self.renderer.render(&mut rpass, &paint_jobs, screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
