use super::bindings::vsfs_pipeline;
use super::model::{ScenePipeline, TransUniformRaw, TransitionTargets};

pub(super) fn make_transition_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("skwd transition layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub(super) fn make_transition_uniform(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("skwd transition uniform"),
        size: std::mem::size_of::<TransUniformRaw>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// Cross-fade targets are kept this long after a layout switch, so a run of
/// switches reuses them instead of allocating two screen-sized textures each time.
pub(super) const TARGETS_KEEP: Duration = Duration::from_secs(10);

/// Whether the kept cross-fade targets should go: the picker is hidden, or
/// they have not been used for [`TARGETS_KEEP`].
pub(super) fn release_targets_due(visible: bool, last_used: Option<Instant>, now: Instant) -> bool {
    !visible || last_used.is_none_or(|used| now.saturating_duration_since(used) >= TARGETS_KEEP)
}

impl ScenePipeline {
    pub(super) fn release_transition_targets(&mut self) {
        self.trans_bind = None;
        self.targets = None;
        self.targets_used = None;
        self.old_drawn.store(false, Ordering::Relaxed);
    }

    /// The old layout must be drawn into target A again (it or A changed).
    pub(super) fn invalidate_old_layout(&self) {
        self.old_drawn.store(false, Ordering::Relaxed);
    }

    pub(super) fn ensure_transition_pipeline(&mut self, device: &wgpu::Device) {
        if self.trans_pipeline.is_some() {
            return;
        }
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skwd transition"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../../shaders/transition.wgsl").into(),
            ),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skwd transition pipeline layout"),
            bind_group_layouts: &[&self.trans_layout],
            push_constant_ranges: &[],
        });
        self.trans_pipeline =
            Some(vsfs_pipeline(device, "skwd transition pipeline", &layout, &shader, self.format));
    }

    pub(super) fn ensure_transition_targets(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) {
        if let Some(targets) = &self.targets
            && targets.width == width
            && targets.height == height
        {
            return;
        }
        let make = |label: &str| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let a_view = make("skwd transition a");
        let b_view = make("skwd transition b");
        self.invalidate_old_layout();
        self.trans_bind = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skwd transition bind"),
            layout: &self.trans_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.trans_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        }));
        self.targets = Some(TransitionTargets { a_view, b_view, width, height });
    }
}
