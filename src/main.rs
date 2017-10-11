extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Window", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .build()
        .unwrap();
    let context = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    let mut event_pump = sdl_context.event_pump().unwrap();

    const n: usize = 20;
    const base: usize = 3;
    let mut points: [f32; n * base] = [0.0f32; n * base];

    let angle_step = 2.0 * std::f32::consts::PI / (n as f32);

    for i in 0 .. n {
        points[i * base] = (i as f32 * angle_step).cos() * 0.5;
        points[i * base + 1] = (i as f32 * angle_step).sin() * 0.5;
        points[i * base + 2] = 0.0f32;
    }


    let flag = [
        1.0f32, 1.0f32, 1.0f32,
        0.0f32, 0.0f32, 1.0f32,
        1.0f32, 0.0f32, 0.0f32
    ];
    let mut colors: [f32; n * base] = [0.0f32; n * base];

    for i in 0 .. n {
        colors[i * base] = flag[i % 3 * base];
        colors[i * base + 1] = flag[(i % 3 * base) + 1];
        colors[i * base + 2] = flag[(i % 3 * base) + 2];
    }


    let mut points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo as *mut u32);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       std::mem::size_of_val(&points) as isize,
                       (&points as *const f32) as *const _,
                       gl::STATIC_DRAW);
    }

    let mut colors_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut colors_vbo as *mut u32);
        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       std::mem::size_of_val(&colors) as isize,
                       (&colors as *const f32) as *const _,
                       gl::STATIC_DRAW);
    }

    let mut vao = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao as *mut u32);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3,
                                gl::FLOAT,
                                gl::FALSE,
                                0,
                                std::ptr::null());

        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
        gl::VertexAttribPointer(1, 3,
                                gl::FLOAT,
                                gl::FALSE,
                                0,
                                std::ptr::null());

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    }

    let vertex_shader = std::ffi::CString::new(r#"
        #version 320 es

        layout(location = 0) in vec3 vertex_position;
        layout(location = 1) in vec3 vertex_color;

        uniform float angle;

        out vec3 color;

        void main() {
            mat3 rotation_mat = mat3(
                cos(angle),  sin(angle), 0,
                -sin(angle), cos(angle), 0,
                0,           0,          1
            );

            color = vertex_color;
            gl_Position = vec4(rotation_mat * vertex_position, 1.0);
        }
    "#).unwrap();

    let fragment_shader = std::ffi::CString::new(r#"
        #version 320 es

        in highp vec3 color;
        out highp vec4 frag_color;

        void main() {
            frag_color = vec4(color, 1.0);
        }
    "#).unwrap();

    let mut vs = 0;
    unsafe {
        vs = gl::CreateShader(gl::VERTEX_SHADER);
        let p = vertex_shader.as_ptr() as *const i8;
        gl::ShaderSource(vs, 1, &p, std::ptr::null());
        gl::CompileShader(vs);

        let mut params: i32 = -1;
        gl::GetShaderiv(vs, gl::COMPILE_STATUS, &mut params as *mut i32);
        if gl::TRUE as i32 != params {
            let mut max_length = 0;
            let mut error_log: [u8; 1024] = [0; 1024];
            gl::GetShaderiv(vs,
                            gl::INFO_LOG_LENGTH,
                            &mut max_length as *mut i32);
            gl::GetShaderInfoLog(vs,
                                 max_length,
                                 &mut max_length as *mut i32,
                                 error_log.as_mut_ptr() as *mut i8);
            error_log[max_length as usize] = 0;

            panic!("ERROR: GL shader index {} did not compile: {}",
                   vs, std::str::from_utf8(&error_log).unwrap());
        }
    }

    let mut fs = 0;
    unsafe {
        fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        let p = fragment_shader.as_ptr() as *const i8;
        gl::ShaderSource(fs, 1, &p, std::ptr::null());
        gl::CompileShader(fs);

        let mut params: i32 = -1;
        gl::GetShaderiv(fs, gl::COMPILE_STATUS, &mut params as *mut i32);
        if gl::TRUE as i32 != params {
            let mut max_length = 0;
            let mut error_log: [u8; 1024] = [0; 1024];
            gl::GetShaderiv(fs,
                            gl::INFO_LOG_LENGTH,
                            &mut max_length as *mut i32);
            gl::GetShaderInfoLog(fs,
                                 max_length,
                                 &mut max_length as *mut i32,
                                 error_log.as_mut_ptr() as *mut i8);
            error_log[max_length as usize] = 0;

            panic!("ERROR: GL shader index {} did not compile: {}",
                   fs, std::str::from_utf8(&error_log).unwrap());
        }
    }

    let mut shader_program = 0;
    let mut angle_loc = 0;
    unsafe {
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, fs);
        gl::AttachShader(shader_program, vs);
        gl::LinkProgram(shader_program);

        let mut params: i32 = -1;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS,
                         &mut params as *mut i32);

        if gl::TRUE as i32 != params {
            panic!("ERROR: could not link shader program GL index {}",
                   shader_program);
        }

        let angle_loc = gl::GetUniformLocation(
            shader_program,
            std::ffi::CString::new("angle").unwrap().as_ptr()
        );

        if angle_loc == -1 {
            panic!("Could not set the angle uniform variable");
        }
    }

    let mut angle = 0.0f32;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        angle += 0.1;

        unsafe {
            gl::Uniform1f(angle_loc, angle);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, n as i32);
        }

        window.gl_swap_window();
    }
}
