use std::{num::NonZeroU64, sync::Arc};
use std::any::TypeId;
use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu}, epaint::Rect,
};
use eframe::egui;

use nalgebra::{Point3, Vector3, Matrix4, Quaternion, Rotation3};
//use crate::{line_segment::LineSegment};
//use crate::{polyline_render_resources::{PolylineRenderResources, self}};

pub mod render_object;
use render_object::polyline_object;

pub mod camera;
use camera::orbit_camera;



pub struct Editor3d{
    //objects: Vec<object::Object>,
    //angle: f32,
    //vertices: Box<[Vertex]>
    pub camera_controller: orbit_camera::CameraController,
}

impl Editor3d{
    pub fn new(cc: &eframe::CreationContext ) -> Self {
        // カメラの定義
        let mut camera_controller = orbit_camera::CameraController::new(
            orbit_camera::Camera::new(
                Point3::new(0.0, 0.0, 2.0),
                Quaternion::new(1.0, 0.0, 0.0, 0.0),
                Matrix4::<f32>::identity(),
                300.0,
                300.0,
                Vector3::new(0.0, 1.0, 0.0), //y up
                1.0,
                45.0,
                0.1,
                100.0,
            ),

            Point3::new(0.0, 0.0, 0.0),
            45.0,
            0.0,
            0.01 //camera move speed
        );

        camera_controller.init();

        let wgpu_render_state = cc.wgpu_render_state.as_ref().expect("ERROR");

        // rendererの定義
        let mut triangle_renderer = triangle_render_resources::TriangleRenderResources::new(wgpu_render_state, &camera_controller);
        let mut polyline_renderer = polyline_render_resources::PolylineRenderResources::new(wgpu_render_state, &camera_controller);

        let device = &wgpu_render_state.device;

        //let new_object = object::Object::new(device, Box::new([
        //        Vertex { position: Vector3::new(0.0, 0.5, 0.0), color: Vector3::new(1.0, 0.0, 0.0) },
        //        Vertex { position: Vector3::new(-0.5, -0.5, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
        //        Vertex { position: Vector3::new(0.5, -0.5, 0.0), color: Vector3::new(0.0, 0.0, 1.0) },

        //    ]),
        //);

        //triangle_renderer.add_data(new_object);
        let line_segment_object = line_segment_object::LineSegmentObject::new(device, Box::new([
            LineSegment {point0: Vector3::new(0.0, 0.5, 0.0), point1: Vector3::new(-0.5, -0.5, 0.0)},
            LineSegment {point0: Vector3::new(-0.5, -0.5, 0.0), point1: Vector3::new(0.5, -0.5, 0.0)},
            LineSegment {point0: Vector3::new(0.5, -0.5, 0.0), point1: Vector3::new(0.0, 0.5, 0.0)}
            ]),
        );

        polyline_renderer.add_data(line_segment_object);

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(triangle_renderer);

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(polyline_renderer);
        

        Self {
            //objects:vec![_object]
            camera_controller
        }

    }

    pub fn get_data(&self, frame: &eframe::Frame)-> Vec<object::GetObject>{
        let wgpu_render_state = *frame.wgpu_render_state().as_ref().expect("ERROR");

        let binding = wgpu_render_state.renderer.read();
        let renderer: &TriangleRenderResources = binding.paint_callback_resources.get().unwrap();

        return renderer.get_data();
    }

    pub fn add_object(&self, frame: &eframe::Frame){
        //let binding = frame.wgpu_render_state();
        let wgpu_render_state = *frame.wgpu_render_state().as_ref().expect("ERROR");

        let mut binding = wgpu_render_state.renderer.write();
        let renderer: &mut TriangleRenderResources = binding.paint_callback_resources.get_mut().unwrap();
        let device: &Arc<wgpu::Device> = &wgpu_render_state.device;

        let new_object = object::Object::new(device, Box::new([
                Vertex { position: Vector3::new(0.0, 0.5, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
                Vertex { position: Vector3::new(-0.5, -0.5, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
                Vertex { position: Vector3::new(0.5, -0.5, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
            ]),
        );
        renderer.add_data(new_object);

    }

    pub fn set_size(&mut self, rect: Rect){
        self.camera_controller.camera.set_size(rect.width(), rect.height());
    }

    pub fn custom_paintng(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // Clone locals so we can move them into the paint callback:
        let move_x = response.drag_delta().x * 0.5;
        let move_y = response.drag_delta().y * 0.5;

        self.set_size(rect);
        self.camera_controller.update_camera_matrix(move_x, move_y);
        self.camera_controller.update_camera();

        let uniform_data = self.camera_controller.get_uniform();

        //let cb = egui_wgpu::CallbackFn::new()
        //    .prepare(move |device, queue, _encoder, paint_callback_resources| {
        //        let resources:&mut TriangleRenderResources = paint_callback_resources.get_mut().unwrap();
        //        resources.prepare(device, queue, uniform_data);
        //        Vec::new()
        //    })
        //    .paint(move |_info, render_pass, paint_callback_resources| {
        //        let resources:&TriangleRenderResources = paint_callback_resources.get().unwrap();
        //        //render_pass.set_vertex_buffer(0, vertex_buffer_slice);
        //        resources.paint(render_pass);
        //    });
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                let resources:&mut PolylineRenderResources = paint_callback_resources.get_mut().unwrap();
                resources.prepare(device, queue, uniform_data);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources:&PolylineRenderResources = paint_callback_resources.get().unwrap();
                //render_pass.set_vertex_buffer(0, vertex_buffer_slice);
                resources.paint(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);

    }

}