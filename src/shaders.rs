use crate::{
    color::Color,
    math::{Mat4, Vec3, Vec4},
};

/// Values that remain constant throughout one render call (the CPU equivalent
/// of GPU uniforms).
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub normal_matrix: Mat4,
    pub light_direction: Vec3,
    pub camera_position: Vec3,
}

/// Per-vertex attributes read from the OBJ or supplied by the application.
#[derive(Clone, Copy, Debug)]
pub struct VertexInput {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

/// Values produced by the vertex shader and interpolated for every fragment.
#[derive(Clone, Copy, Debug, Default)]
pub struct Varyings {
    pub world_position: Vec3,
    pub world_normal: Vec3,
    pub color: Vec3,
}

impl Varyings {
    pub fn lerp(self, other: Self, amount: f32) -> Self {
        Self {
            world_position: self.world_position
                + (other.world_position - self.world_position) * amount,
            world_normal: self.world_normal + (other.world_normal - self.world_normal) * amount,
            color: self.color + (other.color - self.color) * amount,
        }
    }

    pub fn weighted_sum(values: [Self; 3], weights: [f32; 3]) -> Self {
        Self {
            world_position: values[0].world_position * weights[0]
                + values[1].world_position * weights[1]
                + values[2].world_position * weights[2],
            world_normal: values[0].world_normal * weights[0]
                + values[1].world_normal * weights[1]
                + values[2].world_normal * weights[2],
            color: values[0].color * weights[0]
                + values[1].color * weights[1]
                + values[2].color * weights[2],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VertexOutput {
    pub clip_position: Vec4,
    pub varyings: Varyings,
}

impl VertexOutput {
    pub fn lerp(self, other: Self, amount: f32) -> Self {
        Self {
            clip_position: self.clip_position.lerp(other.clip_position, amount),
            varyings: self.varyings.lerp(other.varyings, amount),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FragmentInput {
    pub screen_position: Vec3,
    pub world_position: Vec3,
    pub world_normal: Vec3,
    pub color: Vec3,
}

/// CPU implementation of a vertex shader. Its required output is a position
/// in clip space; the remaining fields become interpolated varyings.
pub fn vertex_shader(vertex: VertexInput, uniforms: &Uniforms) -> VertexOutput {
    let world_homogeneous = uniforms.model * Vec4::from_point(vertex.position);
    let world_position = world_homogeneous.xyz() / world_homogeneous.w;
    let world_normal = (uniforms.normal_matrix * Vec4::from_direction(vertex.normal))
        .xyz()
        .normalized();
    let clip_position = uniforms.projection * uniforms.view * Vec4::from_point(world_position);

    VertexOutput {
        clip_position,
        varyings: Varyings {
            world_position,
            world_normal,
            color: vertex.color,
        },
    }
}

/// CPU implementation of a fragment shader. It runs for every covered sample
/// and returns the final RGB color before the depth operation.
pub fn fragment_shader(fragment: FragmentInput, uniforms: &Uniforms) -> Color {
    let normal = fragment.world_normal.normalized();
    let light = uniforms.light_direction.normalized();
    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let half_vector = (light + view).normalized();

    let diffuse = normal.dot(light).max(0.0);
    let specular = normal.dot(half_vector).max(0.0).powf(36.0) * 0.42;
    let edge_fade = (1.0 - fragment.screen_position.z * 0.08).clamp(0.88, 1.0);
    let intensity = (0.24 + diffuse * 0.76) * edge_fade;
    let lit_color = fragment.color * intensity + Vec3::new(specular, specular, specular);

    Color::new(
        (lit_color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (lit_color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (lit_color.z.clamp(0.0, 1.0) * 255.0) as u8,
    )
}
