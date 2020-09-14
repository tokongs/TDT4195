extern crate nalgebra_glm as glm;

use std::time::{SystemTime, UNIX_EPOCH};
use crate::shader::Shader;
use crate::{util};
use core::ptr;

#[derive(Copy, Clone)]
struct Particle {
    position: glm::Vec3,
    velocity: glm::Vec3,
    color: glm::Vec4,
    life: f32,
    velocity_fun: fn(glm::Vec3, timestep: f32) -> glm::Vec3,
    color_fun: fn(glm::Vec4, timestep: f32) -> glm::Vec4,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    position: glm::Vec3,
    radius: f32,
    life_min: f32,
    life_max: f32,
    shader: Shader,
    vao: u32,
    vbos: [u32; 2],
}

impl Particle {
    pub fn new<>(position: glm::Vec3, life: f32, velocity_fun: fn(glm::Vec3, f32) -> glm::Vec3, color_fun: fn(glm::Vec4, f32) -> glm::Vec4) -> Self {
        Self {
            position,
            velocity_fun,
            color_fun,
            color: color_fun(glm::vec4(0.0, 0.0, 0.0, 1.0), -1.0),
            velocity: velocity_fun(glm::vec3(0.0, 0.0, 0.0), -1.0),
            life,
        }
    }

    fn revive_if_dead(mut self, position: glm::Vec3, life: f32) -> Self {
        if self.life <= 0.0 {
            self.position = position;
            self.life = life;
            self.velocity = (self.velocity_fun)(glm::vec3(0.0, 0.0, 0.0), -1.0);
            self.color = (self.color_fun)(glm::vec4(0.0, 0.0, 0.0, 0.0), -1.0);
        }
        self
    }

    pub fn update(mut self, timestep: f32) -> Self {
        self.life -= timestep;
        self.position += &self.velocity * timestep;
        self.velocity = (self.velocity_fun)(self.velocity, timestep);
        self.color = (self.color_fun)(self.color, timestep);
        self
    }
}

impl ParticleSystem {
    pub fn new(size: usize, position: glm::Vec3, radius: f32, life_min: f32, life_max: f32, velocity_fun: fn(glm::Vec3, f32) -> glm::Vec3, color_fun: fn(glm::Vec4, f32) -> glm::Vec4, shader: Shader) -> ParticleSystem {
        let particles: Vec<Particle> = (0..size).into_iter().map(|_| Particle::new(
            position + glm::vec3(radius * util::generate_rng(-1.0, 1.0), radius * util::generate_rng(-1.0, 1.0), radius * util::generate_rng(-1.0, 1.0)),
            util::generate_rng(life_min, life_max),
            velocity_fun,
            color_fun,
        )).collect();

        let mut vao = 0;
        let mut vbos: [u32; 2] =  [0, 0];
        let buffer: Vec<f32> = vec![
            1.0, 1.0, 0.0,
            -1.0, 1.0, 0.0,
            1.0, -1.0, 0.0,
            -1.0, -1.0, 0.0,

        ];
        unsafe {
            // Generate VAO
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            //Generate vertex and index buffersvbo
            gl::GenBuffers(1,  &mut vbos[0]);
            gl::GenBuffers(1,  &mut vbos[1]);

            //Fill vertex buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
            gl::BufferData(gl::ARRAY_BUFFER, util::byte_size_of_array(&buffer), util::pointer_to_array(&buffer), gl::STATIC_DRAW);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 3, ptr::null());

            let particles_buffer = generate_buffer_data(&particles);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
            gl::BufferData(gl::ARRAY_BUFFER, util::byte_size_of_array(&particles_buffer), util::pointer_to_array(&particles_buffer), gl::STREAM_DRAW);

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 7, ptr::null());
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 7, (3 * util::size_of::<f32>()) as *const gl::types::GLvoid);
            gl::BindVertexArray(0);
        }

        unsafe { ParticleSystem { particles, position: position.clone(), radius, life_min, life_max, shader, vao, vbos } }
    }

    pub fn tick(&mut self, timestep: f32) {
        self.particles = self.particles.iter().map(
            |particle| particle.update(timestep).revive_if_dead(
                &self.position + glm::vec3(
                    &self.radius * util::generate_rng(-1.0, 1.0),
                    &self.radius * util::generate_rng(-1.0, 1.0),
                    &self.radius * util::generate_rng(-1.0, 1.0)), util::generate_rng(self.life_min, self.life_max))
        ).collect();

        unsafe {self.updateBuffer()}
    }

    unsafe fn updateBuffer(&mut self) {
        let buffer_data = generate_buffer_data(&self.particles);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbos[1]);
        gl::BufferData(gl::ARRAY_BUFFER, util::byte_size_of_array(&buffer_data), ptr::null(), gl::STREAM_DRAW);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0, util::byte_size_of_array(&buffer_data), util::pointer_to_array(&buffer_data));
    }

    pub unsafe fn render(&self, projection: &glm::Mat4, view: &glm::Mat4){
        self.shader.activate();
        self.shader.set_uniform_mat4("projection", projection);
        self.shader.set_uniform_mat4("view", view);
        gl::BindVertexArray(self.vao);

        // Make sure the vertex position buffer is used repeatedly, while particle poistion and color moves down the buffer
        gl::VertexAttribDivisor(0, 0);
        gl::VertexAttribDivisor(1, 1);
        gl::VertexAttribDivisor(2, 1);

        gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.particles.len() as i32);
    }
}

fn generate_buffer_data(particles: &Vec<Particle>) -> Vec<f32> {
    particles.iter().flat_map(|p| vec!(p.position.x, p.position.y, p.position.z, p.color.x, p.color.y, p.color.z, p.color.w)).collect()
}