use std::error::Error;

use glam::UVec2;
use tridify_rs::*;

const OUTPUT_WIDTH: u32 = 1920;
const OUTPUT_HEIGHT: u32 = 1080;

pub fn main() -> Result<(), Box<dyn Error>> {
    let app = Tridify::new();
    let gpu_ctx = app.create_headless(TextureSize::D2(UVec2::new(OUTPUT_WIDTH, OUTPUT_HEIGHT)));
    let output_buffer = GpuBuffer::new(
        &gpu_ctx,
        (OUTPUT_WIDTH * OUTPUT_HEIGHT * Color::size_in_bytes()) as u64,
        wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    );

    //Create brush to draw the shapes.
    let mut brush = Brush::from_source(
        BrushDesc::default(),
        &gpu_ctx,
        include_str!("shader.wgsl").to_string(),
    )?;

    //Create a shape batch, add a triangle to it and create a GPU buffer with mesh data.
    let buffer = ShapeBatch::new()
        .add_triangle([
            vertex!(-0.5, -0.5, 0.0, Color::SILVER),
            vertex!(0.5, -0.5, 0.0, Color::SILVER),
            vertex!(0.0, 0.5, 0.0, Color::SILVER),
        ])
        .bake_buffers(&gpu_ctx);

    let mut gpu_cmds = gpu_ctx.create_gpu_cmds();
    let mut render_pass = gpu_cmds.start_render_pass(RenderOptions::default());
    render_pass.render_shapes(&gpu_ctx, &mut brush, &buffer);
    render_pass.finish();
    if let OutputSurface::Headless(texture) = gpu_ctx.get_output() {
        gpu_cmds.texture_to_buffer(texture, &output_buffer);
    }
    gpu_cmds.complete(&gpu_ctx);

    let data = pollster::block_on(output_buffer.map_buffer(&gpu_ctx));
    let image =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(OUTPUT_WIDTH, OUTPUT_HEIGHT, data)
            .unwrap();
    image.save("output.png").unwrap();
    //TODO: Output file does not make sense. Need to use RenderDoc API to debug issue.
    //TODO: Most likely an issue with pic format and data pulled from buffer.
    Ok(())
}
