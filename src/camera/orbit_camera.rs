use eframe::egui_wgpu::wgpu;
use nalgebra::{Point3, Vector3, Matrix4, Quaternion, Rotation3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// Camera Uniform 
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4], //align 16 // size 64
    resolution: [f32; 2], //width, height, size 8 + padding 8
    padding0: f32, 
    padding1: f32,
}

impl CameraUniform {
    pub fn new(width : f32, height: f32) -> Self {
        Self {
            view_proj: Matrix4::<f32>::identity().into(),
            resolution: [width, height],
            padding0: 0.0,
            padding1: 0.0,
        }
    }

    //fn update_view_proj(&mut self, camera: &Camera) {
    //    self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    //}

    fn set_resolution(&mut self, width : f32, height : f32){
        self.resolution = [width, height];
    }

}

pub struct Camera{
    position: Point3<f32>,
    quaternion : Quaternion<f32>,
    view_matrix: Matrix4<f32>,
    init_matrix: Matrix4<f32>,
    width : f32,
    height : f32,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    pub uniform: CameraUniform
}

impl Camera{
    pub fn new(
        position: Point3<f32>,
        quaternion: Quaternion<f32>,
        view_matrix: Matrix4<f32>,
        width: f32,
        height: f32,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self{

        Self{
            position,
            quaternion,
            view_matrix,
            init_matrix: view_matrix,
            width,
            height,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            uniform: CameraUniform::new(width, height),
        }
    }


    //update uniform view matrix
    pub fn update_uniform_view_proj(&mut self, new_projection : Matrix4<f32>){
        self.uniform.view_proj = new_projection.into();
    }

    pub fn set_size(&mut self, width: f32, height : f32){
        self.width = width;
        self.height = height;
        self.aspect = width / height;
        self.uniform.set_resolution(width, height);
    }
}


//単純なカメラ実装
pub struct CameraController{
    pub camera: Camera,
    target: Point3<f32>,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    mouse_dx: f32,
    mouse_dy: f32,
    scroll: f32,
    sensitivity: f32,
    mouse_pressed: bool,
    x_angle: f32,
    y_angle: f32,
}

impl CameraController{
    pub fn new(camera: Camera, target:Point3<f32>,  init_x: f32, init_y:f32, sensitivity: f32) -> Self{
        Self{
            camera,
            target, 
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            scroll: 0.0,
            sensitivity,
            mouse_pressed: false,
            x_angle: -init_x.to_radians(), // * 3.141592/180.0,
            y_angle: -init_y.to_radians(), // * 3.141582/180.0,
        }
    }
    pub fn init(&mut self){
        let _x = self.camera.position.x;
        let _y = self.camera.position.y;
        let _z = self.camera.position.z;
        //self.camera.view_matrix = Matrix4::new(1.0, 0.0, 0.0, 0.0,
        //                                        0.0, 1.0, 0.0, 0.0,
        //                                        0.0, 0.0, 1.0, 0.0,
        //                                        -_x, -_y, -_z, 1.0);
        //nalgebraは列優先なので注意 1x4 1列4行
        self.camera.init_matrix = Matrix4::new(1.0, 0.0, 0.0, -_x,
                                               0.0, 1.0, 0.0, -_y,
                                               0.0, 0.0, 1.0, -_z,
                                               0.0, 0.0, 0.0, 1.0);
        self.camera.view_matrix = self.camera.init_matrix.clone();

        //self.camera.init_matrix = Matrix4::new(1.0, 0.0, 0.0, 0.0,
        //                                        0.0, 1.0, 0.0, 0.0,
        //                                        0.0, 0.0, 1.0, 0.0,
        //                                        -_x, -_y, -_z, 1.0);
        println!("view matrix: {:?}", self.camera.view_matrix);
        self.update_camera_matrix(0.0, 0.0);
        self.update_camera();
    }
    pub fn update_camera_matrix(&mut self, dx:f32, dy:f32) {
        let _matrix = self.camera.view_matrix;
        let mut x_dir = 1.0;
        if self.y_angle > 1.5708 || self.y_angle < -1.5708 {
            x_dir = -1.0;
        }   

        self.x_angle += x_dir * dx * self.sensitivity;
        self.y_angle += dy * self.sensitivity;

        println!("x_angle:{:?}, y_angle:{:?}",self.x_angle, self.y_angle);


        let _rotate_y = Matrix4::from_axis_angle(&Vector3::y_axis(), self.x_angle);
        let _rotate_x = Matrix4::from_axis_angle(&Vector3::x_axis(), self.y_angle);
        println!("rotate_y: {:?}", _rotate_y);
        println!("rotate_x: {:?}", _rotate_x);

        self.camera.view_matrix = self.camera.init_matrix *  _rotate_x * _rotate_y;

    }

    fn build_move_view_projection_matrix(&self) -> Matrix4<f32> {
        //let view = cgmath::Matrix4::look_at_rh(self.camera.position, self.target, self.camera.up);
        let view = Matrix4::look_at_rh(&self.camera.position, 
                                                                                &self.target,
                                                                                &self.camera.up);
        //let proj = cgmath::perspective(cgmath::Deg(self.camera.fovy), self.camera.aspect, self.camera.znear, self.camera.zfar);
        let proj = Matrix4::new_perspective(self.camera.aspect, 
                                                                            self.camera.fovy.to_radians(), 
                                                                            self.camera.znear,
                                                                            self.camera.zfar);
        //init PRJ: [[2.4142134, 0.0, 0.0, 0.0], [0.0, 2.4142134, 0.0, 0.0], [0.0, 0.0, -1.002002, -1.0], [0.0, 0.0, -0.2002002, 0.0]]
        //proj * view
        proj * self.camera.view_matrix
    }

    pub fn update_camera(&mut self){


        //self.camera.update_uniform_view_proj(OPENGL_TO_WGPU_MATRIX * self.build_move_view_projection_matrix());
        self.camera.update_uniform_view_proj(self.build_move_view_projection_matrix());

    }

    pub fn get_uniform(&self) -> CameraUniform{
        return self.camera.uniform.clone();
    }

}