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

fn sort_points_by_y(points: &mut Vec<Point>) {
    for _ in 0..(points.len() - 1) {
        for j in 0..(points.len() - 1) {
            if points[j].y() < points[j + 1].y() {
                points.swap(j, j + 1);
            }
        }
    }
}

fn get_line_points(start: Point, end: Point) -> Vec<Point> {
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

    let mut points: Vec<Point> = vec![];

    for x in x0..x1 {
        if steep {
            points.push(Point::new(y, x));
        } else {
            points.push(Point::new(x, y));
        }

        err -= dy;

        if err < 0 {
            y += ystep;
            err += dx;
        }
    }

    points
}

trait TinyRenderer {
    fn pixel(&mut self, Point, Color);
    fn line(&mut self, Point, Point, Color);
    fn triangle(&mut self, Point, Point, Point, Color);
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

        let points = get_line_points(start, end);

        for p in 0..points.len() {
            self.draw_point(points[p]).unwrap();
        }

        self.set_draw_color(current_color);
    }

    fn triangle(&mut self, p0: Point, p1: Point, p2: Point, c: Color) {
        let mut points = vec![p0, p1, p2];
        sort_points_by_y(&mut points);

        let horizon = get_line_points(points[1], points[2]);

        for l in 0..horizon.len() {
            self.line(points[0], horizon[l], c);
        }
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

    renderer.triangle(Point::new(20, 50), Point::new(100, 200), Point::new(40, 300), Color::RGB(0, 255, 0));

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
