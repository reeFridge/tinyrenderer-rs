extern crate sdl2;
extern crate assimp;
extern crate cgmath;

use assimp::Importer;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use std::cmp::Ordering;
use cgmath::InnerSpace;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Zero;
use cgmath::Matrix4;
use cgmath::SquareMatrix;

static WIDTH: u32 = 700;
static HEIGHT: u32 = 700;
static mut Z_BUFFER: [f32; 800 * 800] = [-1.; 800 * 800];

fn sort_points_by_y(points: &mut Vec<Vector3<f32>>) {
    for _ in 0..(points.len() - 1) {
        for j in 0..(points.len() - 1) {
            if points[j].y > points[j + 1].y {
                points.swap(j, j + 1);
            }
        }
    }
}

fn clamp<N>(a: N, min: N, max: N) -> N where N: PartialOrd {
    if a < min { return min }
    if a > max { return max }
    a
}

fn interpolate(min: f32, max: f32, gradient: f32) -> f32 {
    min + (max - min) * clamp(gradient, 0., 1.)
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
    fn pixel(&mut self, i32, i32, f32, Color);
    fn point(&mut self, Vector3<f32>, Color);
    fn line(&mut self, Point, Point, Color);
    fn process_scan_line(&mut self, i32, Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector3<f32>, Color);
    fn triangle(&mut self, Vector3<f32>, Vector3<f32>, Vector3<f32>, Color);
}

impl<'a> TinyRenderer for Renderer<'a> {
    fn pixel(&mut self, x: i32, y: i32, z: f32, c: Color, ) {
        let index = (x + y * WIDTH as i32) as usize;

        if unsafe { Z_BUFFER[index] < z } {
            let current_color = self.draw_color();
            self.set_draw_color(c);

            self.draw_point(Point::new(x, y)).unwrap();

            unsafe {
                Z_BUFFER[index] = z;
            }

            self.set_draw_color(current_color);
        }
    }

    fn point(&mut self, p: Vector3<f32>, c: Color) {
        if p.x >= 0. && p.y >= 0. && p.x < WIDTH as f32 && p.y < HEIGHT as f32 {
            self.pixel(p.x as i32, p.y as i32, p.z, c);
        }
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

    fn process_scan_line(&mut self, y: i32, pa: Vector3<f32>, pb: Vector3<f32>, pc: Vector3<f32>, pd: Vector3<f32>, c: Color) {
        let grad1 = match (pa.y as i32).cmp(&(pb.y as i32)) {
            Ordering::Equal => 1.,
            _ => (y as f32 - pa.y) / (pb.y - pa.y)
        };

        let grad2 = match (pc.y as i32).cmp(&(pd.y as i32)) {
            Ordering::Equal => 1.,
            _ => (y as f32 - pc.y) / (pd.y - pc.y)
        };

        let mut sx = interpolate(pa.x, pb.x, grad1) as i32;
        let mut ex = interpolate(pc.x, pd.x, grad2) as i32;

        if sx > ex {
            std::mem::swap(&mut sx, &mut ex);
        }

        let z1 = interpolate(pa.z, pb.z, grad1);
        let z2 = interpolate(pc.z, pd.z, grad2);

        for x in sx..ex {
            let gradient = (x - sx) as f32 / (ex - sx) as f32;

            let z = interpolate(z1, z2, gradient);
            self.point(Vector3::<f32>::new(x as f32, y as f32, z), c);
        }
    }

    fn triangle(&mut self, p0: Vector3<f32>, p1: Vector3<f32>, p2: Vector3<f32>, c: Color) {
        let mut points = vec![p0, p1, p2];
        sort_points_by_y(&mut points);

        let dp0p1 = match ((points[1].y - points[0].y) as i32).cmp(&0) {
            Ordering::Greater => (points[1].x - points[0].x) / (points[1].y - points[0].y),
            _ => 0.
        };

        let dp0p2 = match ((points[2].y - points[0].y) as i32).cmp(&0) {
            Ordering::Greater => (points[2].x - points[0].x) / (points[2].y - points[0].y),
            _ => 0.
        };

        for y in points[0].y as i32..points[2].y as i32 {
            if y < points[1].y as i32 {
                if dp0p1 > dp0p2 {
                    self.process_scan_line(y, points[0], points[2], points[0], points[1], c);
                } else {
                    self.process_scan_line(y, points[0], points[1], points[0], points[2], c);
                }
            } else {
                if dp0p1 > dp0p2 {
                    self.process_scan_line(y, points[0], points[2], points[1], points[2], c);
                } else {
                    self.process_scan_line(y, points[1], points[2], points[0], points[2], c);
                }
            }
        }
    }
}

fn viewport(x: f32, y: f32, w: f32, h: f32) -> Matrix4<f32> {
    let mut viewport = Matrix4::<f32>::identity();
    viewport[3][0] = x + w / 2.;
    viewport[3][1] = y + h / 2.;
    viewport[3][2] = 255. / 2.; //depth

    viewport[0][0] = w / 2.;
    viewport[1][1] = h / 2.;
    viewport[2][2] = 255. / 2.;
    return viewport;
}

fn lookat(eye: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let z = (eye - center).normalize();
    let x = up.cross(z).normalize();
    let y = z.cross(x).normalize();
    let mut res = Matrix4::<f32>::identity();

    for i in 0..3 {
        res[i][0] = x[i];
        res[i][1] = y[i];
        res[i][2] = z[i];
        res[3][i] = -center[i];
    }

    return res;
}

fn normal(v: Vector3<f32>) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

fn main() {
    let importer = Importer::new();
    let scene = importer.read_file("resources/model.obj").unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("tinyrenderer-rs", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();
    // front: ( 0,  0,  3)
    // back:  ( 0,  0, -3)
    // left:  (-3,  0,  0)
    // right: ( 3,  0,  0)
    // top:   ( 0,  3,  0.1) ?
    // down:  ( 0, -3,  0.1) ?
    let eye = Vector3::new(1., 1., 3.);
    let center = Vector3::new(0., 0., 0.);

    let model_view = lookat(eye, center, Vector3::new(0., 1., 0.));
    let light_dir = Vector3::new(0., -1., -3.).normalize();
    let mut projection = Matrix4::<f32>::identity();
    let (x, y, w, h) = (WIDTH as f32 / 8., HEIGHT as f32 / 8., WIDTH as f32 * 3. / 4., HEIGHT as f32 * 3. / 4.);
    let viewport = viewport(x, y, w, h);

    projection[2][3] = -1. / normal(eye - center);

    for mesh in scene.mesh_iter() {
        for face in mesh.face_iter() {
            let mut screen_coords: [Vector3<f32>; 3] = [Vector3::zero(); 3];
            let mut world_coords: [Vector3<f32>; 3] = [Vector3::zero(); 3];
            let mut intensity: [f32; 3] = [0.; 3];

            for j in 0..3 {
                let v: Vector3<f32> = match mesh.get_vertex(face[j]) {
                    Some(x) => x.into(),
                    None => Vector3::new(0., 0., 0.)
                };

                let mesh_normal: Vector3<f32> = match mesh.get_normal(face[j]) {
                    Some(x) => x.into(),
                    None => Vector3::new(0., 0., 0.)
                };

                let m = viewport * projection * model_view * Vector4::<f32>::new(v.x, v.y, v.z, 1.);
                let result_vector = Vector3::new(m.x / m.w, HEIGHT as f32 - (m.y / m.w), m.z / m.w);

                screen_coords[j as usize] = result_vector;
                world_coords[j as usize] = v;
                intensity[j as usize] = mesh_normal.dot(light_dir);
            }

            renderer.triangle(
                screen_coords[0],
                screen_coords[1],
                screen_coords[2],
                Color::RGB(
                    (intensity[0] * 255.) as u8,
                    (intensity[1] * 255.) as u8,
                    (intensity[2] * 255.) as u8
                )
            );
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
