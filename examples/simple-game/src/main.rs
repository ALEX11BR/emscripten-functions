use std::process::exit;

use sdl2::{
    event::Event,
    image::{InitFlag, LoadTexture, Sdl2ImageContext},
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
    EventPump,
};

#[cfg(target_os = "emscripten")]
use emscripten_functions::emscripten::set_main_loop_with_arg;

/// A container of all the variables needed for the game to run
struct App {
    // SDL system objects
    canvas: Canvas<Window>,
    event_pump: EventPump,
    _sdl_image: Sdl2ImageContext,
    // ^ can be removed if you don't plan on loading images beyond the `App::new` phase.

    // Textures (we use the `unsafe_textures` feature to make our life easier)
    square_texture: Texture,

    // Game state variables
    square_position: Rect,
}
impl App {
    /// Initialises an `App` struct with all the data needed to start & run the game
    fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let sdl_video = sdl.video()?;
        let _sdl_image = sdl2::image::init(InitFlag::PNG)?;

        let window = sdl_video
            .window("Simple Game", 600, 400)
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .map_err(|err| err.to_string())?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|err| err.to_string())?;
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        let event_pump = sdl.event_pump()?;

        let texture_creator = canvas.texture_creator();
        let square_texture = texture_creator.load_texture("assets/square.png")?;

        let square_position = Rect::new(0, 0, 100, 100);

        Ok(App {
            canvas,
            event_pump,
            _sdl_image,
            square_texture,
            square_position,
        })
    }
}

fn main_loop_iteration(app: &mut App) {
    for event in app.event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                exit(0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => {
                app.canvas.set_draw_color(Color::BLACK);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                app.canvas.set_draw_color(Color::WHITE);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                app.square_position.x -= 5;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                app.square_position.x += 5;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                app.square_position.y -= 5;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                app.square_position.y += 5;
            }
            _ => {}
        }
    }

    app.canvas.clear();
    let _ = app
        .canvas
        .copy(&app.square_texture, None, Some(app.square_position));
    app.canvas.present();
}

fn main() -> Result<(), String> {
    let app = App::new()?;

    #[cfg(not(target_os = "emscripten"))]
    {
        use std::{
            thread::sleep,
            time::{Duration, Instant},
        };

        let mut app = app;

        loop {
            let loop_start = Instant::now();

            main_loop_iteration(&mut app);

            let elapsed_milis = loop_start.elapsed().as_millis();
            sleep(Duration::from_millis(if elapsed_milis >= 1000 / 60 {
                0
            } else {
                (1000 / 60 - elapsed_milis) as u64
            }));
        }
    }
    #[cfg(target_os = "emscripten")]
    {
        set_main_loop_with_arg(main_loop_iteration, app, 0, true);
    }

    Ok(())
}
