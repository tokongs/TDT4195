use crate::util;
use std::ptr;
use crate::shader::Shader;

#[derive(Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coord: glm::Vec2,
}


#[derive(Clone)]
pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vao: u32,
    model_matrix: glm::Mat4,
}

impl Model {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Model {
        return Model {
            vertices,
            indices,
            vao: 0,
            model_matrix: glm::Mat4::new(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0),
        };
    }

    pub unsafe fn init(&mut self) -> &mut Model {
        let mut vbo = 0;
        let mut ibo = 0;

        // Generate VAO
        gl::GenVertexArrays(1, &mut self.vao);
        gl::BindVertexArray(self.vao);

        //Generate vertex and index buffers
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);

        let buffer_data = self.vertices.iter().flat_map(
            |vertex| vec![
                vertex.position.x, vertex.position.y, vertex.position.z,
                vertex.normal.x, vertex.normal.y, vertex.normal.z,
                vertex.tex_coord.x, vertex.tex_coord.y
            ]).collect::<Vec<f32>>();

        //Fill vertex buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, util::byte_size_of_array(&buffer_data), util::pointer_to_array(&buffer_data), gl::STATIC_DRAW);

        // Fill index buffer
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, util::byte_size_of_array(&self.indices), util::pointer_to_array(&self.indices), gl::STATIC_DRAW);

        // Configure vertex attribute layout
        // Position vec3
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 8, ptr::null());

        // normal vec3
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 8, (3 * util::size_of::<f32>()) as *const gl::types::GLvoid);

        // tex coord vec2
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 8, (5 * util::size_of::<f32>()) as *const gl::types::GLvoid);
        gl::BindVertexArray(0);

        self
    }

    // Render the mesh with the given shader. Assumes the shader has a model_matrix uniform
    pub unsafe fn render(&self, shader: &Shader) {
        gl::BindVertexArray(self.vao);

        shader.activate();
        // Update the model matrix
        shader.set_uniform_mat4("model_matrix", &self.model_matrix);

        gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
    }

    pub unsafe fn attach_texture(&mut self, path: &str){
        let img = image::open(path).expect("Failed to load image {}", &path);
        let mut texture = 0;
        gl::GenTextures(1, &texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(GL_TEXTURE_2D, 0, GL_RGB, )
    }

    pub fn rotate(&mut self, axis: glm::Vec3, angle_in_rad: f32) -> &mut Model {
        self.model_matrix = glm::rotate(&self.model_matrix, angle_in_rad, &axis);
        self
    }

    pub fn translate(&mut self, translation: glm::Vec3) -> &mut Model {
        self.model_matrix = glm::translate(&self.model_matrix, &translation);
        self
    }
}