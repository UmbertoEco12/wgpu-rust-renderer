use crate::{
    gltf_loader::load_gltf,
    model::{
        AnimatedBone, Animation, Bone, KeyRotation, KeyScale, KeyTranslation, Mesh, Model,
        ModelVertex, Skeleton,
    },
};
use cgmath::num_traits::zero;
use gltf::animation::{self, util::rotations};
use serde_json::Value;
use std::{
    fs::File,
    io::{self, BufRead, Read},
};
// mesh in obj format
pub fn load_obj(path: &str) -> anyhow::Result<Model> {
    let res: Result<
        (
            Vec<tobj::Model>,
            Result<Vec<tobj::Material>, tobj::LoadError>,
        ),
        tobj::LoadError,
    > = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );
    if let Ok(obj) = res {
        //load modes
        let models = obj.0;
        let meshes = models
            .into_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                        tangent: [0.0; 4],
                        bone_ids: [0.0; 4],
                        bone_weights: [0.0; 4],
                    })
                    .collect::<Vec<ModelVertex>>();
                let indices = m.mesh.indices;

                Mesh {
                    name: m.name,
                    vertices,
                    indices,
                    skeleton: None,
                }
            })
            .collect::<Vec<Mesh>>();

        return Ok(Model { meshes });
    }
    return Err(anyhow::anyhow!("Error"));
}

// mesh in custom json format
pub fn load_json_obj(
    model_filepath: &str,
    anims_filepath: &str,
) -> anyhow::Result<(Model, Vec<Animation>)> {
    let file = File::open(model_filepath)?;
    let mut reader = std::io::BufReader::new(file);

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let json: Value = serde_json::from_str(&content)?;
    let mut model: Model = Default::default();
    if let Some(meshes) = json["Meshes"].as_array() {
        for mesh in meshes {
            let mut model_mesh: Mesh = Default::default();
            if let Some(vertices) = mesh["Vertices"].as_array() {
                for vertex in vertices {
                    let mut model_vertex: ModelVertex = Default::default();
                    if let Some(positions) = vertex["Position"].as_array() {
                        let x: f32 = positions[0].as_f64().unwrap_or_default() as f32;
                        let y = positions[1].as_f64().unwrap_or_default() as f32;
                        let z = positions[2].as_f64().unwrap_or_default() as f32;
                        model_vertex.position = [x, y, z];
                    }
                    if let Some(normals) = vertex["Normal"].as_array() {
                        let x: f32 = normals[0].as_f64().unwrap_or_default() as f32;
                        let y = normals[1].as_f64().unwrap_or_default() as f32;
                        let z = normals[2].as_f64().unwrap_or_default() as f32;
                        model_vertex.normal = [x, y, z];
                    }
                    if let Some(tex_coords) = vertex["TexCoords"].as_array() {
                        let x: f32 = tex_coords[0].as_f64().unwrap_or_default() as f32;
                        let y = tex_coords[1].as_f64().unwrap_or_default() as f32;
                        model_vertex.tex_coords = [x, y];
                    }
                    if let Some(bone_id) = vertex["BoneIDs"].as_array() {
                        let x: f32 = bone_id[0].as_f64().unwrap_or_default() as f32;
                        let y = bone_id[1].as_f64().unwrap_or_default() as f32;
                        let z = bone_id[2].as_f64().unwrap_or_default() as f32;
                        let w = bone_id[3].as_f64().unwrap_or_default() as f32;
                        model_vertex.bone_ids = [x, y, z, w];
                    }
                    if let Some(weights) = vertex["Weights"].as_array() {
                        let x: f32 = weights[0].as_f64().unwrap_or_default() as f32;
                        let y = weights[1].as_f64().unwrap_or_default() as f32;
                        let z = weights[2].as_f64().unwrap_or_default() as f32;
                        let w = weights[3].as_f64().unwrap_or_default() as f32;
                        model_vertex.bone_weights = [x, y, z, w];
                    }
                    model_mesh.vertices.push(model_vertex);
                }
            }
            if let Some(indices_array) = mesh["Indices"].as_array() {
                let indices: Vec<u32> = indices_array
                    .iter()
                    .filter_map(|index| index.as_u64().map(|idx| idx as u32))
                    .collect();
                model_mesh.indices = indices;
            }
            model.meshes.push(model_mesh);
        }
    }

    // load skeleton
    if let Some(bones) = json["Skeleton"]["Bones"].as_array() {
        let mut skeleton: Skeleton = Default::default();
        for bone in bones {
            let bone_id: u32 = bone["Id"].as_u64().expect("") as u32;
            let bone_name: String = bone["Name"].as_str().unwrap_or("Unknown").to_string();
            let bone_parent_name = bone["ParentName"].as_str().unwrap_or("Unknown").to_string();
            let bone_parent_id: i32 = bone["ParentId"].as_i64().expect("") as i32;
            // Parse offset matrix
            let offset_matrix: [[f32; 4]; 4] = if let Some(offset) = bone["Offset"].as_array() {
                let mut matrix: [[f32; 4]; 4] = [[0.0; 4]; 4];
                for (row_index, row) in offset.iter().enumerate() {
                    if let Some(row_values) = row.as_array() {
                        for (col_index, value) in row_values.iter().enumerate() {
                            if let Some(float_value) = value.as_f64() {
                                matrix[row_index][col_index] = float_value as f32;
                            }
                        }
                    }
                }
                matrix
            } else {
                // Handle the case where offset is not an array
                // You might want to provide a default matrix or handle the error accordingly
                [[0.0; 4]; 4]
            };
            let mut parent_id = None;
            if bone_parent_id > -1 {
                parent_id = Some(bone_parent_id as usize);
            }
            skeleton.bones.insert(
                bone_id as usize,
                Bone {
                    name: bone_name,
                    id: bone_id,
                    parent_id: parent_id,
                    inverse_bind_matrix: offset_matrix,
                    index: bone_id as usize,
                },
            );
        }
        // order bones
        let mut bones: Vec<Bone> = Vec::new();
        let size = skeleton.bones.values().len();
        for _ in 0..size {
            bones.push(Default::default());
        }
        for bone in skeleton.bones.values() {
            bones[bone.id as usize] = bone.clone();
        }
        skeleton.bones_ordered = bones;
        // update model skeleton
        model.meshes[0].skeleton = Some(skeleton);
    }
    // load animations
    let anim =
        json_anim_loader(anims_filepath, model.meshes[0].skeleton.as_ref().expect("")).expect("");
    Ok((model, anim))
}

pub fn json_anim_loader(filepath: &str, skeleton: &Skeleton) -> anyhow::Result<Vec<Animation>> {
    let file = File::open(filepath)?;
    let mut reader = std::io::BufReader::new(file);

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let json: Value = serde_json::from_str(&content)?;

    let mut anims: Vec<Animation> = Vec::new();

    if let Some(animations) = json["Animations"].as_array() {
        for animation in animations {
            let mut model_animation: Animation = Default::default();
            let name: String = animation["Name"].as_str().unwrap_or("Unknown").to_string();
            model_animation.name = name;
            if let Some(bones) = animation["Bones"].as_array() {
                for bone in bones {
                    let mut animated_bone: AnimatedBone = Default::default();
                    let bone_name = bone["Name"].as_str().unwrap_or("Unknown").to_string();
                    // translation
                    if let Some(keys) = bone["TranslationKeys"].as_array() {
                        for key in keys {
                            if let Some(positions) = key["Position"].as_array() {
                                let x: f32 = positions[0].as_f64().unwrap_or_default() as f32;
                                let y: f32 = positions[1].as_f64().unwrap_or_default() as f32;
                                let z: f32 = positions[2].as_f64().unwrap_or_default() as f32;
                                //model_vertex.position = [x, y, z];
                                let time = key["Time"].as_f64().expect("") as f32;
                                animated_bone.translation_keys.push(KeyTranslation {
                                    timestamp: time,
                                    translation: [x, y, z],
                                })
                            }
                        }
                    }
                    // rotation
                    if let Some(keys) = bone["RotationKeys"].as_array() {
                        for key in keys {
                            if let Some(rotations) = key["Rotation"].as_array() {
                                let x: f32 = rotations[0].as_f64().unwrap_or_default() as f32;
                                let y: f32 = rotations[1].as_f64().unwrap_or_default() as f32;
                                let z: f32 = rotations[2].as_f64().unwrap_or_default() as f32;
                                let w: f32 = rotations[3].as_f64().unwrap_or_default() as f32;
                                //model_vertex.position = [x, y, z];
                                let time = key["Time"].as_f64().expect("") as f32;
                                animated_bone.rotation_keys.push(KeyRotation {
                                    timestamp: time,
                                    rotation: [x, y, z, w],
                                })
                            }
                        }
                    }
                    // scale
                    if let Some(keys) = bone["ScaleKeys"].as_array() {
                        for key in keys {
                            if let Some(scales) = key["Scale"].as_array() {
                                let x: f32 = scales[0].as_f64().unwrap_or_default() as f32;
                                let y: f32 = scales[1].as_f64().unwrap_or_default() as f32;
                                let z: f32 = scales[2].as_f64().unwrap_or_default() as f32;
                                //model_vertex.position = [x, y, z];
                                let time = key["Time"].as_f64().expect("") as f32;
                                animated_bone.scale_keys.push(KeyScale {
                                    timestamp: time,
                                    scale: [x, y, z],
                                })
                            }
                        }
                    }
                    animated_bone.bone_name = bone_name.clone();
                    model_animation
                        .bone_keyframes_name
                        .insert(bone_name.clone(), animated_bone.clone());
                    let mut key = None;
                    for bone in &skeleton.bones {
                        let bone = bone.1;
                        if bone.name == bone_name {
                            key = Some(bone.id as usize);
                            break;
                        }
                    }
                    if let Some(key) = key {
                        model_animation.bone_keyframes.insert(key, animated_bone);
                    } else {
                        println!("No key found for {}", bone_name);
                    }
                }
            }
            anims.push(model_animation);
        }
    }
    // load animations
    Ok(anims)
}
