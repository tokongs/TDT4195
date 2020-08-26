mod my_format;

extern crate nalgebra_glm as glm;

use gl::types::*;
use std::{
    mem,
    ptr,
    str,
    os::raw::c_void,
};
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod wavefront;
mod model;


use glutin::event::{Event, WindowEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;
use glm::length;
use std::ffi::CString;
use crate::model::Model;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// Helper functions to make interacting with OpenGL a little bit prettier. You will need these!
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}


unsafe fn setup_vao(vertices: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;

    // Generate VAO
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    //Generate vertex and index buffers
    gl::GenBuffers(1, &mut vbo);
    gl::GenBuffers(1, &mut ibo);

    //Fill vertex buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(vertices), util::pointer_to_array(vertices), gl::STATIC_DRAW);

    // Fill index buffer
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(indices), util::pointer_to_array(indices), gl::STATIC_DRAW);

    // Configure vertex attribute layout
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<f32>() * 3, ptr::null());
    gl::BindVertexArray(0);
    return vao;
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Send a copy of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the renderin thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

        }

        /** Data for task 1c
                let vertices: Vec<f32> = vec![
                    0.0, 0.0, 0.0, // center 0
                    -1.0, 1.0, 0.0, // top left 1
                    0.0, 1.0, 0.0, // top center 2
                    1.0, 1.0, 0.0, //top right 3
                    1.0, 0.0, 0.0, // right center 4
                    1.0, -1.0, 0.0, // bottom right 5
                    0.0, -1.0, 0.0, // bottom center 6
                    -1.0, -1.0, 0.0, // bottom left 7
                    -1.0, 0.0, 0.0, // left center 8
                ];
                let indices: Vec<u32> = vec![
                    0, 2, 1,
                    0, 3, 2,
                    0, 4, 3,
                    0, 5, 4,
                    0, 6, 5,
                    0, 7, 6,
                    0, 8, 7,
                    0, 1, 8
                ];
                **/


        // == // Set up your VAO here
        //let vao = unsafe { setup_vao(&vertices, &indices) };

        // Load meshes and textures.
        let mut ball: Model;
        let mut torus: Model;
        let mut cube: Model;

        // Load models
        unsafe {
            ball = my_format::load("resources/ball.myf");
            ball.attach_texture("resources/ball.png");
            ball.init();

            cube = wavefront::load("resources/cube.obj");
            cube.attach_texture("resources/cube.png");
            cube.init();

            torus = my_format::load("resources/torus.myf");
            torus.attach_texture("resources/torus.png");
            torus.init();
        }

        //Move models to right location
        torus.translate(glm::Vec3::new(-2.0, 0.0, 0.0));
        cube.translate(glm::Vec3::new(2.0, 0.0, 0.0));

        // Basic usage of shader helper
        // The code below returns a shader object, which contains the field .program_id
        // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path)
        let shader_program: shader::Shader;
        unsafe {
            shader_program = shader::ShaderBuilder::new().attach_file("shaders/simple.vert").attach_file("shaders/simple.frag").link();

            // Set the view and projection matrices
            shader_program.activate();

            // set the view matrix
            shader_program.set_uniform_mat4("view_matrix",
                                            &glm::look_at(
                                                &glm::Vec3::new(0.0, 4.0, 4.0),
                                                &glm::Vec3::new(0.0, 0.0, 0.0),
                                                &glm::Vec3::new(0.0, 1.0, 0.0)));

            // set the projection matrix
            shader_program.set_uniform_mat4("projection_matrix",
                                            &glm::perspective(800 as f32 / SCREEN_H as f32, (3.14 / 180.0) * 60.0, 0.1, 100.0));
        };


        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        }

                        _ => {}
                    }
                }
            }

            // Rotate everything once per frame
            ball.rotate(glm::Vec3::new(0.0, 1.0, 0.0), 0.01);
            torus.rotate(glm::Vec3::new(0.0, 0.0, 1.0), -0.01);
            cube.rotate(glm::Vec3::new(1.0, 0.0, 1.0), 0.01);

            unsafe {
                gl::ClearColor((elapsed / 10.0) % 1.0, (elapsed / 7.0) % 1.0, (elapsed /8.0) % 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                // Render models
                ball.render(&shader_program);
                torus.render(&shader_program);
                cube.render(&shader_program);
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, ..
                }, ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
