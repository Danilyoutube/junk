use na::{Matrix4, Vector3, U3};

use anyhow::Result;

use super::material::Material;
use crate::renderer::webgl::context::{Context, DrawMode};
use crate::renderer::webgl::renderer::Camera;
use crate::renderer::webgl::shader::Shader;
use crate::scene::node::Node;

#[derive(Debug)]
pub struct PbrMaterial {
  color: Vector3<f32>,
}

impl PbrMaterial {
  pub fn new(color: Vector3<f32>) -> Self {
    PbrMaterial { color }
  }

  pub fn boxed(self) -> Box<Self> {
    Box::new(self)
  }
}

impl Material for PbrMaterial {
  fn get_tag(&self) -> String {
    String::from("pbr")
  }

  fn create_shader(&self, ctx: &Context) -> Result<Shader> {
    let vert_src = include_str!("./shaders/pbr_vert.glsl");
    let frag_src = include_str!("./shaders/pbr_frag.glsl");

    ctx.create_shader(vert_src, frag_src, &vec![])
  }

  fn set_uniforms(&self, shader: &Shader, node: &Node, camera: &Camera) {
    shader.set_vector3("color", &self.color);
    shader.set_matrix4("projectionMatrix", &camera.projection);
    shader.set_matrix4("viewMatrix", &camera.view);
    shader.set_matrix4("modelMatrix", &node.matrix_world);
    shader.set_matrix3(
      "normalMatrix",
      &node
        .matrix_world
        .try_inverse()
        .unwrap_or_else(|| Matrix4::identity())
        .transpose()
        .fixed_slice::<U3, U3>(0, 0)
        .into(),
    );
  }

  fn draw_mode(&self) -> DrawMode {
    DrawMode::Triangles
  }

  fn cull_face(&self) -> bool {
    true
  }

  fn depth_test(&self) -> bool {
    true
  }
}
