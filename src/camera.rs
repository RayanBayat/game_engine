use crate::config::*;
use crate::rect::GpuBinding;

use wgpu::util::DeviceExt;

pub struct Camera {
    pub camera_object: CameraObject,
    pub render_camera: GpuBinding,
}

pub struct CameraObject {
    pub position: [f32; 2],
}

impl Camera {
    pub fn new(
        position: [f32; 2],
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            camera_object: CameraObject { position },
            render_camera: create_render_camera(device, bind_group_layout, position),
        }
    }

    pub fn update(&mut self, player_pos: [f32; 2], screen_size: [f32; 2]) {
        self.camera_object.position[0] = player_pos[0] - screen_size[0] / 2.0;
        self.camera_object.position[1] = player_pos[1] - screen_size[1] / 2.0;
    }

    pub fn position(&self) -> [f32; 2] {
        self.camera_object.position
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 2],
    pub _padding: UniformCameraPadding, // change as needed to ensure 16-byte alignment
}

impl CameraUniform {
    pub fn new(position: [f32; 2]) -> Self {
        Self {
            position,
            _padding: UNIFORM_CAMERA_PADDING,
        }
    }
}

pub fn create_render_camera(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    position: [f32; 2],
) -> GpuBinding {
    let uniform = CameraUniform {
        position,
        _padding: UNIFORM_CAMERA_PADDING, // change as needed to ensure 16-byte alignment
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Camera Bind Group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    GpuBinding {
        uniform_buffer,
        bind_group,
    }
}
