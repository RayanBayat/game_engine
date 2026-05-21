use crate::vertex::Vertex;

use wgpu::util::DeviceExt;

pub struct Rect {
    pub rect_object: RectObject,
    pub render_rect: RenderRect,
}

pub struct RectObject {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

pub struct RenderRect {
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Rect {
    pub fn new(position: [f32; 2], size: [f32; 2], color: [f32; 4], device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, screen_size: [f32; 2]) -> Self {
        Rect {
            rect_object: RectObject { position, size, color },
            render_rect: create_render_rect(device, bind_group_layout, position, size, screen_size, color),
        }
    }

    pub fn position(&self) -> [f32; 2] {
        self.rect_object.position
    }

    pub fn size(&self) -> [f32; 2] {
        self.rect_object.size
    }

    pub fn color(&self) -> [f32; 4] {
        self.rect_object.color
    }

    pub fn move_by(&mut self, delta: [f32; 2]) {
        self.rect_object.position[0] += delta[0];
        self.rect_object.position[1] += delta[1];
    }

    pub fn move_to(&mut self, new_position: [f32; 2]) {
        self.rect_object.position = new_position;
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        return other.position()[0] < self.position()[0] + self.size()[0] &&
            other.position()[0] + other.size()[0] > self.position()[0] &&
            other.position()[1] < self.position()[1] + self.size()[1] &&
            other.position()[1] + other.size()[1] > self.position()[1];
    }

}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectUniform {
    pub position: [f32; 2],
    pub screen_size: [f32; 2],
    pub size: [f32; 2],
    pub _padding: [f32; 2],

    pub color: [f32; 4],
}

pub fn create_render_rect(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    position: [f32; 2],
    size: [f32; 2],
    screen_size: [f32; 2],
    color: [f32; 4],
) -> RenderRect {
    let uniform = RectUniform {
        position,
        size,
        screen_size,
        color,


        _padding: [0.0; 2], // change as needed to ensure 16-byte alignment
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Rect Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Rect Bind Group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    RenderRect {
        uniform_buffer,
        bind_group,
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
    Vertex { position: [1.0, 1.0, 0.0], color: [1.0, 1.0, 1.0] },
];

pub const INDICES: &[u16] = &[
    0, 1, 3, // triangle 1
    1, 2, 3, // triangle 2
];

