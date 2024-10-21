use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod uniforms;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use uniforms::{Uniforms, create_noise, create_model_matrix, create_view_matrix, create_perspective_matrix, create_viewport_matrix};

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            // Apply fragment shader
            let shaded_color = fragment_shader(&fragment, &uniforms);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Render Solar System",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    // model position
    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0f32;

    // camera parameters
    let mut camera = Camera::new(
        Vec3::new(10.0, 30.0, 50.0),  // Alejamos la cámara
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let obj = Obj::load("assets/model/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 
    let mut time = 0;

    let noise = create_noise();
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms { 
        model_matrix: Mat4::identity(), 
        view_matrix: Mat4::identity(), 
        projection_matrix, 
        viewport_matrix, 
        time: 0, 
        noise
    };

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        // Renderizar la esfera central en el origen
        let central_translation = Vec3::new(0.0, 0.0, 0.0);
        let central_rotation = Vec3::new(0.0, 0.0, 0.0);
        let central_scale = 1.5f32;

        uniforms.model_matrix = create_model_matrix(central_translation, central_scale, central_rotation);
        uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        uniforms.time = time;

        framebuffer.set_current_color(0xFFDDDD); // Color de la esfera central
        render(&mut framebuffer, &uniforms, &vertex_arrays);

        // Radios de las esferas en órbita
        let radii = [12.0, 14.0, 16.0, 18.0, 20.0, 28.0]; // Radios para cada esfera

        let angle_step = 2.0 * PI / radii.len() as f32; // Ángulo entre cada esfera

        for (i, &radius) in radii.iter().enumerate() {
            let angle = i as f32 * angle_step;

            // Convertir coordenadas polares a cartesianas
            let x_position = radius * angle.cos();
            let z_position = radius * angle.sin();

            let translation = Vec3::new(x_position, 0.0, z_position); // Las esferas estarán en el plano XZ
            let rotation = Vec3::new(0.0, 0.0, 0.0);
            let scale = 1.0f32;

            uniforms.model_matrix = create_model_matrix(translation, scale, rotation);
            uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
            uniforms.time = time;

            framebuffer.set_current_color(0xFFAAAA); // Color de las esferas que orbitan
            render(&mut framebuffer, &uniforms, &vertex_arrays);

            // Si es la tercera esfera (índice 2), renderiza la esfera pequeña en una de sus esquinas
            if i == 2 {
                // Posición de la esfera pequeña (en este caso, se coloca a la derecha de la esfera)
                let small_sphere_offset = Vec3::new(20.0, 5.0, 0.0); // Ajusta estos valores para mover la esfera pequeña
                let small_sphere_translation = translation + small_sphere_offset; // Suma el offset a la posición de la tercera esfera
                
                uniforms.model_matrix = create_model_matrix(small_sphere_translation, 0.5, rotation); // Pequeña escala para la esfera
                framebuffer.set_current_color(0xFFFF00); // Color de la esfera pequeña
                render(&mut framebuffer, &uniforms, &vertex_arrays);
            }            
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.0;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.1;

    //  camera orbit controls
    if window.is_key_down(Key::Left) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
        camera.orbit(0.0, rotation_speed);
    }

    // Camera movement controls
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
        movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
        movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    // Camera zoom controls
    if window.is_key_down(Key::Up) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.zoom(-zoom_speed);
    }
}
