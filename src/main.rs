use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nave_renderer::{framebuffer::Framebuffer, model::Model, renderer};
use std::{error::Error, path::PathBuf};

const SCREEN_WIDTH: usize = 900;
const SCREEN_HEIGHT: usize = 700;
const DEFAULT_MODEL: &str = "assets/Nave.obj";

struct Options {
    model_path: PathBuf,
}

fn options_from_arguments() -> Result<Options, String> {
    let mut arguments = std::env::args().skip(1);
    let mut model_path = PathBuf::from(DEFAULT_MODEL);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--model" => {
                model_path = arguments
                    .next()
                    .map(PathBuf::from)
                    .ok_or("falta la ruta después de --model")?;
            }
            "--help" | "-h" => {
                println!("Uso: cargo run --release -- [--model archivo.obj]");
                std::process::exit(0);
            }
            unknown => return Err(format!("argumento desconocido: {unknown}")),
        }
    }

    Ok(Options { model_path })
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = options_from_arguments()?;
    let model = Model::load(&options.model_path)?;
    println!(
        "Modelo cargado: {} vértices, {} normales, {} caras, {} triángulos",
        model.vertices.len(),
        model.normals.len(),
        model.faces.len(),
        model.triangle_count()
    );

    let mut framebuffer = Framebuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut camera = renderer::Camera::default();
    renderer::render(&mut framebuffer, &model, camera);

    let mut window = Window::new(
        "Nave.obj — Software Renderer",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut camera_changed = false;
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            camera.yaw -= 0.025;
            camera_changed = true;
        }
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
            camera.yaw += 0.025;
            camera_changed = true;
        }
        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
            camera.pitch = (camera.pitch + 0.02).min(1.2);
            camera_changed = true;
        }
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            camera.pitch = (camera.pitch - 0.02).max(-1.2);
            camera_changed = true;
        }
        if window.is_key_down(Key::Equal) {
            camera.distance_factor = (camera.distance_factor - 0.035).max(1.25);
            camera_changed = true;
        }
        if window.is_key_down(Key::Minus) {
            camera.distance_factor = (camera.distance_factor + 0.035).min(5.0);
            camera_changed = true;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            camera = renderer::Camera::default();
            camera_changed = true;
        }
        if camera_changed {
            renderer::render(&mut framebuffer, &model, camera);
        }
        window.update_with_buffer(framebuffer.pixels(), SCREEN_WIDTH, SCREEN_HEIGHT)?;
    }

    Ok(())
}
