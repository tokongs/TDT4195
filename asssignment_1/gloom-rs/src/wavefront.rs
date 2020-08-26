use std::str::FromStr;
use std::{ffi::CString, path::Path, ptr, str};

use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use crate::model::{Model, Vertex};

extern crate nalgebra_glm as glm;

/**
    Read a Wavefront file and output a Mesh. The file has to have posititions, normals and tex_coords.
    It will also need triangulated faces.
*/
pub fn load(path: &str) -> Model {
    let src = std::fs::read_to_string(Path::new(path))
        .expect(&format!("Failed to read .obj file. {}", path));

    let mut positions = vec![];
    let mut normals = vec![];
    let mut tex_coords = vec![];
    let mut vertices = vec![];
    let mut indices = vec![];

    for line in src.lines() {
        let chars: Vec<char> = line.chars().collect();
        if chars[0] == 'v' && chars[1] == ' ' {
            let position = parse_vec3(chars[2..].iter().collect());
            positions.push(position);
        } else if chars[0] == 'v' && chars[1] == 'n' {
            normals.push(parse_vec3(chars[3..].iter().collect()));
        } else if chars[0] == 'v' && chars[1] == 't' {
            tex_coords.push(parse_tex_coords(chars[3..].iter().collect()));
        } else if chars[0] == 'f' && chars[1] == ' ' {
            parse_face(&mut vertices, &mut indices, &positions, &normals, &tex_coords, chars[2..].iter().collect());
        }
    }

    Model::new(vertices, indices)
}

/**
    Read a Wavefront face line and insert vertices and indices.
    If a given vertx already in the vertices array just insert it's index into indiecs.
*/
fn parse_face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    positions: &Vec<glm::Vec3>,
    normals: &Vec<glm::Vec3>,
    tex_coords: &Vec<glm::Vec2>,
    line: String,
) {
    let components = line.trim().split(" ").collect::<Vec<&str>>();
    for component in components {

        // Vector with position, normal and tex coord.
        let p_n_tc = component.trim().split("/").collect::<Vec<&str>>();
        let position_index = usize::from_str(p_n_tc[0]).unwrap() - 1;
        let normal_index = usize::from_str(p_n_tc[2]).unwrap() - 1;
        let tex_coord_index = usize::from_str(p_n_tc[1]).unwrap() - 1;

        let vertex = Vertex {
            position: positions[position_index].clone(),
            normal: normals[normal_index].clone(),
            tex_coord: tex_coords[tex_coord_index].clone(),
        };

        if vertices.contains(&vertex) {
            indices.push(vertices.iter().position(|&v| v == vertex).unwrap() as u32);
        } else {
            indices.push(vertices.len() as u32);
            vertices.push(vertex);
        }
    }
}

fn parse_vec3(line: String) -> glm::Vec3 {
    let components = line.trim().split(" ").collect::<Vec<&str>>();
    return glm::Vec3::new(
        f32::from_str(components[0]).unwrap(),
        f32::from_str(components[1]).unwrap(),
        f32::from_str(components[2]).unwrap(),
    );
}


fn parse_tex_coords(line: String) -> glm::Vec2 {
    let components = line.trim().split(" ").collect::<Vec<&str>>();

    let u = f32::from_str(components[0]).unwrap();
    let mut v = 0.0;
    if components.len() >= 2 {
        v = f32::from_str(components[1]).unwrap();
    }

    return glm::Vec2::new(u, v);
}
