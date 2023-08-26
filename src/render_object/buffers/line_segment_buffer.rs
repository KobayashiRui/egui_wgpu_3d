use eframe::egui_wgpu::wgpu;
use nalgebra::{Vector3, Vector4, Rotation3};


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineSegment {
    //pub position: [f32; 3],
    pub point0: Vector3<f32>,
    pub point1: Vector3<f32>,
    //pub color: [f32; 3],
    //pub color: Vector3<f32>,
}

impl LineSegment {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineSegment>() as wgpu::BufferAddress,
            //step_mode: wgpu::VertexStepMode::Vertex,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineMaterial {
    pub color: Vector4<f32>,
    pub depth_bias: f32,
    pub width: f32,
    pub padding0: f32,
    pub padding1: f32,
}