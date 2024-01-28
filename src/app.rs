use cgmath::{Matrix4, SquareMatrix};
use gltf::Mesh;

use crate::{
    camera::{self, Camera},
    input,
    model::{AnimatedBone, Bone, BoneTransformsUniform},
    model_shader,
    renderer::Renderer,
    shader::{self, ColorUniform, Render},
    testing::{CameraController, LoadedModel},
    transform::{self, Transform},
    window::{self, WindowSize, WinitWindow},
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    fmt::Binary,
    time::{Duration, Instant},
};

pub trait UpdateCallback {
    fn update(&mut self, delta_time: f32);
}

pub struct App {
    window: Box<WinitWindow>,
    renderer: Box<Renderer>,
    updates: Vec<Box<dyn UpdateCallback>>,
    last_frame_time: Instant,
    fps: u32,
}

impl App {
    pub fn add_update_callback(&mut self, callback: Box<dyn UpdateCallback>) {
        self.updates.push(callback);
    }
}

static mut APP_STATE: Option<App> = None;

async fn new() -> App {
    // init logger for wgpu
    env_logger::init();
    // init input
    input::init();
    // init app
    let window = window::get_new_window(window::WindowInitSettings {
        on_update_fn: window_update,
        on_resize_func: window_resize,
        ..Default::default()
    });
    // init Renderer
    let mut renderer = Renderer::new(
        &window.get_winit_window().expect("Error: Window not found"),
        window.get_window_size(),
    )
    .await;

    App {
        window: Box::new(window),
        renderer: Box::new(renderer),
        last_frame_time: Instant::now(),
        updates: Vec::new(),
        fps: 0,
    }
}

fn window_update() {
    unsafe {
        if let Some(app) = APP_STATE.as_mut() {
            update(app);
            let _ = app.renderer.render();
        }
    }
}

fn window_resize(size: WindowSize) {
    unsafe {
        if let Some(app) = APP_STATE.as_mut() {
            app.renderer.resize(size);
        }
    }
}

pub async fn run() {
    unsafe {
        // create app
        if APP_STATE.is_none() {
            APP_STATE = Some(new().await);
        }
        // init
        if let Some(app) = APP_STATE.as_mut() {
            // add test scene
            app.add_update_callback(Box::new(TestUpdate::new()));
            // start loop
            app.window.run();
        }
    }
}

fn update(app: &mut App) {
    let current_time = Instant::now();
    let delta_time = current_time - app.last_frame_time;

    // Convert delta_time to seconds (f32)
    let delta_time_seconds = delta_time.as_secs_f32();

    let fps = 1.0 / delta_time_seconds;
    // save app fps
    app.fps = fps as u32;
    // call update callbacks
    for callback in &mut app.updates {
        callback.update(delta_time_seconds);
    }
    app.last_frame_time = current_time;
}

pub fn get_renderer<'a>() -> Result<&'a mut Renderer, &'a str> {
    unsafe {
        if let Some(app) = APP_STATE.as_mut() {
            return Ok(app.renderer.as_mut());
        } else {
            return Err("Get Renderer Error");
        }
    }
}
use std::rc::Rc;
struct TestUpdate {
    last_update_time: Instant,
    camera: CameraController,
    anim_index: u32,
    models: Vec<LoadedModel>,
}

impl TestUpdate {
    pub fn new() -> Self {
        // glb/gltf not working
        let model_path = "res/mesh_data.json";
        let anim_path = "res/anim_data.json";
        let camera = CameraController::new();
        let mut models: Vec<LoadedModel> = Vec::new();
        // load model
        models.push(LoadedModel::new(
            model_path,
            anim_path,
            Transform::identity(),
        ));
        Self {
            last_update_time: Instant::now(),
            camera,
            models,
            anim_index: 0,
        }
    }
}

impl UpdateCallback for TestUpdate {
    fn update(&mut self, delta_time: f32) {
        let current_time = Instant::now();
        let elapsed_time = current_time.duration_since(self.last_update_time);
        // every 3 secs print fps
        if elapsed_time >= Duration::from_secs(3) {
            let fps = 1.0 / delta_time;
            println!("Fps: {}", fps);
            self.last_update_time = current_time;
        }
        // update camera
        self.camera.update(delta_time);
        for model in &mut self.models {
            model.update_camera(&self.camera.camera);
            model.update(delta_time);
        }
        if crate::input::is_key_just_released(crate::input::KeyCode::Space) {
            println!("Space just released");
            self.anim_index += 1;
            if self.anim_index == 20 {
                self.anim_index = 0;
            }
        }
        // let new_bones = animate(
        //     &self.mesh_animation.1[0],
        //     &self.mesh_animation.0.meshes[0],
        //     self.anim_index as usize,
        // );
        // model_shader
        //     .bone_transform_buffer
        //     .change_transforms(new_bones, &renderer.queue);
    }
}

use crate::model;
fn animate(
    animation: &model::Animation,
    mesh: &model::Mesh,
    current_anim_index: usize,
) -> BoneTransformsUniform {
    let mut transforms: HashMap<usize, cgmath::Matrix4<f32>> = HashMap::new();
    //let current_anim_index = 0;
    let skel = mesh.skeleton.as_ref().expect("error skeleton");
    for bone in animation.bone_keyframes.values() {
        calculate_bone_transforms(
            bone,
            &mut transforms,
            current_anim_index,
            &animation.bone_keyframes,
            &(skel).bones,
        );
    }
    let mut final_transforms: BoneTransformsUniform = BoneTransformsUniform::new();
    for i in 0..10 {
        let id = Matrix4::identity();
        if let Some(transform) = transforms.get_key_value(&i) {
            let tr = transform.1.clone();
            final_transforms.transforms[i] = tr.into();
        } else {
            // add identity matrix
            final_transforms.transforms[i] = id.into();
        }
    }
    final_transforms
}

fn calculate_bone_transforms(
    bone: &AnimatedBone,
    transforms: &mut HashMap<usize, cgmath::Matrix4<f32>>,
    current_anim_index: usize,
    animation_bones: &HashMap<usize, AnimatedBone>,
    bones: &HashMap<usize, Bone>,
) -> Matrix4<f32> {
    // transform already calculated
    if let Some(tr) = transforms.get_key_value(&(bone.bone_id as usize)) {
        return tr.1.clone();
    }
    if let Some(parent_id) = bone.parent_index {
        // calculate parent transform
        if let Some(parent_transform) = transforms.get_key_value(&parent_id) {
            // parent already calculated
            let transform = crate::transform::Transform::new(
                cgmath::Vector3::from(bone.translation_keys[current_anim_index].translation),
                cgmath::Quaternion::from(bone.rotation_keys[current_anim_index].rotation),
                cgmath::Vector3::from(bone.scale_keys[current_anim_index].scale),
            );
            let mut offset = Matrix4::identity();
            if let Some(bone) = bones.get_key_value(&(bone.bone_id as usize)) {
                offset = Matrix4::from(bone.1.inverse_bind_matrix.clone());
            }
            let mat = parent_transform.1 * transform.matrix() * offset;
            transforms.insert(bone.bone_id as usize, mat);
            return mat;
        } else {
            // calculate parent
            let parent_transform = calculate_bone_transforms(
                animation_bones
                    .get_key_value(&(parent_id as usize))
                    .expect("Parent not found")
                    .1,
                transforms,
                current_anim_index,
                animation_bones,
                bones,
            );

            let transform = crate::transform::Transform::new(
                cgmath::Vector3::from(bone.translation_keys[current_anim_index].translation),
                cgmath::Quaternion::from(bone.rotation_keys[current_anim_index].rotation),
                cgmath::Vector3::from(bone.scale_keys[current_anim_index].scale),
            );
            let mut offset = Matrix4::identity();
            if let Some(bone) = bones.get_key_value(&(bone.bone_id as usize)) {
                offset = Matrix4::from(bone.1.inverse_bind_matrix.clone());
            }
            let mat = parent_transform * transform.matrix() * offset;
            transforms.insert(bone.bone_id as usize, mat);
            return mat;
        }
    } else {
        // root
        // calculate transform
        let transform = crate::transform::Transform::new(
            cgmath::Vector3::from(bone.translation_keys[current_anim_index].translation),
            cgmath::Quaternion::from(bone.rotation_keys[current_anim_index].rotation),
            cgmath::Vector3::from(bone.scale_keys[current_anim_index].scale),
        );
        let mut offset = Matrix4::identity();
        if let Some(bone) = bones.get_key_value(&(bone.bone_id as usize)) {
            offset = Matrix4::from(bone.1.inverse_bind_matrix.clone());
        }
        let mat = transform.matrix() * offset;
        transforms.insert(bone.bone_id as usize, mat);
        return mat;
    }
}
