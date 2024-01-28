use obj_loader::load_json_obj;

pub mod app;
pub mod camera;
pub mod gltf_loader;
pub mod input;
pub mod light;
pub mod model;
pub mod model_shader;
pub mod obj_loader;
pub mod renderer;
pub mod shader;
pub mod testing;
pub mod texture;
pub mod transform;
pub mod vertex;
pub mod window;

fn main() {
    pollster::block_on(app::run());
}
