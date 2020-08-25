use std::fs::File;
use std::io::Write;

use crate::mesh::{Mesh, Vertex};

pub fn write(path: &str, mesh: Mesh) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    let vertex_data = mesh.vertices.iter().flat_map(
        |vertex| vec![
            vertex.position.x, vertex.position.y, vertex.position.z,
            vertex.normal.x, vertex.normal.y, vertex.normal.z,
            vertex.tex_coord.x, vertex.tex_coord.y
        ]).collect::<Vec<f32>>();

    for value in vertex_data {
        write!(file, " {}", value);
    }
    write!(file, "\n");
    for index in mesh.indices {
        write!(file, " {}", index);
    }
    write!(file, "")
}