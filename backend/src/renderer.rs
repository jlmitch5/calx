use std::default::Default;
use std::mem;
use glium;
use glium::texture;
use glium::LinearBlendingFactor::*;

pub struct Renderer {
    shader: glium::Program,
    texture: texture::Texture2d,
    params: glium::DrawParameters,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Renderer {
    pub fn new<T>(display: &glium::Display, texture_image: T) -> Renderer
        where T: texture::Texture2dData {
        let shader = glium::Program::from_source(display,
            include_str!("sprite.vert"),
            include_str!("sprite.frag"),
            None).unwrap();
        let texture = texture::Texture2d::new(display, texture_image);

        let mut params: glium::DrawParameters = Default::default();
        params.backface_culling = glium::BackfaceCullingMode::CullCounterClockWise;
        params.depth_function = glium::DepthFunction::IfLessOrEqual;
        params.blending_function = Some(glium::BlendingFunction::Addition {
            source: SourceAlpha, destination: OneMinusSourceAlpha });

        Renderer {
            shader: shader,
            texture: texture,
            params: params,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices = Vec::new();
        self.indices = Vec::new();
    }

    /// Draw the accumulated geometry and clear the accum buffers.
    pub fn draw<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface {
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.clear_depth(1.0);
        // Extract the geometry accumulation buffers and convert into
        // temporary Glium buffers.
        let vertices = glium::VertexBuffer::new(
            display, mem::replace(&mut self.vertices, Vec::new()));
        let indices = glium::IndexBuffer::new(
            display,
            glium::index_buffer::TrianglesList(
                mem::replace(&mut self.indices, Vec::new())));

        let uniforms = glium::uniforms::UniformsStorage::new("texture",
            glium::uniforms::Sampler(&self.texture, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }));

        target.draw(&vertices, &indices, &self.shader, &uniforms, &self.params).unwrap();
    }
}

#[vertex_format]
#[derive(Copy)]
/// Geometry vertex in on-screen graphics.
pub struct Vertex {
    /// Coordinates on screen
    pub pos: [f32; 3],
    /// Texture coordinates
    pub tex_coord: [f32; 2],
    /// Color for the light parts of the texture
    pub color: [f32; 4],
    /// Color for the dark parts of the texture
    pub back_color: [f32; 4],
}
