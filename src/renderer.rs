use crate::{
    clipping::clip_triangle,
    color::Color,
    framebuffer::Framebuffer,
    math::{Mat4, Vec3},
    model::{Face, Model},
    shaders::{Uniforms, VertexInput, VertexOutput, vertex_shader},
    triangle::{ScreenVertex, draw_triangle},
};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance_factor: f32,
    pub field_of_view_degrees: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.55,
            pitch: 0.30,
            distance_factor: 2.45,
            field_of_view_degrees: 48.0,
        }
    }
}

struct TransformedFace {
    vertices: Vec<VertexOutput>,
}

pub fn render(framebuffer: &mut Framebuffer, model: &Model, camera: Camera) {
    draw_space_background(framebuffer);
    let uniforms = create_uniforms(model, camera, framebuffer.width(), framebuffer.height());

    // 1. Vertex shader: all face vertices become clip-space vertices first.
    let transformed_faces = run_vertex_stage(model, &uniforms);

    // 2. Primitive assembly: each OBJ polygon is triangulated as a fan.
    for face in transformed_faces {
        for index in 1..face.vertices.len() - 1 {
            let triangle = [
                face.vertices[0],
                face.vertices[index],
                face.vertices[index + 1],
            ];

            // 3. Real homogeneous clipping against left, right, bottom, top,
            // near and far. A crossing triangle may produce several triangles.
            for clipped_triangle in clip_triangle(triangle) {
                let Some(screen_vertices) =
                    to_screen_triangle(clipped_triangle, framebuffer.width(), framebuffer.height())
                else {
                    continue;
                };

                // 4. Rasterization + fragment shader + per-fragment depth test.
                draw_triangle(framebuffer, screen_vertices, &uniforms);
            }
        }
    }
}

fn create_uniforms(model: &Model, camera: Camera, width: usize, height: usize) -> Uniforms {
    let target = model.center();
    let radius = model.bounding_radius(target).max(1.0);
    let orbit_direction = Vec3::new(
        camera.yaw.sin() * camera.pitch.cos(),
        camera.pitch.sin(),
        -camera.yaw.cos() * camera.pitch.cos(),
    );
    let eye = target + orbit_direction * (radius * camera.distance_factor);

    Uniforms {
        model: Mat4::IDENTITY,
        view: Mat4::look_at(eye, target, Vec3::Y),
        projection: Mat4::perspective(
            camera.field_of_view_degrees.to_radians(),
            width as f32 / height as f32,
            0.1,
            radius * 20.0,
        ),
        // The model matrix is currently identity. If non-uniform scale is added,
        // this should become the inverse transpose of the model matrix.
        normal_matrix: Mat4::IDENTITY,
        light_direction: Vec3::new(-0.35, 0.85, -0.55).normalized(),
        camera_position: eye,
    }
}

fn run_vertex_stage(model: &Model, uniforms: &Uniforms) -> Vec<TransformedFace> {
    model
        .faces
        .iter()
        .map(|face| {
            let fallback_normal = face_normal(model, face);
            let center = face
                .vertex_indices
                .iter()
                .map(|index| model.vertices[*index])
                .fold(Vec3::ZERO, |sum, position| sum + position)
                / face.vertex_indices.len() as f32;
            let color = material_color(center);

            let vertices = face
                .vertex_indices
                .iter()
                .enumerate()
                .map(|(corner, position_index)| {
                    let normal = face.normal_indices[corner]
                        .map(|normal_index| model.normals[normal_index])
                        .unwrap_or(fallback_normal);
                    vertex_shader(
                        VertexInput {
                            position: model.vertices[*position_index],
                            normal,
                            color,
                        },
                        uniforms,
                    )
                })
                .collect();

            TransformedFace { vertices }
        })
        .collect()
}

fn face_normal(model: &Model, face: &Face) -> Vec3 {
    let a = model.vertices[face.vertex_indices[0]];
    let b = model.vertices[face.vertex_indices[1]];
    let c = model.vertices[face.vertex_indices[2]];
    (b - a).cross(c - a).normalized()
}

fn to_screen_triangle(
    triangle: [VertexOutput; 3],
    width: usize,
    height: usize,
) -> Option<[ScreenVertex; 3]> {
    let mut screen = [to_screen_vertex(triangle[0], width, height)?; 3];
    screen[1] = to_screen_vertex(triangle[1], width, height)?;
    screen[2] = to_screen_vertex(triangle[2], width, height)?;
    Some(screen)
}

fn to_screen_vertex(vertex: VertexOutput, width: usize, height: usize) -> Option<ScreenVertex> {
    if vertex.clip_position.w.abs() <= f32::EPSILON {
        return None;
    }

    // Perspective divide: clip space -> normalized device coordinates.
    let inverse_w = 1.0 / vertex.clip_position.w;
    let ndc_x = vertex.clip_position.x * inverse_w;
    let ndc_y = vertex.clip_position.y * inverse_w;
    let ndc_z = vertex.clip_position.z * inverse_w;

    // Viewport transform: NDC [-1, 1] -> framebuffer pixels.
    Some(ScreenVertex::new(
        (ndc_x + 1.0) * 0.5 * width as f32,
        (1.0 - ndc_y) * 0.5 * height as f32,
        (ndc_z * 0.5 + 0.5).clamp(0.0, 1.0),
        inverse_w,
        vertex.varyings,
    ))
}

fn material_color(center: Vec3) -> Vec3 {
    let color = if center.y > 0.48 && center.z > -1.2 && center.z < 0.2 {
        Color::new(79, 226, 246)
    } else if center.z > 2.25 {
        Color::new(239, 105, 57)
    } else if center.x.abs() > 1.0 {
        Color::new(74, 126, 190)
    } else {
        Color::new(126, 180, 224)
    };
    Vec3::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}

fn draw_space_background(framebuffer: &mut Framebuffer) {
    framebuffer.clear_gradient(Color::new(5, 10, 25), Color::new(16, 31, 58));

    let mut state = 0x5EED_u32;
    for star in 0..190 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let x = (state as usize % framebuffer.width()) as i32;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let y = (state as usize % framebuffer.height()) as i32;
        let brightness = 115 + (state >> 24) as u8 / 2;
        framebuffer.set_pixel(x, y, Color::new(brightness, brightness, 255));
        if star % 31 == 0 {
            framebuffer.set_pixel(x + 1, y, Color::new(90, 115, 160));
        }
    }
}
