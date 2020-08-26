use std::path::Path;
use crate::model::{Model, Vertex};

extern crate nalgebra_glm as glm;

pub fn load(path: &str) -> Model {
    let src = std::fs::read_to_string(Path::new(path))
        .expect(&format!("Failed to read .myf file. {}", path));

    let mut lines = src.lines();
    let vertex_line = lines.next().unwrap();
    let index_line = lines.next().unwrap();
    let vertex_floats = vertex_line.strip_prefix(" ")
        .unwrap()
        .split(" ")
        .map(|s| s.parse::<f32>().expect("Failed to parse f32"))
        .collect::<Vec<f32>>();
    let mut vertices = vec![];

    for i in (0..vertex_floats.len()).step_by(8) {
        let vertex = Vertex {
            position: glm::Vec3::new(vertex_floats[0 + i], vertex_floats[i + 1], vertex_floats[i + 2]),
            normal: glm::Vec3::new(vertex_floats[i + 3], vertex_floats[i + 4], vertex_floats[i + 5]),
            tex_coord: glm::Vec2::new(vertex_floats[i + 6], vertex_floats[i + 7])
        };
        vertices.push(vertex);
    }

    Model::new(vertices, index_line.strip_prefix(" ")
        .unwrap().split(" ")
        .map(|s| s.parse::<u32>().expect("Failed to parse u32"))
        .collect::<Vec<u32>>())
}