use cgmath::num_traits::ops::inv;
use cgmath::{Matrix4, SquareMatrix, Zero};

use crate::app::UpdateCallback;
use crate::camera::{Camera, ModelMatrixUniform};
use crate::model::{self, AnimatedBone, Animation, Bone, BoneTransformsUniform, Model, Skeleton};
use crate::model_shader::{self, ModelShader};
use crate::obj_loader;
use crate::shader::{self, Render};
use crate::transform::{self, Transform};
use cgmath::Vector3;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub struct CameraController {
    pub camera: Camera,
    speed: f32,
}

impl CameraController {
    pub fn new() -> Self {
        let renderer = crate::app::get_renderer().expect("error");
        let camera: Camera =
            Camera::default(renderer.config.width as f32 / renderer.config.height as f32);
        Self { camera, speed: 2.5 }
    }
}

impl UpdateCallback for CameraController {
    fn update(&mut self, delta_time: f32) {
        let renderer = crate::app::get_renderer().expect("error");
        // update camera aspect ratio
        self.camera.aspect_ratio = renderer.config.width as f32 / renderer.config.height as f32;

        // wasd camera
        // if crate::input::is_key_pressed(crate::input::KeyCode::A) {
        //     self.camera
        //         .transform
        //         .translate((-self.speed * delta_time, 0.0, 0.0).into());
        // }
        // if crate::input::is_key_pressed(crate::input::KeyCode::D) {
        //     self.camera
        //         .transform
        //         .translate((self.speed * delta_time, 0.0, 0.0).into());
        // }
        // if crate::input::is_key_pressed(crate::input::KeyCode::S) {
        //     self.camera
        //         .transform
        //         .translate((0.0, 0.0, self.speed * delta_time).into());
        // }
        // if crate::input::is_key_pressed(crate::input::KeyCode::W) {
        //     self.camera
        //         .transform
        //         .translate((0.0, 0.0, -self.speed * delta_time).into());
        // }
        if crate::input::is_key_pressed(crate::input::KeyCode::A) {
            // Rotate the camera around the up axis
            self.camera
                .transform
                .rotate_around_axis(Vector3::unit_y(), -self.speed * delta_time);
        }
        if crate::input::is_key_pressed(crate::input::KeyCode::D) {
            // Rotate the camera around the up axis
            self.camera
                .transform
                .rotate_around_axis(Vector3::unit_y(), self.speed * delta_time);
        }
        if crate::input::is_key_pressed(crate::input::KeyCode::S) {
            // Rotate the camera down around its right axis
            self.camera
                .transform
                .rotate_around_axis(Vector3::unit_x(), -self.speed * delta_time);
        }
        if crate::input::is_key_pressed(crate::input::KeyCode::W) {
            // Rotate the camera up around its right axis
            self.camera
                .transform
                .rotate_around_axis(Vector3::unit_x(), self.speed * delta_time);
        }
    }
}

pub struct LoadedModel {
    transform: Transform,
    model: (Model, Vec<crate::model::Animation>),
    shader: Rc<RefCell<ModelShader>>,
    animation_player: Option<AnimationPlayer>,
    selected_anim_index: usize,
}

impl UpdateCallback for LoadedModel {
    fn update(&mut self, delta_time: f32) {
        if let Some(animation_player) = &mut self.animation_player {
            let skeleton = self.model.0.meshes[0].skeleton.as_ref();
            if let Some(skeleton) = skeleton {
                let new_bones = animation_player.animate_with_ordered_bones(
                    delta_time,
                    &self.model.1[self.selected_anim_index],
                    &skeleton,
                );
                // get renderer
                let renderer = crate::app::get_renderer().expect("error");
                // borrow shader
                let mut shader = (*self.shader).borrow_mut();
                // update shader
                shader
                    .bone_transform_buffer
                    .change_transforms(new_bones, &renderer.queue);
            }
            if crate::input::is_key_just_pressed(crate::input::KeyCode::N) {
                // go next animation
                self.selected_anim_index += 1;
                // if it is bigger set 0
                if self.selected_anim_index >= self.model.1.len() {
                    self.selected_anim_index = 0;
                }
                // reset time
                if let Some(animation_player) = &mut self.animation_player {
                    animation_player.reset();
                }
            }
        }
    }
}

impl LoadedModel {
    pub fn new(model_path: &str, anim_path: &str, transform: Transform) -> Self {
        let renderer = crate::app::get_renderer().expect("error");
        // load model gltf (not working)
        //let model: (Model, Vec<crate::model::Animation>) =
        //    crate::gltf_loader::load_gltf(path).expect("Error mesh not found");

        let model = obj_loader::load_json_obj(model_path, anim_path).expect("model error");
        for anim in &model.1 {
            println!("anim {}", anim.name);
        }
        // load shader
        let shader: Rc<RefCell<ModelShader>> = Rc::new(RefCell::new(
            model_shader::ModelShader::new("src/model_shader.wgsl", &renderer, &model.0),
        ));

        renderer.add_shader(Rc::clone(&shader) as Rc<RefCell<dyn Render>>);

        let mut animation_player = None;
        if model.1.len() > 0 {
            animation_player = Some(AnimationPlayer::new());
        }
        LoadedModel {
            transform,
            model,
            shader: Rc::clone(&shader),
            animation_player,
            selected_anim_index: 0,
        }
    }

    pub fn translate(&mut self, translation: cgmath::Vector3<f32>) {
        // get renderer
        let renderer = crate::app::get_renderer().expect("error");
        // borrow shader
        let mut shader = (*self.shader).borrow_mut();
        // transform
        self.transform.translate(translation);
        // set uniform
        shader.model_buffer.update_matrix(
            ModelMatrixUniform {
                matrix: self.transform.matrix().into(),
            },
            &renderer.queue,
        )
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        // get renderer
        let renderer = crate::app::get_renderer().expect("error");
        // borrow shader
        let mut shader = (*self.shader).borrow_mut();
        // update camera
        shader.camera_buffer.update_camera(&camera, &renderer.queue);
    }
}

pub struct AnimationPlayer {
    current_time: f32,
    current_anim_index: usize,
    frame_time: f32,
}

impl AnimationPlayer {
    pub fn new() -> Self {
        AnimationPlayer {
            current_time: 0.0,
            current_anim_index: 0,
            frame_time: 1.0 / 24.0, // 24 fps
        }
    }
    pub fn reset(&mut self) {
        self.current_anim_index = 0;
        self.current_time = 0.0;
    }

    pub fn get_bone_model_matrix(&mut self, bone: &AnimatedBone) -> Matrix4<f32> {
        if self.current_anim_index == bone.translation_keys.len() {
            self.current_anim_index = 0;
        }
        // get bone transform
        crate::transform::Transform::new(
            cgmath::Vector3::from(bone.translation_keys[self.current_anim_index].translation),
            cgmath::Quaternion::from(bone.rotation_keys[self.current_anim_index].rotation),
            cgmath::Vector3::from(bone.scale_keys[self.current_anim_index].scale),
        )
        .matrix()
    }

    pub fn animate_with_ordered_bones(
        &mut self,
        delta_time: f32,
        animation: &Animation,
        skeleton: &Skeleton,
    ) -> BoneTransformsUniform {
        let mut bone_transforms: Vec<cgmath::Matrix4<f32>> = Vec::new();
        for _ in 0..skeleton.bones_ordered.len() {
            bone_transforms.push(Matrix4::zero());
        }
        let mut final_transforms: BoneTransformsUniform = BoneTransformsUniform::new();
        for bone in &skeleton.bones_ordered {
            let mut transform = Matrix4::identity();
            if let Some(anim_bone) = animation.bone_keyframes_name.get_key_value(&(bone.name)) {
                //animation.bone_keyframes.get_key_value(&(bone.id as usize)) {
                transform = self.get_bone_model_matrix(&anim_bone.1);
                if let Some(parent) = bone.parent_id {
                    transform = bone_transforms[parent] * transform;
                }
                bone_transforms[bone.id as usize] = transform;
            }
            let mut inverse_bind_matrix = Matrix4::from(bone.inverse_bind_matrix);
            final_transforms.transforms[bone.id as usize] =
                (transform * inverse_bind_matrix).into();
        }
        self.update_time(delta_time);
        final_transforms
    }

    fn update_time(&mut self, delta_time: f32) {
        // update time
        self.current_time += delta_time;
        if self.current_time > self.frame_time {
            self.current_anim_index += 1;
            self.current_time = 0.0;
        }
    }
    pub fn animate(
        &mut self,
        delta_time: f32,
        animation: &Animation,
        skeleton: &Skeleton,
    ) -> BoneTransformsUniform {
        let mut bone_transforms: HashMap<usize, cgmath::Matrix4<f32>> = HashMap::new();
        let mut final_transforms: BoneTransformsUniform = BoneTransformsUniform::new();
        for bone in skeleton.bones.values() {
            let transform =
                self.calculate_transform(bone, animation, &mut bone_transforms, skeleton);
            let mut inv_matrix = Matrix4::from(bone.inverse_bind_matrix);
            //inv_matrix.transpose_self();
            final_transforms.transforms[bone.id as usize] = (transform * inv_matrix).into();
        }
        self.update_time(delta_time);
        final_transforms
    }

    fn calculate_transform(
        &mut self,
        bone: &Bone,
        animation: &Animation,
        bone_transforms: &mut HashMap<usize, cgmath::Matrix4<f32>>,
        skeleton: &Skeleton,
    ) -> Matrix4<f32> {
        // already calculated
        if let Some(transform) = bone_transforms.get_key_value(&(bone.id as usize)) {
            return transform.1.clone();
        }
        let mut tranform: Matrix4<f32> = Matrix4::identity();
        // calculate
        if let Some(anim_bone) = animation.bone_keyframes.get_key_value(&(bone.id as usize)) {
            let anim_bone = anim_bone.1;
            // set transform to this
            tranform = self.get_bone_model_matrix(anim_bone);
            // if it has a parent
            if let Some(parent_id) = bone.parent_id {
                // get the parent transform
                let parent = skeleton
                    .bones
                    .get_key_value(&parent_id)
                    .expect("Parent not found")
                    .1;
                // get parent transform
                let parent_transform =
                    self.calculate_transform(parent, animation, bone_transforms, skeleton);
                tranform = parent_transform * tranform;
            }
            bone_transforms.insert(bone.id as usize, tranform);
        }
        tranform
    }
}
