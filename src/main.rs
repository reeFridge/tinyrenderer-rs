extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Renderer;

trait TinyRenderer {
    fn pixel(&mut self, Point, Color);
    fn line(&mut self, Point, Point, Color);
}

impl<'a> TinyRenderer for Renderer<'a> {
    fn pixel(&mut self, p: Point, c: Color) {
        let current_color = self.draw_color();

        self.set_draw_color(c);
        self.draw_point(p).unwrap();
        self.set_draw_color(current_color);
    }

    fn line(&mut self, start: Point, end: Point, c: Color) {
        let current_color = self.draw_color();
        self.set_draw_color(c);

        let dx = (end.x() - start.x()).abs();
        let dy = (end.y() - start.y()).abs();

        let mut err = 0;
        let derr = dy;
        let mut y = start.y();

        for x in start.x() .. end.x() {
            self.draw_point(Point::new(x, y)).unwrap();
            err += derr;

            if 2 * err >= dx {
                y += 1;
                err -= dx;
            }
        }

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
    renderer.pixel(Point::new(200, 200), Color::RGB(0, 255, 0));
    renderer.line(Point::new(250, 250), Point::new(385, 460), Color::RGB(0, 0, 255));

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
