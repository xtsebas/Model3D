use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use nalgebra_glm::Vec2;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use crate::light::Light;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use fastnoise_lite::FastNoiseLite;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Transformación de posición base
  let position = Vec4::new(
      vertex.position.x,
      vertex.position.y,
      vertex.position.z,
      1.0,
  );

  // Zoom para el relieve
  let zoom = 5.0;
  let displacement_amount = uniforms.noise.get_noise_3d(
      vertex.position.x * zoom,
      vertex.position.y * zoom,
      vertex.position.z * zoom,
  );

  // Desplazamiento a lo largo de la normal del vértice
  let displaced_position = vertex.position + vertex.normal * displacement_amount * 0.5;

  // Transformación del vértice desplazado
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * Vec4::new(
      displaced_position.x,
      displaced_position.y,
      displaced_position.z,
      1.0,
  );

  // División en perspectiva
  let w = transformed.w;
  let ndc_position = Vec4::new(
      transformed.x / w,
      transformed.y / w,
      transformed.z / w,
      1.0,
  );

  // Aplicar la matriz de viewport
  let screen_position = uniforms.viewport_matrix * ndc_position;

  // Transformar la normal
  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
  let transformed_normal = normal_matrix * vertex.normal;

  // Crear un nuevo vértice con atributos transformados
  Vertex {
      position: vertex.position,
      normal: vertex.normal,
      tex_coords: vertex.tex_coords,
      color: vertex.color,
      transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
      transformed_normal,
  }
}


pub fn select_shader(index: usize, fragment: &Fragment, uniforms: &Uniforms) -> Color {
    match index {
        0 => sun_shader().0,                           // El Sol
        1 => mercury_shader(fragment, uniforms),      // Mercurio
        2 => venus_shader(fragment, uniforms),        // Venus
        3 => earth_shader(fragment, uniforms),        // Tierra
        4 => mars_shader(fragment, uniforms),         // Marte
        5 => jupiter_shader(fragment, uniforms),      // Júpiter
        6 => saturn_shader(fragment, uniforms),       // Saturno
        7 => uranus_shader(fragment, uniforms),       // Urano
        8 => ring_shader(fragment).0,                 // Anillos de Saturno
        _ => sun_shader().0,                          // Por defecto: el Sol
    }
}

fn ring_shader(fragment: &Fragment) -> (Color, u32) {
    // Coordenadas en 2D para determinar la distancia desde el centro de los anillos
    let position = Vec2::new(fragment.vertex_position.x as f32, fragment.vertex_position.z as f32); // Usar X y Z para planos
    let distance_from_center = position.magnitude(); // Calcular la distancia desde el centro

    // Definir el número de bandas y su ancho
    let num_bands = 4; // Número total de bandas en los anillos
    let max_distance = 1.0_f32; // Distancia máxima para las bandas (ajustar según el tamaño de los anillos)
    let band_width = max_distance / num_bands as f32; // Ancho de cada banda

    // Calcular en qué banda está el fragmento actual
    let band_index = (distance_from_center / band_width).floor() as i32;

    // Variar el color de los anillos en función de su índice
    let band_colors = [
        Color::from_hex(0xB0C4DE), // Azul claro
        Color::from_hex(0x708090), // Gris pizarra
        Color::from_hex(0xA9A9A9), // Gris claro
        Color::from_hex(0xF5F5DC), // Beige
    ];

    // Seleccionar el color basado en el índice de la banda y el número de bandas
    let color = band_colors[(band_index.abs() % num_bands) as usize % band_colors.len()];

    // Aplicar un efecto de difuminado en los bordes de las bandas
    let edge_distance = (distance_from_center % band_width) / band_width;
    let smooth_edge = (1.0_f32 - edge_distance).clamp(0.0_f32, 1.0_f32);

    // Modificar la opacidad para dar un efecto de transparencia a los anillos
    let final_color = color * smooth_edge;

    (final_color, 0)
}


fn sun_shader() -> (Color, u32) {
    let base_color = Color::from_float(1.0, 0.9, 0.5); // Color amarillo/dorado para el Sol
    let emission = 100; // Máxima emisión para el efecto de glow/bloom
  
    (base_color, emission)
  }

fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores para diferentes biomas
  let land_color = Color::new(34, 139, 34);       // Verde para continentes
  let ocean_color = Color::new(30, 144, 255);     // Azul para océanos
  let snow_color = Color::new(255, 250, 250);     // Blanco para zonas polares
  let cloud_color = Color::new(255, 255, 255);    // Blanco para las nubes

  // Zoom para el ruido que genera los biomas
  let zoom = 15.0;
  let noise_value = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
      fragment.vertex_position.z * zoom,
  );

  // Capa base para la superficie terrestre
  let base_color = if noise_value < -0.3 {
      ocean_color.lerp(&Color::new(25, 105, 210), (noise_value + 0.3) / 0.3)
  } else if noise_value > 0.7 {
      land_color.lerp(&snow_color, (noise_value - 0.7) / 0.3)
  } else {
      ocean_color.lerp(&land_color, (noise_value + 0.3) / 1.0)
  };

  // Primera capa de nubes en movimiento
  let cloud_zoom1 = 10.0;
  let displacement_x1 = uniforms.noise.get_noise_2d(fragment.vertex_position.x * cloud_zoom1, fragment.vertex_position.y * cloud_zoom1) * 0.3;
  let displacement_z1 = uniforms.noise.get_noise_2d(fragment.vertex_position.z * cloud_zoom1, fragment.vertex_position.y * cloud_zoom1) * 0.3;
  let cloud_noise_value1 = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * cloud_zoom1 + displacement_x1,
      fragment.vertex_position.y * cloud_zoom1,
      fragment.vertex_position.z * cloud_zoom1 + displacement_z1,
  );

  // Opacidad de la primera capa de nubes
  let cloud_opacity1 = (cloud_noise_value1 * 0.5 + 0.5).min(1.0).max(0.0);

  // Segunda capa de nubes en movimiento (opcional, para mayor complejidad)
  let cloud_zoom2 = 8.0;
  let displacement_x2 = uniforms.noise.get_noise_2d(fragment.vertex_position.x * cloud_zoom2, fragment.vertex_position.y * cloud_zoom2) * 0.4;
  let displacement_z2 = uniforms.noise.get_noise_2d(fragment.vertex_position.z * cloud_zoom2, fragment.vertex_position.y * cloud_zoom2) * 0.4;
  let cloud_noise_value2 = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * cloud_zoom2 + displacement_x2,
      fragment.vertex_position.y * cloud_zoom2,
      fragment.vertex_position.z * cloud_zoom2 + displacement_z2,
  );

  // Opacidad de la segunda capa de nubes
  let cloud_opacity2 = (cloud_noise_value2 * 0.5 + 0.5).min(1.0).max(0.0);

  // Combinación de las capas de nubes con la superficie
  let combined_clouds = cloud_color * cloud_opacity1 + cloud_color * cloud_opacity2;
  let final_color = base_color.lerp(&combined_clouds, 0.5); // Ajusta la opacidad general de las nubes

  final_color
}


fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores base para Marte
  let base_color = Color::new(139, 69, 19);       // Marrón oscuro para la base
  let crater_color = Color::new(105, 54, 30);     // Color más oscuro para los cráteres
  let rocky_color = Color::new(169, 86, 30);      // Color intermedio para áreas rocosas

  // Capa base para la superficie rocosa
  let zoom = 20.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );

  // Interpolación para dar aspecto rocoso
  let base_layer = base_color.lerp(&rocky_color, noise_value * 0.5 + 0.5);

  // Añadir detalles de cráteres
  let crater_zoom = 8.0;
  let crater_noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * crater_zoom,
      fragment.vertex_position.y * crater_zoom,
  );

  // Interpolación para los cráteres
  if crater_noise_value < -0.3 {
      base_layer.lerp(&crater_color, (-crater_noise_value - 0.3) / 0.7)
  } else {
      base_layer
  }
}

fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Implementación del shader de Júpiter aquí
    Color::new(255, 200, 0) // Color ejemplo
}


fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let planet_color = Color::new(255, 225, 180);  // Color suave para Saturno
  let ring_color = Color::new(220, 220, 220);    // Color gris para los anillos

  let distance_from_center = fragment.vertex_position.x.hypot(fragment.vertex_position.y);
  let ring_width = 5.0;
  let ring_threshold = 10.0;

  // Determina si el fragmento está dentro de los anillos
  let in_rings = distance_from_center > ring_threshold && (distance_from_center % ring_width) < 1.0;

  if in_rings {
      ring_color
  } else {
      planet_color
  }
}


fn jupiter_gaseous_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let base_gas_color = Color::new(255, 223, 128); // Color amarillo suave

    // Configurar el ruido para variaciones en el gas
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    noise.set_frequency(Some(0.1));

    // Usar la posición del fragmento para calcular una variación de ruido
    let noise_value = noise.get_noise_2d(fragment.vertex_position.x, fragment.vertex_position.y) * 0.5 + 0.5;

    // Variar la intensidad del color amarillo basándose en el ruido
    let intensity = 0.5 + 0.5 * noise_value;
    let gas_color = Color::new(
        (base_gas_color.r as f32 * intensity) as u8,
        (base_gas_color.g as f32 * intensity) as u8,
        (base_gas_color.b as f32 * intensity) as u8,
    );

    // Mezcla de un tono más claro para dar una apariencia de gas difuso
    let overlay_color = Color::new(255, 250, 210);
    blend_colors(gas_color, overlay_color, 0.3)
}

// Función para mezclar dos colores con un factor de transparencia (pseudo-gas)
fn blend_colors(color1: Color, color2: Color, factor: f32) -> Color {
    Color::new(
        ((color1.r as f32 * factor + color2.r as f32 * (1.0 - factor)) as u8),
        ((color1.g as f32 * factor + color2.g as f32 * (1.0 - factor)) as u8),
        ((color1.b as f32 * factor + color2.b as f32 * (1.0 - factor)) as u8),
    )
}



fn mercury_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(169, 169, 169);  // Gris claro
  let crater_color = Color::new(105, 105, 105);  // Gris oscuro para cráteres

  let zoom = 20.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom
  );

  // Simular cráteres basados en el ruido
  let is_crater = noise_value < -0.2;

  let color = if is_crater {
      crater_color
  } else {
      base_color
  };

  color * fragment.intensity
}

fn venus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(218, 165, 32);     // Color cálido para la superficie
  let cloud_color = Color::new(255, 228, 181);   // Color crema para las nubes

  let zoom = 8.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );

  base_color.lerp(&cloud_color, noise_value.abs())
}



fn uranus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(173, 216, 230);  // Azul claro
  let highlight_color = Color::new(224, 255, 255);  // Azul verdoso

  let zoom = 5.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );

  base_color.lerp(&highlight_color, noise_value)
}

fn neptune_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(0, 0, 128);      // Azul profundo
  let highlight_color = Color::new(70, 130, 180); // Azul cielo

  let zoom = 5.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );

  base_color.lerp(&highlight_color, noise_value)
}

