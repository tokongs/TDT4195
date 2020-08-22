use std::str::FromStr;
use std::{ffi::CString, path::Path, ptr, str};

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate nalgebra_glm as glm;

#[derive(Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coord: glm::Vec2,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn load(path: &str) -> Mesh {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut positions = Vec::<glm::Vec3>::new();
        let mut normals = Vec::<glm::Vec3>::new();
        let mut tex_coords = Vec::<glm::Vec2>::new();
        let mut vertices = vec![];
        let mut indices = Vec::new();

        for line in reader.lines() {
            let chars: Vec<char> = line.unwrap().chars().collect();
            if chars[0] == 'v' && chars[1] == ' ' {
                let position = Mesh::parseVec3(chars[2..].iter().collect());
                positions.push(position);
            } else if chars[0] == 'v' && chars[1] == 'n' {
                normals.push(Mesh::parseVec3(chars[3..].iter().collect()));
            } else if chars[0] == 'v' && chars[1] == 't' {
                tex_coords.push(Mesh::parseTexCoords(chars[3..].iter().collect()));
            } else if chars[0] == 'f' && chars[1] == ' ' {
                Mesh::parseFace(&mut vertices, &mut indices, &positions, &normals, &tex_coords, chars[2..].iter().collect());
            }
        }

        Mesh { vertices, indices }
    }

    fn parseFace(
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

            indices.push(vertices.len() as u32);
            vertices.push(vertex);
        }
    }

    fn parseVec3(line: String) -> glm::Vec3 {
        let components = line.trim().split(" ").collect::<Vec<&str>>();
        return glm::Vec3::new(
            f32::from_str(components[0]).unwrap(),
            f32::from_str(components[1]).unwrap(),
            f32::from_str(components[2]).unwrap(),
        );
    }


    fn parseTexCoords(line: String) -> glm::Vec2 {
        let components = line.trim().split(" ").collect::<Vec<&str>>();

        let u = f32::from_str(components[0]).unwrap();
        let mut v = 0.0;
        if components.len() >= 2 {
            v = f32::from_str(components[1]).unwrap();
        }

        return glm::Vec2::new(u, v);
    }
}
