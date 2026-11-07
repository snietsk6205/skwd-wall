use std::borrow::Cow;

use crate::contracts::rendering::{InstanceRaw, SandySnap};

pub const PROJECTED: u32 = 128;
pub const RIBBON_COLUMNS: u32 = 4096;
pub const GHOST: u32 = 8192;
pub const BACKDROP: u32 = 16384;
pub const BACKFACE: u32 = 32768;
pub const MUTED: u32 = 65_536;
pub const UNFRAMED_RECT: u32 = 131_072;
/// Custom build: draw the card upside down, fading out (Coverflow reflection).
pub const REFLECTION: u32 = 262_144;

pub(crate) fn sandy_video_in(progress: f32, live: bool) -> f32 {
    if !live {
        return 0.0;
    }
    let t = ((progress - 0.10) / (0.35 - 0.10)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn sandy_video_out(valid: bool, ring: bool) -> f32 {
    if valid && !ring { 1.0 } else { 0.0 }
}

pub(super) fn sandy_vertices(grid: [f32; 2]) -> u32 {
    (grid[0] as u32).max(1) * (grid[1] as u32).max(1) * 12
}

pub(super) fn sandy_is_scaled(resolution_scale: f32) -> bool {
    resolution_scale < 0.999
}

pub(super) fn sandy_target_dimensions(
    bounds_width: f32,
    bounds_height: f32,
    surface_scale: f32,
    resolution_scale: f32,
) -> (u32, u32) {
    let scale = surface_scale * resolution_scale.clamp(0.1, 1.0);
    let width = (bounds_width * scale).ceil().max(1.0) as u32;
    let height = (bounds_height * scale).ceil().max(1.0) as u32;
    (width, height)
}

pub(super) fn draw_sandy<'pass>(
    pass: &mut wgpu::RenderPass<'pass>,
    pipeline: &'pass wgpu::RenderPipeline,
    bind_group: &'pass wgpu::BindGroup,
    snapshot: &SandySnap,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    let vertices = sandy_vertices(snapshot.grid);
    if snapshot.swirl >= 0.55 || snapshot.knobs[9] >= 16.5 {
        pass.draw(vertices / 2..vertices, 0..1);
    } else {
        pass.draw(0..vertices, 0..1);
    }
}

pub(super) fn scale_instances(
    instances: &[InstanceRaw],
    surface_scale: f32,
) -> Cow<'_, [InstanceRaw]> {
    if (surface_scale - 1.0).abs() < 1e-4 {
        return Cow::Borrowed(instances);
    }
    let mut scaled = instances.to_vec();
    for instance in &mut scaled {
        for value in &mut instance.rect {
            *value *= surface_scale;
        }
        for value in &mut instance.radii {
            *value *= surface_scale;
        }
        instance.params[0] *= surface_scale;
        instance.params[1] *= surface_scale;
        instance.shape[0] *= surface_scale;
        if instance.misc[3] & PROJECTED != 0 {
            for value in
                instance.quad_a.iter_mut().chain(&mut instance.quad_b).chain(&mut instance.quad_l)
            {
                *value *= surface_scale;
            }
        }
    }
    Cow::Owned(scaled)
}

pub(super) fn place_instances(
    instances: &[InstanceRaw],
    surface_scale: f32,
    origin: [f32; 2],
) -> Cow<'_, [InstanceRaw]> {
    if origin[0].abs() < 1e-4 && origin[1].abs() < 1e-4 {
        return scale_instances(instances, surface_scale);
    }
    let mut placed = scale_instances(instances, surface_scale).into_owned();
    let ox = origin[0] * surface_scale;
    let oy = origin[1] * surface_scale;
    for instance in &mut placed {
        instance.rect[0] += ox;
        instance.rect[1] += oy;
        if instance.misc[3] & PROJECTED != 0 {
            for quad in [&mut instance.quad_a, &mut instance.quad_b] {
                quad[0] += ox;
                quad[1] += oy;
                quad[2] += ox;
                quad[3] += oy;
            }
        }
    }
    Cow::Owned(placed)
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
