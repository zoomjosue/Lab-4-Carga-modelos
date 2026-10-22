use crate::{
    framebuffer::Framebuffer,
    math::Vec3,
    shaders::{FragmentInput, Uniforms, Varyings, fragment_shader},
};

#[derive(Clone, Copy, Debug)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
    pub inverse_w: f32,
    pub varyings: Varyings,
}

impl ScreenVertex {
    pub const fn new(x: f32, y: f32, depth: f32, inverse_w: f32, varyings: Varyings) -> Self {
        Self {
            x,
            y,
            depth,
            inverse_w,
            varyings,
        }
    }
}

fn edge(a: ScreenVertex, b: ScreenVertex, x: f32, y: f32) -> f32 {
    (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
}

/// Fills one triangle using barycentric coordinates and a depth buffer.
pub fn draw_triangle(
    framebuffer: &mut Framebuffer,
    vertices: [ScreenVertex; 3],
    uniforms: &Uniforms,
) {
    let [a, b, c] = vertices;
    let area = edge(a, b, c.x, c.y);
    if area.abs() <= f32::EPSILON {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x =
        a.x.max(b.x)
            .max(c.x)
            .ceil()
            .min(framebuffer.width().saturating_sub(1) as f32) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y =
        a.y.max(b.y)
            .max(c.y)
            .ceil()
            .min(framebuffer.height().saturating_sub(1) as f32) as i32;
    let inverse_area = 1.0 / area;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sample_x = x as f32 + 0.5;
            let sample_y = y as f32 + 0.5;
            let weight_a = edge(b, c, sample_x, sample_y) * inverse_area;
            let weight_b = edge(c, a, sample_x, sample_y) * inverse_area;
            let weight_c = 1.0 - weight_a - weight_b;

            if weight_a >= 0.0 && weight_b >= 0.0 && weight_c >= 0.0 {
                let inverse_w =
                    weight_a * a.inverse_w + weight_b * b.inverse_w + weight_c * c.inverse_w;
                if inverse_w <= f32::EPSILON {
                    continue;
                }

                let perspective_weights = [
                    weight_a * a.inverse_w / inverse_w,
                    weight_b * b.inverse_w / inverse_w,
                    weight_c * c.inverse_w / inverse_w,
                ];
                let varyings = Varyings::weighted_sum(
                    [a.varyings, b.varyings, c.varyings],
                    perspective_weights,
                );
                let depth = weight_a * a.depth + weight_b * b.depth + weight_c * c.depth;
                let color = fragment_shader(
                    FragmentInput {
                        screen_position: Vec3::new(sample_x, sample_y, depth),
                        world_position: varyings.world_position,
                        world_normal: varyings.world_normal,
                        color: varyings.color,
                    },
                    uniforms,
                );
                framebuffer.set_pixel_depth(x, y, depth, color);
            }
        }
    }
}
