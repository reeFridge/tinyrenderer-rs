extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use std::cmp::Ordering;

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

        let mut x0 = start.x();
        let mut y0 = start.y();
        let mut x1 = end.x();
        let mut y1 = end.y();

        let steep = (y1 - y0).abs() > (x1 - x0).abs();

        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }

        let reverse = x0 > x1;

        if reverse {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = (y1 - y0).abs();

        let mut err = dx / 2;
        let mut y = y0;
        let ystep = match y0.cmp(&y1) {
            Ordering::Less => 1,
            _ => -1
        };

        for x in x0..x1 {
            if steep {
                self.draw_point(Point::new(y, x)).unwrap();
            } else {
                self.draw_point(Point::new(x, y)).unwrap();
            }

            err -= dy;

            if err < 0 {
                y += ystep;
                err += dx;
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
    renderer.line(Point::new(250, 250), Point::new(91, 0), Color::RGB(0, 255, 255));
    renderer.line(Point::new(250, 250), Point::new(385, 0), Color::RGB(255, 0, 255));

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
