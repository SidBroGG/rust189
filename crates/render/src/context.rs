use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// Manages the `wgpu` graphics pipeline, hardware device, command queue, and presentation surface.
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

impl GpuContext {
    /// Initializes a new GPU context with high-performance adapter and configured surface.
    pub async fn new(window: Arc<Window>, vsync: bool) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Instance initialization targeting DirectX 12 / Vulkan / Metal
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Create surface targeting the Arc<Window>
        let surface = instance.create_surface(window.clone())?;

        // Request high-performance adapter for low latency and high framerates
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter for surface"))?;

        let adapter_info = adapter.get_info();
        tracing::info!(
            name = %adapter_info.name,
            backend = ?adapter_info.backend,
            device_type = ?adapter_info.device_type,
            "Acquired GPU adapter"
        );

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rust189-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        device.on_uncaptured_error(Box::new(|err| {
            tracing::error!(error = %err, "wgpu validation error");
        }));

        // Configure surface format and present mode
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = choose_present_mode(&surface_caps, vsync);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        tracing::info!(
            width = width,
            height = height,
            format = ?surface_format,
            present_mode = ?present_mode,
            "wgpu surface successfully configured"
        );

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
            size,
        })
    }

    /// Resizes the surface when the window dimensions change.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// Sets vertical synchronization (VSync).
    pub fn set_vsync(&mut self, vsync: bool) {
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let present_mode = choose_present_mode(&surface_caps, vsync);

        if self.surface_config.present_mode != present_mode {
            self.surface_config.present_mode = present_mode;
            self.surface.configure(&self.device, &self.surface_config);
            tracing::info!(?present_mode, "Updated GPU surface present mode");
        }
    }

    /// Acquires the next surface texture for rendering.
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}

/// Chooses the optimal present mode based on surface capabilities and VSync preference.
pub(crate) fn choose_present_mode(
    surface_caps: &wgpu::SurfaceCapabilities,
    vsync: bool,
) -> wgpu::PresentMode {
    if vsync {
        // Fifo is guaranteed by the spec to be supported on all platforms and provides true, stable VSync without tearing or jitter.
        wgpu::PresentMode::Fifo
    } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
        wgpu::PresentMode::Mailbox
    } else if surface_caps.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
        wgpu::PresentMode::AutoNoVsync
    } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
        wgpu::PresentMode::Immediate
    } else {
        wgpu::PresentMode::Fifo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_caps(present_modes: Vec<wgpu::PresentMode>) -> wgpu::SurfaceCapabilities {
        wgpu::SurfaceCapabilities {
            formats: vec![wgpu::TextureFormat::Rgba8UnormSrgb],
            present_modes,
            alpha_modes: vec![wgpu::CompositeAlphaMode::Auto],
            usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }

    #[test]
    fn test_choose_present_mode_vsync() {
        let caps = mock_caps(vec![wgpu::PresentMode::Fifo, wgpu::PresentMode::AutoVsync]);
        assert_eq!(choose_present_mode(&caps, true), wgpu::PresentMode::Fifo);

        let caps_fifo_only = mock_caps(vec![wgpu::PresentMode::Fifo]);
        assert_eq!(choose_present_mode(&caps_fifo_only, true), wgpu::PresentMode::Fifo);
    }

    #[test]
    fn test_choose_present_mode_no_vsync() {
        let caps = mock_caps(vec![
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::AutoNoVsync,
        ]);
        assert_eq!(choose_present_mode(&caps, false), wgpu::PresentMode::AutoNoVsync);

        let caps_imm = mock_caps(vec![wgpu::PresentMode::Fifo, wgpu::PresentMode::Immediate]);
        assert_eq!(choose_present_mode(&caps_imm, false), wgpu::PresentMode::Immediate);

        let caps_mailbox = mock_caps(vec![wgpu::PresentMode::Fifo, wgpu::PresentMode::Mailbox]);
        assert_eq!(choose_present_mode(&caps_mailbox, false), wgpu::PresentMode::Mailbox);

        let caps_fifo = mock_caps(vec![wgpu::PresentMode::Fifo]);
        assert_eq!(choose_present_mode(&caps_fifo, false), wgpu::PresentMode::Fifo);
    }
}


