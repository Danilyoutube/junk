use generational_arena::Index;
use gltf::accessor::DataType;
use gltf::mesh::Semantic;
use gltf::Gltf;
use na::Vector3;
use std::collections::HashMap;

use super::context::{BufferTarget, BufferUsage, TypedArrayKind};
use super::renderer::{
  Attribute, AttributeName, Geometry, Material, Mesh, PBRMaterialParams, Primitive, Renderer,
};
use super::shader::AttributeOptions;

pub fn create_gltf_attributes(gltf: &Gltf, renderer: &mut Renderer) -> Vec<Attribute> {
  let mut attributes: Vec<Attribute> = vec![];
  let mut buffer_indices: HashMap<usize, Index> = HashMap::new();

  for accessor_def in gltf.accessors() {
    if accessor_def.sparse().is_some() {
      panic!("sparse is not supported");
    }

    if let Some(view_def) = accessor_def.view() {
      let view_index = view_def.index();
      let blob = gltf.blob.as_ref().unwrap();

      let buffer_handle = if let Some(handle) = buffer_indices.get(&view_index) {
        *handle
      } else {
        let offset = view_def.offset();
        let length = view_def.length();

        let data = &blob[offset..(offset + length)];
        let acc_idx = accessor_def.index();
        let is_index_buffer = gltf
          .meshes()
          .find(|m| {
            m.primitives()
              .find(|p| match p.indices() {
                Some(acc) => acc.index() == acc_idx,
                None => false,
              })
              .is_some()
          })
          .is_some();
        let buffer_target = if is_index_buffer {
          BufferTarget::ElementArrayBuffer
        } else {
          BufferTarget::ArrayBuffer
        };
        let handle = renderer.create_buffer(buffer_target, BufferUsage::StaticDraw, data);
        buffer_indices.insert(view_index, handle);

        handle
      };

      attributes.push(Attribute {
        buffer: buffer_handle,
        options: AttributeOptions {
          component_type: match accessor_def.data_type() {
            DataType::U8 => TypedArrayKind::Uint8,
            DataType::I8 => TypedArrayKind::Int8,
            DataType::I16 => TypedArrayKind::Int16,
            DataType::U16 => TypedArrayKind::Uint16,
            DataType::U32 => TypedArrayKind::Uint32,
            DataType::F32 => TypedArrayKind::Float32,
          },
          item_size: accessor_def.dimensions().multiplicity() as i32,
          normalized: accessor_def.normalized(),
          stride: view_def.stride().unwrap_or(0) as i32,
          offset: accessor_def.offset() as i32,
        },
      });
    } else {
      attributes.push(Attribute {
        buffer: Index::from_raw_parts(0, 0),
        options: AttributeOptions {
          component_type: TypedArrayKind::Float32,
          item_size: 3,
          normalized: false,
          stride: 0,
          offset: 0,
        },
      });
    }
  }

  attributes
}

pub fn create_gltf_meshes(gltf: &Gltf, all_attributes: &[Attribute]) -> Vec<Mesh> {
  let mut meshes: Vec<Mesh> = vec![];

  for mesh_def in gltf.meshes() {
    let mut primitives: Vec<Primitive> = vec![];

    for primitive_def in mesh_def.primitives() {
      let mut attributes: HashMap<AttributeName, Attribute> = HashMap::new();
      let mut count = 0;

      for (semantic_def, accessor_def) in primitive_def.attributes() {
        let attr_name = match semantic_def {
          Semantic::Positions => AttributeName::Position,
          Semantic::Normals => AttributeName::Normal,
          Semantic::TexCoords(value) => match value {
            0 => AttributeName::Uv,
            _ => AttributeName::Unknown(semantic_def.to_string()),
          },
          _ => AttributeName::Unknown(semantic_def.to_string()),
        };
        attributes.insert(attr_name, all_attributes[accessor_def.index()].clone());

        count = accessor_def.count() as i32;
      }

      let indices;

      if let Some(indices_accessor) = primitive_def.indices() {
        indices = Some(all_attributes[indices_accessor.index()].clone());
        count = indices_accessor.count() as i32;
      } else {
        indices = None;
      }

      let geometry = Geometry {
        attributes,
        indices,
        count,
      };

      let material = Material::PBR(PBRMaterialParams {
        color: Vector3::new(0.0, 0.0, 0.0),
      });

      primitives.push(Primitive { geometry, material });
    }

    meshes.push(Mesh {
      primitives,
      name: mesh_def.name().map(|n| n.to_string()),
    });
  }

  meshes
}
