use cgmath::SquareMatrix;

use crate::{
    camera,
    model::{
        AnimatedBone, Animation, Bone, KeyRotation, KeyScale, KeyTranslation, Mesh, Model,
        ModelVertex, Skeleton,
    },
};
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
};

pub fn process_node(node: &gltf::Node) {
    println!("processing node: {:#?}", node.name());
}

pub fn process_mesh(mesh: &gltf::Mesh, buffer_data: &Vec<Vec<u8>>, node: &gltf::Node) -> Model {
    println!("processing mesh: {:#?}", mesh.name());
    let primitives = mesh.primitives();
    //let skin = mesh.

    println!("primitives count {}", primitives.len());
    let mut meshes = Vec::new();
    primitives.for_each(|primitive| {
        let mut vertices: Vec<ModelVertex> = Vec::new();
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords_0 = Vec::new();
        let mut tangents = Vec::new();
        let mut joints = Vec::new();
        let mut weights = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_count = 0;
        let mut skeleton: Option<Skeleton> = None;
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
        // get other values
        // read positions
        if let Some(position_attribute) = reader.read_positions() {
            position_attribute.for_each(|position| {
                positions.push(position);
            })
        }
        vertex_count = positions.len();
        // read normals
        if let Some(normal_attribute) = reader.read_normals() {
            normal_attribute.for_each(|normal| {
                normals.push(normal);
            })
        }
        // read tex_coords
        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            tex_coord_attribute.for_each(|tex_coord| {
                tex_coords_0.push(tex_coord);
            })
        }
        // read tangents
        if let Some(tangent_attribute) = reader.read_tangents() {
            tangent_attribute.for_each(|tangent| {
                tangents.push(tangent);
            })
        }
        // if it has a skeleton
        if let Some(skin) = node.skin() {
            skeleton = Some(process_skin(&skin, buffer_data));
            // get bones ids
            if let Some(joint_attribute) = reader.read_joints(0).map(|v| v.into_u16()) {
                // Iterate over joint attributes
                joint_attribute.for_each(|joint| {
                    let mut f_array = [0.0, 0.0, 0.0, 0.0];
                    let mut index = 0;
                    for j in joint {
                        let mut joint_id: usize = j as usize;
                        // find new bone id from gltf joint index
                        for bone in &skeleton.as_ref().expect("").bones_ordered {
                            if bone.index == j as usize {
                                joint_id = bone.id as usize;
                                break;
                            }
                        }
                        f_array[index] = joint_id as f32;
                        index += 1;
                    }
                    joints.push(f_array);
                });
            }
            // get bones weights
            if let Some(weight_attribute) = reader.read_weights(0).map(|v| v.into_f32()) {
                // Iterate over joint attributes
                weight_attribute.for_each(|weight| {
                    weights.push(weight);
                });
            }
            // add vertices
        }
        for i in 0..vertex_count {
            let pos = positions[i];
            let tex_coords = tex_coords_0[i];
            let normals = normals[i];
            let mut tangent = [0.0, 0.0, 0.0, 0.0];
            let mut joint = [0.0, 0.0, 0.0, 0.0];
            let mut weight = [0.0, 0.0, 0.0, 0.0];
            if tangents.len() > 0 {
                tangent = tangents[i];
            }
            if joints.len() > 0 {
                joint = joints[i];
            }
            if weights.len() > 0 {
                weight = weights[i];
            }
            vertices.push(ModelVertex {
                position: pos,
                tex_coords: tex_coords,
                normal: normals,
                tangent: tangent,
                bone_ids: joint,
                bone_weights: weight,
            })
        }
        if let Some(indices_raw) = reader.read_indices() {
            indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
        }
        meshes.push(Mesh {
            name: mesh.name().unwrap_or_else(|| "unnamed mesh").to_string(),
            vertices: vertices,
            indices: indices,
            skeleton: skeleton,
        })
    });
    return Model { meshes };
}

pub fn process_skin(skin: &gltf::Skin, buffer_data: &Vec<Vec<u8>>) -> Skeleton {
    // let skel = skin.skeleton();
    let joints = skin.joints();
    let mut bones = HashMap::new();
    let name: String = skin.name().unwrap_or_else(|| "skeleton").to_string();
    let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
    let mut matrices = Vec::new();
    if let Some(inverse_matrices_attribute) = reader.read_inverse_bind_matrices() {
        inverse_matrices_attribute.for_each(|mat| {
            if cgmath::Matrix4::from(mat.clone()) == cgmath::Matrix4::identity() {
                println!("Identity");
            }
            let mut m = cgmath::Matrix4::from(mat);
            // let mut m = m.invert().expect("");
            // m.transpose_self();
            let mat = m.into();
            matrices.push(mat);
        })
    }
    let mut count = 0;
    let mut children: HashMap<usize, usize> = HashMap::new();
    joints.for_each(|joint| {
        //println!("processing joint: {:#?}, index {}", &joint.name(), &joint.index());
        for child in joint.children() {
            // k: bone index, v: parent index
            children.insert(child.index(), joint.index());
        }
        let parent_id = children.get_key_value(&joint.index());
        let mut parent_index = None;
        if let Some(parent_id) = parent_id {
            parent_index = Some(parent_id.1.clone());
        }

        bones.insert(
            joint.index(),
            Bone {
                id: joint.index() as u32,
                name: joint.name().unwrap_or_else(|| "unnamed bone").to_string(),
                inverse_bind_matrix: matrices[count],
                parent_id: parent_index,
                index: joint.index(),
            },
        );
        count += 1;
    });
    // reset bone parenting
    for bone in bones.values_mut() {
        if let Some(parent_id) = children.get(&(bone.id as usize)) {
            bone.parent_id = Some(*parent_id);
        } else {
            bone.parent_id = None;
        }
    }
    // Order the bones
    let mut ordered_bones: Vec<Bone> = Vec::new();

    // insert roots
    for bone in bones.values() {
        if bone.parent_id.is_none() {
            ordered_bones.push(bone.clone());
        }
    }
    // insert all other nodes
    while ordered_bones.len() < bones.values().len() {
        for bone in bones.values() {
            // check if bone is present
            let mut is_present = false;
            for ord_bone in &ordered_bones {
                if ord_bone.id == bone.id {
                    is_present = true;
                    break;
                }
            }
            if is_present {
                continue;
            }
            if let Some(parent_id) = bone.parent_id {
                // check if parent is present
                for ord_bone in &ordered_bones {
                    if ord_bone.id == parent_id as u32 {
                        is_present = true;
                        break;
                    }
                }
                if is_present {
                    // add node
                    ordered_bones.push(bone.clone());
                    continue;
                }
            }
        }
    }
    let mut index = 0;
    // Now, ordered_bones contains bones in the desired order
    for bone in &mut ordered_bones {
        bone.id = index;
        index += 1;
    }
    Skeleton {
        name: name,
        bones: bones,
        bones_ordered: ordered_bones,
    }
}

pub fn process_animations(
    animation: &gltf::Animation,
    buffer_data: &Vec<Vec<u8>>,
    skeleton: &Skeleton,
) -> Animation {
    println!("processing animation {:#?}", animation.name());
    let mut anim_bones: HashMap<usize, AnimatedBone> = HashMap::new();
    let mut children: HashMap<usize, usize> = HashMap::new();
    for channel in animation.channels() {
        let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
        let bone_id = channel.target().node().index();

        let bone_name = channel
            .target()
            .node()
            .name()
            .unwrap_or_else(|| "")
            .to_string();
        let mut timestamps: Vec<f32> = Vec::new();

        //println!("bone id: {:#?} bone name: {:#?}", bone_id, &bone_name);
        if anim_bones.contains_key(&bone_id) == false {
            for child in channel.target().node().children() {
                // k: bone index, v: parent index
                children.insert(child.index(), bone_id);
            }
            let parent_id = children.get_key_value(&bone_id);
            let mut parent_index = None;
            if let Some(parent_id) = parent_id {
                parent_index = Some(parent_id.1.clone());
            }
            anim_bones.insert(
                bone_id,
                AnimatedBone {
                    bone_id: bone_id as u32,
                    bone_name,
                    parent_index,
                    translation_keys: Vec::new(),
                    rotation_keys: Vec::new(),
                    scale_keys: Vec::new(),
                },
            );
        }

        if let Some(inputs) = reader.read_inputs() {
            match inputs {
                gltf::accessor::Iter::Standard(iter) => {
                    let times: Vec<f32> = iter.collect();
                    for time in times {
                        timestamps.push(time);
                    }
                }
                _ => {
                    println!("Iter not supported")
                }
            }
        }
        let mut index = 0;
        if let Some(output) = reader.read_outputs() {
            match output {
                // add translation keyframes
                gltf::animation::util::ReadOutputs::Translations(translations) => {
                    translations.for_each(|translation| {
                        if let Some(animated_bone) = anim_bones.get_mut(&bone_id) {
                            animated_bone.translation_keys.push(KeyTranslation {
                                translation: translation,
                                timestamp: timestamps[index],
                            });
                            index += 1;
                        }
                    });
                }
                // add rotation keyframes
                gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                    rotations.into_f32().for_each(|rotation| {
                        if let Some(animated_bone) = anim_bones.get_mut(&bone_id) {
                            animated_bone.rotation_keys.push(KeyRotation {
                                rotation: rotation,
                                timestamp: timestamps[index],
                            });
                            index += 1;
                        }
                    });
                }
                // add scale keyframes
                gltf::animation::util::ReadOutputs::Scales(scales) => {
                    scales.for_each(|scale| {
                        if let Some(animated_bone) = anim_bones.get_mut(&bone_id) {
                            animated_bone.scale_keys.push(KeyScale {
                                scale,
                                timestamp: timestamps[index],
                            });
                            index += 1;
                        }
                    });
                }
                _ => {}
            }
        }
    }
    // calculate parents again
    for bone in anim_bones.values_mut() {
        if let Some(parent_id) = children.get(&(bone.bone_id as usize)) {
            bone.parent_index = Some(*parent_id);
        } else {
            bone.parent_index = None;
        }
    }
    let mut ordered_hash_map: HashMap<usize, AnimatedBone> = HashMap::new();
    for bone in &skeleton.bones_ordered {
        // find anim bone
        for anim_bone in anim_bones.values_mut() {
            if anim_bone.bone_name == bone.name {
                // add
                anim_bone.bone_id = bone.id;
                ordered_hash_map.insert(bone.id as usize, anim_bone.clone());
                break;
            }
        }
    }
    Animation {
        name: animation
            .name()
            .unwrap_or_else(|| "Unnamed animation")
            .to_string(),
        bone_keyframes: ordered_hash_map,
        bone_keyframes_name: HashMap::new(),
    }
}

// struct DataUri<'a> {
//     mime_type: &'a str,
//     base64: bool,
//     data: &'a str,
// }

// fn split_once(input: &str, delimiter: char) -> Option<(&str, &str)> {
//     let mut iter = input.splitn(2, delimiter);
//     Some((iter.next()?, iter.next()?))
// }

// impl<'a> DataUri<'a> {
//     fn parse(uri: &'a str) -> Result<DataUri<'a>, ()> {
//         let uri = uri.strip_prefix("data:").ok_or(())?;
//         let (mime_type, data) = split_once(uri, ',').ok_or(())?;

//         let (mime_type, base64) = match mime_type.strip_suffix(";base64") {
//             Some(mime_type) => (mime_type, true),
//             None => (mime_type, false),
//         };

//         Ok(DataUri {
//             mime_type,
//             base64,
//             data,
//         })
//     }

//     fn decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
//         if self.base64 {
//             base64::Engine::decode(&base64::engine::general_purpose::STANDARD, self.data)
//         } else {
//             Ok(self.data.as_bytes().to_owned())
//         }
//     }
// }

pub fn load_gltf(path: &str) -> anyhow::Result<(Model, Vec<Animation>)> {
    return Err(anyhow::anyhow!("Not working"));
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let gltf = gltf::Gltf::from_reader(reader)?;

    // Assuming your GLTF file is in a directory, get the base path
    let base_path = std::path::Path::new(path).parent().unwrap();

    let mut buffer_data: Vec<Vec<u8>> = Vec::new();
    const VALID_MIME_TYPES: &[&str] = &["application/octet-stream", "application/gltf-buffer"];
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                if let Some(blob) = gltf.blob.as_deref() {
                    buffer_data.push(blob.into());
                } else {
                    println!("error missing blob");
                }
            }
            gltf::buffer::Source::Uri(uri) => {
                return Err(anyhow::anyhow!("Unsupported buffer format"));
                // let uri = percent_encoding::percent_decode_str(uri)
                //     .decode_utf8()
                //     .unwrap();
                // let uri = uri.as_ref();
                // let buffer_bytes = match DataUri::parse(uri) {
                //     Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => {
                //         data_uri.decode()?
                //     }
                //     _ => {
                //         return Err(anyhow::anyhow!("Unsupported buffer format"));
                //     }
                // };
                // buffer_data.push(buffer_bytes);
            }
        }
    }
    let mut animations = Vec::new();

    // Now you have the buffer data in the buffer_data vector
    for node in gltf.nodes() {
        match node.mesh() {
            Some(mesh) => {
                println!("mesh name: {:#?}", mesh.name());
                //let skin = node.skin();
                if let Some(skin) = node.skin() {
                    println!("mesh has skin {:#?}", skin.name());
                }
                let model = process_mesh(&mesh, &buffer_data, &node);
                // process animations
                for anim in gltf.animations() {
                    animations.push(process_animations(
                        &anim,
                        &buffer_data,
                        &model.meshes[0].skeleton.as_ref().expect(""),
                    ));
                }
                return Ok((model, animations));
            }
            _ => {}
        }
    }
    Err(anyhow::anyhow!("No mesh was found"))
}
