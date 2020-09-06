extern crate nalgebra_glm as glm;

use std::time::{SystemTime, UNIX_EPOCH};
use crate::shader::Shader;
use crate::{byte_size_of_array, util};

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
    vao: u32
}

impl Particle {
    pub fn new<>(position: glm::Vec3, life: f32, velocity_fun: fn(glm::Vec3, f32) -> glm::Vec3, color_fun: fn(glm::Vec4, f32) -> glm::Vec4) -> Self {
        Self {
            position,
            velocity_fun,
            color_fun,
            color: color_fun(glm::vec4(0.0, 0.0, 0.0, 0.0), -1.0),
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
        self.position = &self.velocity * timestep;
        self.velocity = (self.velocity_fun)(self.velocity, timestep);
        self.color = (self.color_fun)(self.color, timestep);
        self
    }
}

impl ParticleSystem {
    pub fn new(size: usize, position: glm::Vec3, radius: f32, life_min: f32, life_max: f32, velocity_fun: fn(glm::Vec3, f32) -> glm::Vec3, color_fun: fn(glm::Vec4, f32) -> glm::Vec4, shader: Shader) -> ParticleSystem {
        let particles = (0..size).into_iter().map(|_| Particle::new(
            position + glm::vec3(radius * generate_rng(-1.0, 1.0), radius * generate_rng(-1.0, 1.0), radius * generate_rng(-1.0, 1.0)),
            generate_rng(life_min, life_max),
            velocity_fun,
            color_fun,
        )).collect();

        let mut vao = 0;
        let buffer = vec![
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            -0.5, 0.5, 0.0,
            0.5, 0.5, 0.0,
        ];
        unsafe {
            // Generate VAO
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            //Generate vertex and index buffers
            gl::GenBuffers(1, &mut vbo);

            //Fill vertex buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&buffer), util::pointer_to_array(vertices), gl::STATIC_DRAW);

            // Configure vertex attribute layout
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<f32>() * 7, ptr::null());

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, size_of::<f32>() * 7, (3 * size_of::<f32>()) as *const gl::types::GLvoid);
            gl::BindVertexArray(0);
        }

        unsafe { ParticleSystem { particles, position: position.clone(), radius, life_min, life_max, shader } }
    }

    pub fn tick(&mut self, timestep: f32) {
        self.particles = self.particles.iter().map(
            |particle| particle.update(timestep).revive_if_dead(
                &self.position + glm::vec3(
                    &self.radius * generate_rng(-1.0, 1.0),
                    &self.radius * generate_rng(-1.0, 1.0),
                    &self.radius * generate_rng(-1.0, 1.0)), generate_rng(self.life_min, self.life_max))
        ).collect();
    }

    pub fn render(){

    }
}

fn generate_rng(min: f32, max: f32) -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f32;

    ((nanos % 255.0) / 255.0) * (max - min) + min
}