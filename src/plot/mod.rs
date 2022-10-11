use anyhow::{anyhow, Context, Result};
use femtovg::{renderer::OpenGl, Align, Baseline, Canvas, Color, Paint, Path};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use ordered_float::NotNan;
use winit::platform::run_return::EventLoopExtRunReturn as _;
use winit::platform::unix::{WindowBuilderExtUnix, XWindowType};

use crate::{JArray, JError, Word};

pub fn plot(arr: &JArray) -> Result<Word> {
    let arr: Vec<NotNan<f32>> = match arr {
        JArray::FloatArray(x) => x
            .iter()
            .copied()
            .map(|c| NotNan::new(c as f32).with_context(|| anyhow!("{:?}", c)))
            .collect::<Result<_>>()?,
        _ => return Err(JError::DomainError.into()),
    };

    let min = *arr.iter().min().ok_or(JError::LengthError)?;
    let max = *arr.iter().max().ok_or(JError::LengthError)?;

    let plot = arr
        .iter()
        .map(|x| (x - min) / (max - min))
        .collect::<Vec<_>>();

    let window_size = glutin::dpi::PhysicalSize::new(1000, 600);
    let mut el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(true)
        .with_x11_window_type(vec![XWindowType::Dialog])
        .with_title("rj plot.");

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
    let font = canvas.add_font_mem(include_bytes!("roboto-ascii.ttf"))?;

    el.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(*physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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

                // draw_baselines(&mut canvas, &fonts, 5.0, 50.0, font_size, supports_emojis);

                let mut paint = Paint::color(Color::hex("ccc"));
                let lineu =
                    |canvas: &mut Canvas<OpenGl>, (sx, sy): (f32, f32), (ex, ey): (f32, f32)| {
                        let mut path = Path::new();
                        path.move_to(sx, sy);
                        path.line_to(ex, ey);
                        canvas.stroke_path(&mut path, Paint::color(Color::hex("ccc")));
                    };

                paint.set_font(&[font]);
                paint.set_text_baseline(Baseline::Top);
                paint.set_text_align(Align::Right);
                paint.set_font_size(24.);

                let zero_zero = (100f32, size.height as f32 - 30.);
                let one_one = (size.width as f32 - 30., 30f32);

                // y axis
                lineu(
                    &mut canvas,
                    (zero_zero.0, one_one.1),
                    (zero_zero.0, zero_zero.1 + 5.),
                );
                // x axis
                lineu(
                    &mut canvas,
                    (zero_zero.0 - 5., zero_zero.1),
                    (one_one.0, zero_zero.1),
                );

                let mut path = Path::new();
                let max_x = (plot.len() - 1) as f32;
                let mut points = plot.iter().enumerate();

                let usable_width = one_one.0 - zero_zero.0;
                let usable_height = one_one.1 - zero_zero.1;

                let tx = |x: f32| x * (usable_width as f32) + zero_zero.0 as f32;
                let ty = |y: f32| y * (usable_height as f32) + zero_zero.1 as f32;

                let (x, &y) = points.next().expect("non-empty");
                path.move_to(tx(x as f32 / max_x), ty(*y));
                for (x, &y) in points {
                    path.line_to(tx(x as f32 / max_x), ty(*y));
                }
                canvas.stroke_path(&mut path, Paint::color(Color::hex("1111ee")));

                paint.set_font_size(18.);
                for x in 0..plot.len() {
                    let tx = tx(x as f32 / max_x);
                    lineu(&mut canvas, (tx, zero_zero.1 + 3.), (tx, zero_zero.1 - 3.));
                    canvas
                        .fill_text(tx + 5., zero_zero.1 + 5., format!("{x}"), paint)
                        .expect("already painted");
                }

                canvas
                    .fill_text(
                        zero_zero.0 - 15.,
                        zero_zero.1 - 15.,
                        format!("{min}"),
                        paint,
                    )
                    .expect("text");
                canvas
                    .fill_text(zero_zero.0 - 15., one_one.1, format!("{max}"), paint)
                    .expect("text");

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

    Word::noun([69i64])
}
