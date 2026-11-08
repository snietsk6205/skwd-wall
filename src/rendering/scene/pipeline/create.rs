use iced::widget::shader::Pipeline;

use super::super::atlas;
use super::model::{BrowserScenePipeline, Globals, ScenePipeline, ThumbFormat};
use super::sandy_resources::{make_sandy_blit_layout, make_sandy_layout, make_sandy_uniform};
use super::textures::{make_array_texture, make_near_texture, make_preview_texture};
use super::transition_resources::{make_transition_layout, make_transition_uniform};
use crate::contracts::rendering::InstanceRaw;

const INITIAL_INSTANCES: usize = 128;

impl Pipeline for ScenePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let mut pipeline = Self::create(device, format, ThumbFormat::Compressed);
        // Built now, before the first frame, instead of in the middle of the
        // first layout switch.
        pipeline.ensure_transition_pipeline(device);
        pipeline
    }
}

impl ScenePipeline {
    fn create(device: &wgpu::Device, format: wgpu::TextureFormat, thumbs: ThumbFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skwd item"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/item.wgsl").into()),
        });

        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skwd globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skwd scene layout"),
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
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("skwd sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let compressed = crate::contracts::preview::compressed_thumbnails();
        let bc = matches!(thumbs, ThumbFormat::Compressed)
            && compressed
            && device.features().contains(wgpu::Features::TEXTURE_COMPRESSION_BC);
        let budget = match thumbs {
            ThumbFormat::Compressed => atlas::budget(bc),
            ThumbFormat::Rgba => atlas::browser_budget(compressed),
        };
        let near_format = if bc {
            wgpu::TextureFormat::Bc7RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let far_format = if bc {
            wgpu::TextureFormat::Bc1RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        log::info!(
            "thumbnail atlas formats: near={near_format:?}, far={far_format:?}, device_bc={}",
            device.features().contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
        );
        let atlas_size = budget.size;
        let near_layers = budget.near_init_layers;
        let far_layers = budget.far_initial_layers;
        let near_tex = make_near_texture(device, near_format, near_layers);
        let far_tex =
            make_array_texture(device, far_layers, "skwd far atlas", far_format, atlas_size);
        let preview_tex = make_preview_texture(
            device,
            "skwd preview",
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        );
        let preview_out_tex = make_preview_texture(
            device,
            "skwd preview out",
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let bind_group = Self::build_bind_group(
            device,
            &bind_layout,
            &globals,
            &near_tex,
            &far_tex,
            &preview_tex,
            &sampler,
        );

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skwd pipeline layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x4,
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
                4 => Float32x4,
                5 => Float32x4,
                6 => Float32x4,
                7 => Float32x4,
                8 => Uint32x4,
                9 => Float32x4,
                10 => Float32x4,
                11 => Float32x4,
                12 => Float32x4,
                13 => Float32x4,
                14 => Float32x4,
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skwd item pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[instance_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skwd instances"),
            size: (std::mem::size_of::<InstanceRaw>() * INITIAL_INSTANCES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_capacity = instances.size();
        let old_instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skwd old instances"),
            size: (std::mem::size_of::<InstanceRaw>() * INITIAL_INSTANCES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let old_capacity = old_instances.size();

        let trans_layout = make_transition_layout(device);
        let trans_uniform = make_transition_uniform(device);
        let sandy_layout = make_sandy_layout(device);
        let sandy_uniform = make_sandy_uniform(device);
        let sandy_blit_layout = make_sandy_blit_layout(device);
        Self {
            pipeline,
            globals,
            bind_layout,
            bind_group,
            sampler,
            near_tex,
            far_tex,
            preview_tex,
            preview_out_tex,
            out_copy: false,
            near_format,
            far_format,
            atlas_size,
            near_cap: budget.near_cap,
            near_grow_chunk: budget.near_grow_chunk,
            near_layers,
            far_layers,
            instances,
            instance_capacity,
            instance_count: 0,
            old_instances,
            old_capacity,
            old_count: 0,
            format,
            targets: None,
            targets_used: None,
            old_drawn: std::sync::atomic::AtomicBool::new(false),
            trans_pipeline: None,
            trans_layout,
            trans_uniform,
            sandy_layout,
            sandy_uniform,
            sandy_pipeline: None,
            sandy_bind: None,
            sandy_target: None,
            sandy_blit_layout,
            sandy_blit_pipeline: None,
            sandy_blit_bind: None,
            trans_bind: None,
            last_main: None,
            last_old: None,
            embedded_origin: [0.0, 0.0],
        }
    }
}

impl Pipeline for BrowserScenePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self(ScenePipeline::create(device, format, ThumbFormat::Rgba))
    }
}
