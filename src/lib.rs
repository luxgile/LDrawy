#[macro_use]
pub mod drawy {

    use std::time::{Duration, Instant};

    use glium::{
        glutin::{
            self,
            event::{Event, VirtualKeyCode},
        },
        implement_vertex, Display, Program, Surface,
    };

    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    impl Color {
        pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
            Self { r, g, b, a }
        }

        pub const CLEAR: Color = Color::new(0.0, 0.0, 0.0, 0.0);
        pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
        pub const GRAY: Color = Color::new(0.5, 0.5, 0.5, 1.0);
        pub const SILVER: Color = Color::new(0.75, 0.75, 0.75, 1.0);
        pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
        pub const MAROON: Color = Color::new(0.5, 0.0, 0.0, 1.0);
        pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
        pub const PURPLE: Color = Color::new(0.5, 0.0, 0.5, 1.0);
        pub const GREEN: Color = Color::new(0.0, 0.5, 0.0, 1.0);
        pub const LIME: Color = Color::new(0.0, 1.0, 0.0, 1.0);
        pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
        pub const BLUE_NAVY: Color = Color::new(0.0, 0.0, 0.5, 1.0);
        pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
        pub const BLUE_TEAL: Color = Color::new(0.0, 0.5, 0.5, 1.0);
        pub const BLUE_AQUA: Color = Color::new(0.0, 1.0, 1.0, 1.0);
    }

    pub trait UserWindowHandler {
        fn startup(&self, _wnd: &Window) {}

        fn process_logic(&self) {}

        fn process_render(&self, _wnd: &Window) {}

        fn cleanup(&self, _wnd: &Window) {}
    }

    pub struct WindowSettings {
        max_fps: u64,
    }

    pub struct Window {
        settings: WindowSettings,
        display: Display,
        delta_time: f64,
        frame_count: u64,
    }

    impl Window {
        pub fn create_and_run(user: impl UserWindowHandler + 'static) {
            let event_loop = glutin::event_loop::EventLoop::new();
            let wb = glutin::window::WindowBuilder::new();
            let cb = glutin::ContextBuilder::new();
            let display = glium::Display::new(wb, cb, &event_loop).unwrap();

            let mut window = Window {
                settings: WindowSettings { max_fps: 60 },
                display,
                delta_time: 0.0,
                frame_count: 0,
            };

            user.startup(&window);

            event_loop.run(move |ev, _, flow| {
                window.frame_count += 1;
                let start_time = Instant::now();

                if Self::exit_request(&ev) {
                    *flow = glutin::event_loop::ControlFlow::Exit;
                    user.cleanup(&window);
                    return;
                }

                user.process_render(&window);

                //Limit framerate
                let elapsed_time = Instant::now().duration_since(start_time).as_millis() as u64;
                let wait_time = match window.settings.max_fps > 0
                    && 1000 / window.settings.max_fps >= elapsed_time
                {
                    true => 1000 / window.settings.max_fps - elapsed_time,
                    false => 0,
                };
                window.delta_time = wait_time as f64 / 1000.0;

                let wait_instant = start_time + Duration::from_millis(wait_time);
                *flow = glutin::event_loop::ControlFlow::WaitUntil(wait_instant);
            });
        }

        ///Get Canvas to start drawing in the back buffer.
        pub fn start_frame(&self, color: Color) -> Canvas {
            let mut canvas = Canvas {
                frame: self.display.draw(),
            };
            canvas.clear_color(color);
            canvas
        }

        ///Checks if the current event requires the window to be closed.
        fn exit_request(ev: &Event<()>) -> bool {
            if let glutin::event::Event::WindowEvent { event, .. } = ev {
                match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        return true;
                    }
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            return true;
                        }
                    }
                    _ => return false,
                }
            }
            false
        }

        #[must_use]
        #[inline]
        pub fn display(&self) -> &Display {
            &self.display
        }

        /// Get the canvas's delta time.
        #[must_use]
        #[inline]
        pub fn delta_time(&self) -> f64 {
            self.delta_time
        }

        /// Get the canvas's frame count.
        #[must_use]
        #[inline]
        pub fn frame_count(&self) -> u64 {
            self.frame_count
        }
    }

    implement_vertex!(Vertex, pos);
    #[derive(Copy, Clone)]
    pub struct Vertex {
        pos: [f32; 2],
    }

    impl Vertex {
        pub fn from_viewport(x: f32, y: f32) -> Self {
            Self { pos: [x, y] }
        }
        pub fn from_pixel(canvas: &Canvas, x: u32, y: u32) -> Self {
            let dim = canvas.frame.get_dimensions();
            Self {
                pos: [x as f32 / dim.0 as f32, y as f32 / dim.1 as f32],
            }
        }
        #[must_use]
        #[inline]
        pub fn x(&self) -> f32 {
            self.pos[0]
        }
        #[must_use]
        #[inline]
        pub fn y(&self) -> f32 {
            self.pos[1]
        }
    }

    #[macro_export]
    macro_rules! vertex {
        ($a:expr, $b:expr) => {
            Vertex::from_viewport($a, $b)
        };
    }
    pub(crate) use vertex;

    ///Queue of shapes to be drawn. All shapes added to the same batch will be drawn at the same time using the same brush.
    #[derive(Default)]
    pub struct ShapeBatch {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    }

    impl ShapeBatch {
        ///Add a triangle to the batch specifying its 3 vertices
        pub fn add_triangle(&mut self, v: [Vertex; 3]) {
            let index = self.indices.len() as u32;
            self.vertices.push(v[0]);
            self.indices.push(index);
            self.vertices.push(v[1]);
            self.indices.push(index + 1);
            self.vertices.push(v[2]);
            self.indices.push(index + 2);
        }

        ///Add a square to the batch specifying the center, width and height
        pub fn add_square(&mut self, c: Vertex, w: f32, h: f32) {
            //Adding vertices
            let hw = w / 2.0;
            let hh = h / 2.0;
            self.vertices.push(vertex!(c.x() - hw, c.y() - hh));
            self.vertices.push(vertex!(c.x() + hw, c.y() - hh));
            self.vertices.push(vertex!(c.x() - hw, c.y() + hh));
            self.vertices.push(vertex!(c.x() + hw, c.y() + hh));

            //Adding indices
            let index = self.indices.len() as u32;
            self.indices.push(index);
            self.indices.push(index + 1);
            self.indices.push(index + 2);
            self.indices.push(index + 2);
            self.indices.push(index + 1);
            self.indices.push(index + 3);
        }
    }

    ///Buffers created from the batch and prepared to be sent directly to the GPU
    pub struct ShapeBuffer {
        vertex_buffer: glium::VertexBuffer<Vertex>,
        index_buffer: glium::IndexBuffer<u32>,
    }

    impl ShapeBatch {
        pub fn bake_buffers(self, wnd: &Window) -> ShapeBuffer {
            let vertex_buffer = glium::VertexBuffer::new(wnd.display(), &self.vertices).unwrap();
            let index_buffer = glium::IndexBuffer::new(
                wnd.display(),
                glium::index::PrimitiveType::TrianglesList,
                &self.indices,
            )
            .unwrap();
            ShapeBuffer {
                vertex_buffer,
                index_buffer,
            }
        }
    }

    ///Used to configurate how to draw shapes in the GPU
    pub struct Brush {
        program: Program,
    }

    impl Brush {
        pub fn new_basic(wnd: &Window) -> Brush {
            let program = glium::Program::from_source(
                &wnd.display,
                r#"
            #version 330 core
            in vec2 pos;
            void main() {
                gl_Position = vec4(pos, 0.0, 1.0);
            }
            "#,
                r#"
            #version 330 core
            out vec4 color;
            void main() {
                color = vec4(1.0, 1.0, 0.0, 1.0);
            }
            "#,
                None,
            )
            .unwrap();
            Self { program }
        }
        pub fn from_source<'a>(
            wnd: &Window, vertex: &'a str, fragment: &'a str, geometry: Option<&'a str>,
        ) -> Brush {
            let program =
                glium::Program::from_source(&wnd.display, vertex, fragment, geometry).unwrap();
            Self { program }
        }
    }

    pub struct Canvas {
        frame: glium::Frame,
    }

    impl Canvas {
        pub fn clear_color(&mut self, color: Color) {
            self.frame.clear_color(color.r, color.g, color.b, color.a);
        }
        pub fn finish_canvas(self) -> Result<(), glium::SwapBuffersError> {
            self.frame.finish()
        }
        pub fn draw_batch(&mut self, _wnd: &Window, brush: &Brush, buffers: ShapeBuffer) {
            self.frame
                .draw(
                    &buffers.vertex_buffer,
                    &buffers.index_buffer,
                    &brush.program,
                    &glium::uniforms::EmptyUniforms, //TODO: Implement uniforms in ShapeBuffer
                    &Default::default(),
                )
                .unwrap();
        }
    }
}
