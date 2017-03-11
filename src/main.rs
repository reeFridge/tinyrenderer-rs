extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Renderer;

trait TinyRenderer {
    fn set_pixel(&mut self, i32, i32, Color);
}

impl<'a> TinyRenderer for Renderer<'a> {
    fn set_pixel(&mut self, x: i32, y: i32, c: Color) {
        let current_color = self.draw_color();

        self.set_draw_color(c);
        self.draw_point(Point::new(x, y)).unwrap();
        self.set_draw_color(current_color);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("tinyrenderer-rs", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();

    //Draw pixel
    renderer.set_pixel(200, 200, Color::RGB(0, 255, 0));

    renderer.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        //code here
    }
}
