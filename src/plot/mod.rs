use anyhow::Result;
use femtovg::{renderer::OpenGl, Align, Baseline, Canvas, Color, Paint, Path};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use winit::dpi::LogicalSize;
use winit::platform::run_return::EventLoopExtRunReturn as _;
#[cfg(feature = "glutin-x11")]
use winit::platform::unix::{WindowBuilderExtUnix, XWindowType};

use crate::{arr0d, IntoJArray, JArray, JError};

#[derive(Copy, Clone, Debug)]
struct Rect {
    // left, right (in x)
    l: f32,
    r: f32,
    // top, bottom (in y)
    t: f32,
    b: f32,
}

impl Rect {
    /// translate a value between 0 and 1, with 0 being left and 1 being right, into a window coordinate
    fn tx(&self, x: f32) -> f32 {
        let usable_width = self.r - self.l;
        x * usable_width + self.l
    }

    /// translate a value between 0 and 1, with 0 being BOTTOM and 1 being TOP, into a window coordinate
    fn ty(&self, y: f32) -> f32 {
        let usable_height = self.t - self.b;
        y * usable_height + self.b
    }
}

pub fn plot(arr: &JArray) -> Result<JArray> {
    let arr = arr.approx().ok_or_else(|| JError::DomainError)?;
    let (min, max) = arr
        .iter()
        .minmax_by_key(|c| OrderedFloat::from(**c))
        .into_option()
        .ok_or_else(|| JError::LengthError)?;

    let plot = arr
        .iter()
        .map(|x| (x - min) / (max - min))
        .collect::<Vec<_>>();

    pop_window(|canvas| {
        let (width, height) = (canvas.width(), canvas.height());
        let mut paint = Paint::color(Color::hex("ccc"));
        paint.set_font_size(18.);

        // The area upon which we're drawing, which the axises abut, in window coordinates,
        // i.e. 0,0 is the *top* left, and y is upside down
        let grid = Rect {
            l: 100.,
            r: width - 30.,

            t: 20.,
            b: height - 40.,
        };

        // y axis
        black_line(canvas, (grid.l, grid.t), (grid.l, grid.b + 5.));
        // x axis
        black_line(canvas, (grid.l - 5., grid.b), (grid.r, grid.b));

        // plot a line
        let max_x = (plot.len() - 1) as f32;
        let mut path = Path::new();
        for (x, y) in plot.iter().enumerate() {
            let tx = grid.tx(x as f32 / max_x);
            let ty = grid.ty(*y);
            if path.is_empty() {
                path.move_to(tx, ty);
            } else {
                path.line_to(tx, ty);
            }
        }
        canvas.stroke_path(&mut path, Paint::color(Color::hex("1111ee")));

        // label the x axis
        paint.set_text_align(Align::Center);
        paint.set_text_baseline(Baseline::Top);
        for x in 0..plot.len() {
            let tx = grid.tx(x as f32 / max_x);
            // tick
            black_line(canvas, (tx, grid.b + 3.), (tx, grid.b - 3.));
            canvas.fill_text(tx, grid.b + 5., format!("{x}"), paint)?;
        }

        // label the y axis
        paint.set_text_align(Align::Right);
        paint.set_text_baseline(Baseline::Middle);
        canvas.fill_text(grid.l - 8., grid.b, format!("{min}"), paint)?;
        canvas.fill_text(grid.l - 8., grid.t, format!("{max}"), paint)?;
        Ok(())
    })?;

    Ok(arr0d(69i64).into_jarray())
}

fn black_line(canvas: &mut Canvas<OpenGl>, (sx, sy): (f32, f32), (ex, ey): (f32, f32)) {
    let mut path = Path::new();
    path.move_to(sx, sy);
    path.line_to(ex, ey);
    canvas.stroke_path(&mut path, Paint::color(Color::hex("ccc")));
}

fn pop_window(mut paint_on: impl FnMut(&mut Canvas<OpenGl>) -> Result<()>) -> Result<()> {
    let window_size = glutin::dpi::PhysicalSize::new(1000, 600);
    let mut el = EventLoop::new();
    #[allow(unused_mut)]
    let mut wb = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_min_inner_size(LogicalSize::new(150, 100))
        .with_resizable(true)
        .with_title("rj plot.");

    #[cfg(feature = "glutin-x11")]
    {
        wb = wb.with_x11_window_type(vec![XWindowType::Dialog]);
    }

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let renderer =
        OpenGl::new_from_glutin_context(&windowed_context).expect("Cannot create renderer");
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(
        window_size.width as u32,
        window_size.height as u32,
        windowed_context.window().scale_factor() as f32,
    );

    // pyftsubset examples/assets/Roboto-Regular.ttf --unicodes=U+0001-00ff --output-file=roboto-ascii.ttf
    canvas.add_font_mem(include_bytes!("roboto-ascii.ttf"))?;

    el.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    windowed_context.window().set_visible(false);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let dpi_factor = windowed_context.window().scale_factor();
                let size = windowed_context.window().inner_size();
                canvas.set_size(size.width as u32, size.height as u32, dpi_factor as f32);
                canvas.clear_rect(
                    0,
                    0,
                    size.width as u32,
                    size.height as u32,
                    Color::rgbf(0.9, 0.9, 0.9),
                );

                paint_on(&mut canvas).expect("painting failed");

                canvas.save();
                canvas.reset();
                canvas.restore();

                canvas.flush();
                windowed_context
                    .swap_buffers()
                    .expect("swap_buffers inside event loop");
            }
            Event::MainEventsCleared => windowed_context.window().request_redraw(),
            _ => (),
        }
    });
    Ok(())
}
