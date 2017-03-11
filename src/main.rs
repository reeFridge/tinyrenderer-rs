extern crate sdl2;
extern crate assimp;

use assimp::Importer;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use std::cmp::Ordering;

use assimp::Vector3D;

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
    let (width, height) = (600, 600);
    let importer = Importer::new();
    let scene = importer.read_file("resources/model.obj").unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("tinyrenderer-rs", width, height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();

    for mesh in scene.mesh_iter() {
        for face in mesh.face_iter() {
            for j in 0..3 {
                let v0 = match mesh.get_vertex(face[j]) {
                    Some(x) => x,
                    None => Vector3D::new(0., 0., 0.)
                };

                let v1 = match mesh.get_vertex(face[(j + 1) % 3]) {
                    Some(x) => x,
                    None => Vector3D::new(0., 0., 0.)
                };

                let x0 = width as i32 - ((v0.x + 1.) * width as f32 / 2.) as i32;
                let y0 = height as i32 - ((v0.y + 1.) * height as f32 / 2.) as i32;
                let x1 = width as i32 - ((v1.x + 1.) * width as f32 / 2.) as i32;
                let y1 = height as i32 - ((v1.y + 1.) * height as f32 / 2.) as i32;

                renderer.line(Point::new(x0, y0), Point::new(x1, y1), Color::RGB(255, 255, 255));
            }
        }
    }

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
