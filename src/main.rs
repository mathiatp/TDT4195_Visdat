extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
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



// == // Modify and complete the function below for the first task
unsafe fn vao_setup(vertex_coordinates: &Vec<f32>, arr_indices: &Vec<u32>) -> u32 {
    let mut vao_ID:u32 = 0;
    gl::GenVertexArrays(1,&mut vao_ID);
    gl::BindVertexArray(vao_ID); //Opens the "file" that is the VOA so it is in the statemachine

    let mut buf_IDs:u32 = 0;
    gl::GenBuffers(1, &mut buf_IDs);
    gl::BindBuffer(gl::ARRAY_BUFFER, buf_IDs);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertex_coordinates),
        pointer_to_array(vertex_coordinates),
        gl::STATIC_DRAW
    );
        gl::VertexAttribPointer(0,2,gl::FLOAT,gl::FALSE,0,ptr::null());
        gl::EnableVertexAttribArray(0);

        let mut index_buf_IDs:u32 = 0;
        gl::GenBuffers(1, &mut index_buf_IDs);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,index_buf_IDs);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            byte_size_of_array(arr_indices),
            pointer_to_array(arr_indices),
            gl::STATIC_DRAW,
        );
        return vao_ID;
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
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

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
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // OBS OBS - Always check that the correct multiple of tringles in the draw-function further down is correct//

        // let vertex_cooridnates: Vec<f32> = vec![-0.9, 0.0, 0.0, 
        //                                         -0.7, 0.0, 0.0, 
        //                                         -0.8, 0.15, 0.0,
        //                                         // -0.7, 0.0, 0.0, 
        //                                         -0.5, 0.0, 0.0, 
        //                                         -0.6, 0.15, 0.0,
        //                                         //-0.5, 0.0, 0.0,
        //                                         -0.3, 0.0, 0.0,
        //                                         -0.4, 0.15, 0.0,
        //                                         //-0.3, 0.0, 0.0,
        //                                         -0.1, 0.0, 0.0,
        //                                         -0.2, 0.15, 0.0,
        //                                         //-0.1, 0.0, 0.0,
        //                                         -0.1, -0.2,0.0,
        //                                         0.05,-0.1,0.0
        //                                         ];
        // let arr_indices: Vec<u32> = vec![0, 1, 2,
        //                                  1, 3, 4,
        //                                  3,5,6,
        //                                  5,7,8,
        //                                  7,9,10];
        // let vao1: u32 = unsafe {vao_setup(&vertex_cooridnates, &arr_indices)};

        //Task 2 a)//
        // let vertex_cooridnates2: Vec<f32> = vec![0.6, -0.8, -1.2, 
        //                                         0.0, 0.4, -0.2, 
        //                                         -1.2, 0.0, 1.2];
        // let arr_indices2: Vec<u32> = vec![0, 1, 2];

        // let vao2: u32 = unsafe {vao_setup(&vertex_cooridnates2, &arr_indices2)};

        //Task 2b)//
        // let vertex_cooridnates3: Vec<f32> = vec![-0.3,  0.1, 0.0, 
        //                                           0.3,  0.1, 0.0, 
        //                                          -0.3,  0.4, 0.0,//
        //                                          -0.2, -0.1, 0.0,
        //                                           0.4, -0.1, 0.0,
        //                                           0.4, -0.4, 0.0
        //                                           ];
        // let arr_indices3: Vec<u32> = vec![0, 2, 1,
        //                                   3, 5, 4,
        //                                   ];

        // let vao3: u32 = unsafe {vao_setup(&vertex_cooridnates3, &arr_indices3)};
        //Task 2 a)//
        // let vertex_cooridnates4: Vec<f32> = vec![-0.6, -0.4,0.0, 
        //                                         -0.4, -0.4, 0.0, 
        //                                         -0.5, -0.2, 0.0];
        // let arr_indices4: Vec<u32> = vec![0, 1, 2];

        // let vao4: u32 = unsafe {vao_setup(&vertex_cooridnates4, &arr_indices4)};

        //Task 3 - Draw a circle//
        let mut vertex_cooridnates_circle: Vec<f32> = vec![0.5, 0.0];
        let mut arr_indices_circle: Vec<u32> = vec![0];
        let pi:f32 = 3.14159;
        let mut theta:f32 = 0.0;
        let mut x: f32 = 0.0;
        let mut y:f32 = 0.0;
        let number_of_points_in_circle :u32 = 1000;
        let rad_in_circle: f32 = 2.0*pi;
        let rad_between_vertices:f32 = rad_in_circle/number_of_points_in_circle as f32;
        let radius:f32 = 0.5;
        for i in 1..number_of_points_in_circle{
            x = radius*(rad_between_vertices*i as f32).cos();
            vertex_cooridnates_circle.push(x);
            
            y = radius*(rad_between_vertices*i as f32).sin();
            vertex_cooridnates_circle.push(y);

            arr_indices_circle.push(i);
            arr_indices_circle.push(i);

        }


        let vao5: u32 = unsafe {vao_setup(&vertex_cooridnates_circle, &arr_indices_circle)};




        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        let shader = unsafe{shader::ShaderBuilder::new()
            .attach_file("./shaders/simple.frag")
            .attach_file("./shaders/simple.vert")
            .link()
        };
        unsafe{shader.activate()};
       

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
                        },
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        },


                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }

            unsafe {
                gl::ClearColor(0.76862745, 0.71372549, 0.94901961, 1.0); // moon raker, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here
                gl::DrawElements(
                    gl::LINES,
                    2000,               //OBS!!!!!! HAS TO BE CHANGED BASED ON NUBER OF TRINAGLES/LINES!!!! 3*number of tringles/2*number of lines                 
                    gl::UNSIGNED_INT,
                    ptr::null()
                );
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
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
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
                    },
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
