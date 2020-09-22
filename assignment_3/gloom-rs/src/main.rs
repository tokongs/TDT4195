mod toolbox;
mod particle_system;

extern crate nalgebra_glm as glm;

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod shader;
mod util;
mod mesh;
mod scene_graph;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use crate::particle_system::ParticleSystem;
use crate::scene_graph::SceneNode;
use crate::shader::Shader;
use glm::Vec3;
use std::ops::Neg;
use crate::toolbox::simple_heading_animation;
use crate::mesh::Helicopter;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

struct Heli {
    root: SceneNode,
    main: SceneNode,
    door: SceneNode,
    tail: SceneNode,
}

impl Heli {
    fn create_heli(helicopter: &Helicopter) -> Self {

        let heli_body_vao = unsafe {
            setup_vao(&helicopter.body.vertices, &helicopter.body.normals, &helicopter.body.colors, &helicopter.body.indices)
        };

        let heli_main_rotor_vao = unsafe {
            setup_vao(&helicopter.main_rotor.vertices, &helicopter.main_rotor.normals, &helicopter.main_rotor.colors, &helicopter.main_rotor.indices)
        };

        let heli_tail_rotor_vao = unsafe {
            setup_vao(&helicopter.tail_rotor.vertices, &helicopter.tail_rotor.normals, &helicopter.tail_rotor.colors, &helicopter.tail_rotor.indices)
        };

        let heli_door_vao = unsafe {
            setup_vao(&helicopter.door.vertices, &helicopter.door.normals, &helicopter.door.colors, &helicopter.door.indices)
        };

        let mut result = Self {
            root: SceneNode::from_vao(heli_body_vao, helicopter.body.index_count),
            main: SceneNode::from_vao(heli_main_rotor_vao, helicopter.main_rotor.index_count),
            tail: SceneNode::from_vao(heli_tail_rotor_vao, helicopter.tail_rotor.index_count),
            door: SceneNode::from_vao(heli_door_vao, helicopter.door.index_count)
        };

        result.tail.reference_point = Vec3::new(0.35, 2.3, 10.4);

        result.root.add_child(&result.main);
        result.root.add_child(&result.tail);
        result.root.add_child(&result.door);

        result
    }
}

unsafe fn setup_vao(vertices: &Vec<f32>, normals: &Vec<f32>, colors: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;

    // Generate VAO
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    //Generate vertex and index buffers
    gl::GenBuffers(1, &mut vbo);
    gl::GenBuffers(1, &mut ibo);

    let mut buffer_data = vec![];

    for i in 0..(vertices.len() / 3 - 1) {
        buffer_data.push(vertices[&i*3 + 0]);
        buffer_data.push(vertices[&i*3 + 1]);
        buffer_data.push(vertices[&i*3 + 2]);
        buffer_data.push(normals[&i*3 + 0]);
        buffer_data.push(normals[&i*3 + 1]);
        buffer_data.push(normals[&i*3 + 2]);
        buffer_data.push(colors[&i*4 + 0]);
        buffer_data.push(colors[&i*4 + 1]);
        buffer_data.push(colors[&i*4 + 2]);
        buffer_data.push(colors[&i*4 + 3]);
    }
    //Fill vertex buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(gl::ARRAY_BUFFER, util::byte_size_of_array(&buffer_data), util::pointer_to_array(&buffer_data), gl::STATIC_DRAW);

    // Fill index buffer
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, util::byte_size_of_array(indices), util::pointer_to_array(indices), gl::STATIC_DRAW);

    // Configure vertex attribute layout
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 10, ptr::null());

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 10, (3 * util::size_of::<f32>()) as *const gl::types::GLvoid);

    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, util::size_of::<f32>() * 10, (6 * util::size_of::<f32>()) as *const gl::types::GLvoid);
    gl::BindVertexArray(0);

    return vao;
}

unsafe fn draw_scene(root: &SceneNode, view_projection_matrix: &glm::Mat4, shader: &Shader){


    if root.index_count > 0{
        gl::BindVertexArray(root.vao_id);
        shader.activate();
        shader.set_uniform_mat4("vp_matrix", &view_projection_matrix);
        shader.set_uniform_mat4("model_matrix", &root.current_transformation_matrix);

        gl::DrawElements(gl::TRIANGLES, root.index_count as i32, gl::UNSIGNED_INT, ptr::null());
    }

    for &child in &root.children {
        draw_scene(&*child, view_projection_matrix, shader);
    }
}

unsafe fn update_node_transformations(root: &*mut SceneNode, transformation_so_far: &glm::Mat4){
    (*(*root)).current_transformation_matrix.fill_with_identity();
    (*(*root)).current_transformation_matrix = glm::scale(&(*(*root)).current_transformation_matrix, &(*(*root)).scale);
    (*(*root)).current_transformation_matrix = glm::translate(&(*(*root)).current_transformation_matrix, &(*(*root)).position);
    (*(*root)).current_transformation_matrix = glm::translate(&(*(*root)).current_transformation_matrix, &&(*(*root)).reference_point);
    (*(*root)).current_transformation_matrix = glm::rotate_x(&(*(*root)).current_transformation_matrix, (*(*root)).rotation.x);
    (*(*root)).current_transformation_matrix = glm::rotate_y(&(*(*root)).current_transformation_matrix, (*(*root)).rotation.y);
    (*(*root)).current_transformation_matrix = glm::rotate_z(&(*(*root)).current_transformation_matrix, (*(*root)).rotation.z);
    (*(*root)).current_transformation_matrix = glm::translate(&(*(*root)).current_transformation_matrix, &-&(*(*root)).reference_point);
    (*(*root)).current_transformation_matrix = transformation_so_far * &(*(*root)).current_transformation_matrix;

    println!("num children {}", (*(*root)).children.len() as u32);
    for mut child in &(*(*root)).children {
        update_node_transformations(child, &(*(*root)).current_transformation_matrix);
    }
}

fn rotate_rotors(heli: &mut Heli, delta_time: f32) {
    heli.main.rotation += glm::vec3(0.0, 200.0 * &delta_time, 0.0);
    heli.tail.rotation += glm::vec3(-200.0 * &delta_time, 0.0, 0.0);
}

fn change_heading(heli: &mut Heli, time: f32){
    let heading = simple_heading_animation(time);


    heli.root.position.x = heading.x;
    heli.root.position.z = heading.z;
    heli.root.rotation.x = heading.pitch;
    heli.root.rotation.y = heading.yaw;
    heli.root.rotation.z = heading.roll;
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
            gl::Enable (gl::DEPTH_TEST);
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



        // Basic usage of shader helper
        // The code below returns a shader object, which contains the field .program_id
        // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path), but it only needs to be called once
        // shader::ShaderBuilder::new().attach_file("./path/to/shader").link();
        let shader = unsafe {
            shader::ShaderBuilder::new().attach_file("./shaders/simple.frag").attach_file("./shaders/simple.vert").link()
        };

        let projection = glm::perspective(800 as f32 / SCREEN_H as f32, (3.14 / 180.0) * 60.0, 1.0, 500.0);

        let terrain_mesh = mesh::Terrain::load("resources/lunarsurface.obj");
        let terrain_vao = unsafe {
            setup_vao(&terrain_mesh.vertices, &terrain_mesh.normals, &terrain_mesh.colors, &terrain_mesh.indices)
        };

        let mut root_node = SceneNode::new();
        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.index_count);

        let helicopter = mesh::Helicopter::load("resources/helicopter.obj");

        let mut heli1 = Heli::create_heli(&helicopter);
        let mut heli2 = Heli::create_heli(&helicopter);
        let mut heli3 = Heli::create_heli(&helicopter);
        let mut heli4 = Heli::create_heli(&helicopter);
        let mut heli5 = Heli::create_heli(&helicopter);

        terrain_node.add_child(&heli1.root);
        terrain_node.add_child(&heli2.root);
        terrain_node.add_child(&heli3.root);
        terrain_node.add_child(&heli4.root);
        terrain_node.add_child(&heli5.root);

        root_node.add_child(&terrain_node);

        let scene_graph = root_node;

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
        let mut timer = 0.0;
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            let mut view = glm::identity();

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            x += delta_time * 10.0;
                        }
                        VirtualKeyCode::D => {
                            x += delta_time * -10.0;
                        }
                        VirtualKeyCode::W => {
                            z += delta_time * 10.0;
                        }
                        VirtualKeyCode::S => {
                            z += delta_time * -10.0;
                        }
                        VirtualKeyCode::Q => {
                            y += delta_time * 10.0;
                        }
                        VirtualKeyCode::E => {
                            y += delta_time * -10.0;
                        }
                        VirtualKeyCode::Up => {
                            rx += delta_time * -10.0;
                        }
                        VirtualKeyCode::Down => {
                            rx += delta_time * 10.0;
                        }
                        VirtualKeyCode::Right => {
                            ry += delta_time * 10.0;
                        }
                        VirtualKeyCode::Left => {
                            ry += delta_time * -10.0;
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

            timer += delta_time;
            view = glm::rotate_y(&view, ry);
            view = glm::rotate_x(&view, rx);

            view = glm::translate(&view, &glm::vec3(x, 0.0, 0.0));
            view = glm::translate(&view, &glm::vec3(0.0, y, 0.0));
            view = glm::translate(&view, &glm::vec3(0.0, 0.0, z));

            rotate_rotors(&mut heli1, delta_time);
            rotate_rotors(&mut heli2, delta_time);
            rotate_rotors(&mut heli3, delta_time);
            rotate_rotors(&mut heli4, delta_time);
            rotate_rotors(&mut heli5, delta_time);

            change_heading(&mut heli1,  timer);
            change_heading(&mut heli2, timer + 0.8);
            change_heading(&mut heli3, timer + 1.6);
            change_heading(&mut heli4, timer + 2.4);
            change_heading(&mut heli5, timer + 3.2);

            unsafe {
                let identity = glm::identity();
                update_node_transformations(&(&scene_graph as *const SceneNode as *mut SceneNode), &identity);
            }

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                draw_scene(&scene_graph, &(&projection*&view), &shader);
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
