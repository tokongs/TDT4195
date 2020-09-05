extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod shader;
mod util;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
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

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

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
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<f32>() * 7, ptr::null());

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, size_of::<f32>() * 7, (3 * size_of::<f32>()) as *const gl::types::GLvoid);
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
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    //windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    //windowed_context.window().set_cursor_visible(false);
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
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

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        let vertices: Vec<f32> = vec![
            0.0, 0.5, -3.0, 0.0, 1.0, 0.0, 0.5,
            -0.5, 0.0, -3.0, 0.0, 1.0, 0.0, 0.5,
            0.5, 0.0, -3.0, 0.0, 1.0, 0.0, 0.5,

            0.4, 0.5, -2.0, 0.0, 0.0, 1.0, 0.5,
            -0.1, 0.0, -2.0, 0.0, 0.0, 1.0, 0.5,
            0.9, 0.0, -2.0, 0.0, 0.0, 1.0, 0.5,

            -0.4, 0.5, -1.0, 1.0, 0.0, 0.0, 0.5,
            -0.9, 0.0, -1.0, 1.0, 0.0, 0.0, 0.5,
            0.1, 0.0, -1.0, 1.0, 0.0, 0.0, 0.5,

        ];
        let indices: Vec<u32> = vec![
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ];
        let vao = unsafe {
            setup_vao(&vertices, &indices)
        };

        // Basic usage of shader helper
        // The code below returns a shader object, which contains the field .program_id
        // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path), but it only needs to be called once
        // shader::ShaderBuilder::new().attach_file("./path/to/shader").link();
        let shader = unsafe {
            shader::ShaderBuilder::new().attach_file("./shaders/simple.frag").attach_file("./shaders/simple.vert").link()
        };
        unsafe {
            shader.activate();
            shader.set_uniform_mat4("projection", &glm::perspective(800 as f32 / SCREEN_H as f32, (3.14 / 180.0) * 60.0, 1.0, 100.0));
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        let mut value = 1.0;
        // The main rendering loop
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut rx = 0.0;
        let mut ry = 0.0;
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            let mut cameraMatrix = glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            );

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            x += delta_time * 1.0;
                        }
                        VirtualKeyCode::D => {
                            x += delta_time * -1.0;
                        }
                        VirtualKeyCode::W => {
                            z += delta_time * 1.0;
                        }
                        VirtualKeyCode::S => {
                            z += delta_time * -1.0;
                        }
                        VirtualKeyCode::Q => {
                            y += delta_time * 1.0;
                        }
                        VirtualKeyCode::E => {
                            y += delta_time * -1.0;
                        }
                        VirtualKeyCode::Up => {
                            rx += delta_time * -1.0;
                        }
                        VirtualKeyCode::Down => {
                            rx += delta_time * 1.0;
                        }
                        VirtualKeyCode::Right => {
                            ry += delta_time * 1.0;
                        }
                        VirtualKeyCode::Left => {
                            ry += delta_time * -1.0;
                        }
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                value += delta.0 / 10.0;
                *delta = (0.0, 0.0);
            }
            cameraMatrix =  glm::rotate_x(&cameraMatrix, rx);
            cameraMatrix =  glm::rotate_y(&cameraMatrix, ry);

            cameraMatrix =  glm::translate(&cameraMatrix, &glm::vec3(x, 0.0, 0.0));
            cameraMatrix =  glm::translate(&cameraMatrix, &glm::vec3(0.0, y, 0.0));
            cameraMatrix =  glm::translate(&cameraMatrix, &glm::vec3(0.0, 0.0, z));


            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindVertexArray(vao);
                shader.activate();

                shader.set_uniform_mat4("view", &cameraMatrix);


                gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
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
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
