extern crate tdmath;

use tdmath::Vector3;

#[derive(Debug)]
pub struct Vertex {
    pub p: Vector3,
    pub uv: [f32; 2],
}

pub struct Model {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<usize>,
}