use cgmath::InnerSpace;
use cgmath::Rotation;
use cgmath::Rotation3;
use cgmath::Zero;
use cgmath::{Matrix4, Quaternion, Rad, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Transform {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Transform {
            position: Vector3::zero(),
            rotation: Quaternion::from_angle_x(Rad(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.position += translation;
    }

    pub fn rotate(&mut self, axis_angle: Vector3<f32>) {
        let angle = axis_angle.magnitude();
        let axis = if angle != 0.0 {
            axis_angle.normalize()
        } else {
            Vector3::unit_x() // Default axis if magnitude is zero
        };
        let quaternion = Quaternion::from_axis_angle(axis, Rad(angle));
        self.rotation = quaternion * self.rotation;
    }

    pub fn scale(&mut self, scale: Vector3<f32>) {
        self.scale.x *= scale.x;
        self.scale.y *= scale.y;
        self.scale.z *= scale.z;
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(Vector3::unit_z())
    }

    pub fn up(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(Vector3::unit_y())
    }

    pub fn rotate_around_axis(&mut self, axis: Vector3<f32>, angle: f32) {
        let quaternion = Quaternion::from_axis_angle(axis, Rad(angle));
        self.rotation = quaternion * self.rotation;
    }
}
