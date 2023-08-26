use core::num;
use std::{any::TypeId, sync::Arc};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu, CallbackFn},
};

use uuid::Uuid;

//use crate::render_object::buffers;
use super::buffers::*;

pub struct PolylineObject{
    pub id: uuid::Uuid,
    pub line_segments: Box<[line_segment_buffer::LineSegment]>,
    pub vertex_buffer: wgpu::Buffer
}

impl PolylineObject {
    pub fn new(device: &wgpu::Device, line_segments: Box<[line_segment_buffer::LineSegment]>) -> Self {  
        let id = uuid::Uuid::new_v4();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&id.to_string()),
            contents: bytemuck::cast_slice(&line_segments),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self{
            id,
            line_segments,
            vertex_buffer
        }
    }

}

pub struct PolylineRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform_buffer: wgpu::Buffer, //camera uniform buffer
    pub polyline_material_bind_group: wgpu::BindGroup,
    pub polyline_material_uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub data: Vec<PolylineObject>,
}

impl PolylineRenderResources {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState, camera_controller: &orbit_camera::CameraController) -> Self{
        let device = &wgpu_render_state.device;

        //シェーダーを読み込む
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("polyline_render_resources"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wgpu_3d_polyline_shader.wgsl").into()),
        });

        //########## バーテックスバッファーの作成
        let line_segments: Box<[LineSegment]> = Box::new([]);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyline_render_resources"),
            contents: bytemuck::cast_slice(&line_segments),
            usage: wgpu::BufferUsages::VERTEX,
        });

        //########## カメラ関連 #############
        //カメラ用のユニフォームバッファーの作成
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyline_render_resources"),
            contents: bytemuck::cast_slice(&[camera_controller.camera.uniform]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        //Camera BindGroupのレイアウトを作成する
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("polyline_render_resources"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    //min_binding_size: NonZeroU64::new(16),
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        //Camera用のBind Groupを作成する
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("polyline_render_resources"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        //########## Polylin Material関連 #############

        //polyline_materialを定義
        let polyline_material = LineMaterial{
            color: Vector4::new(0.0, 1.0, 1.0, 1.0),
            depth_bias: -0.0002,
            width: 10.0,
            padding0: 0.0,
            padding1: 0.0,
        };

        //polyline material用のユニフォームバッファーを作成
        let polyline_material_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyline_render_resources"),
            contents: bytemuck::cast_slice(&[polyline_material]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        //Polyline MaterialのBindGroupのレイアウト作成
        let polyline_material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("polyline_render_resources"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        //Polyline MaterialのBindGroupを作成
        let polyline_material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("polyline_render_resources"),
            layout: &polyline_material_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: polyline_material_uniform_buffer.as_entire_binding(),
            }],
        });

        //パイプラインレイアウトを作成する
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("polyline_render_resources"),
            bind_group_layouts: &[&camera_bind_group_layout, &polyline_material_bind_group_layout],
            push_constant_ranges: &[],
        });

        //パイプラインの作成
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("polyline_render_resources"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[LineSegment::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let data = vec![];

        return Self {
            pipeline,
            camera_bind_group,
            camera_uniform_buffer,
            polyline_material_bind_group,
            polyline_material_uniform_buffer,
            vertex_buffer,
            data
        }

    }

    pub fn add_data(&mut self, data: LineSegmentObject){
        self.data.push(data);
    }

    //pub fn get_data(&self) -> Vec<GetObject>{
    //    let mut data: Vec<GetObject> = vec![];
    //    for ob in self.data.iter() {
    //        data.push(ob.get());
    //    }
    //    return data;
    //}

    pub fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, camera_uniform: orbit_camera::CameraUniform) {

        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        )

    }

    pub fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        // Draw our triangle!
        for d in &self.data {
            let num = d.line_segments.len() as u32;
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.polyline_material_bind_group, &[]);
            render_pass.set_vertex_buffer(0, d.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..num);
        }

    }


}