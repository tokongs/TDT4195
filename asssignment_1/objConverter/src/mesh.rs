
use std::ptr;
extern crate nalgebra_glm as glm;


#[derive(Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coord: glm::Vec2,
}

#[derive(Clone)]
pub struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        return Mesh {
            vertices,
            indices
        }
    }
}