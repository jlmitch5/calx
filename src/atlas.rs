use std::num::{next_power_of_two};
use image::{GenericImage, SubImage, ImageBuf, Rgba, Pixel};
use util;
use geom::{V2, Rect};

pub struct AtlasBuilder {
    images: Vec<ImageBuf<Rgba<u8>>>,
    draw_offsets: Vec<V2<int>>,
}

impl AtlasBuilder {
    pub fn new() -> AtlasBuilder {
        AtlasBuilder {
            images: vec![],
            draw_offsets: vec![],
        }
    }

    pub fn push<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, offset: V2<int>, mut image: I) -> uint {

        let Rect(pos, dim) = util::crop_alpha(&image);
        let cropped = SubImage::new(&mut image,
            pos.0 as u32, pos.1 as u32, dim.0 as u32, dim.1 as u32);

        let (w, h) = cropped.dimensions();
        let img = ImageBuf::from_pixels(
            cropped.pixels().map::<Rgba<u8>>(
                |(_x, _y, p)| p.to_rgba())
            .collect(),
            w, h);
        self.images.push(img);
        self.draw_offsets.push(pos + offset);
        self.images.len() - 1
    }
}

pub struct Atlas {
    pub image: ImageBuf<Rgba<u8>>,
    pub vertices: Vec<Rect<f32>>,
    pub texcoords: Vec<Rect<f32>>,
}

impl Atlas {
    pub fn new(builder: &AtlasBuilder) -> Atlas {
        let dims : Vec<V2<int>> = builder.images.iter()
            .map(|img| { let (w, h) = img.dimensions(); V2(w as int, h as int) })
            .collect();

        // Add 1 pixel edges to images to prevent texturing artifacts from
        // adjacent pixels in separate subimages.
        let expanded_dims = dims.iter()
            .map(|v| v + V2(1, 1))
            .collect();

        // Guesstimate the size for the atlas container.
        let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(0, |a, b| a + b);
        let mut d = next_power_of_two((total_area as f64).sqrt() as uint) as u32;
        let mut offsets;

        loop {
            assert!(d < 1000000000); // Sanity check
            match util::pack_rectangles(V2(d as int, d as int), &expanded_dims) {
                Some(ret) => {
                    offsets = ret;
                    break;
                }
                None => {
                    d = d * 2;
                }
            }
        }

        // Blit subimages to atlas image.
        let mut image: ImageBuf<Rgba<u8>> = ImageBuf::new(d, d);
        for (i, &offset) in offsets.iter().enumerate() {
            util::blit(&builder.images[i], &mut image, offset);
        }

        let image_dim = V2(d, d);

        // Construct subimage rectangles.
        let texcoords: Vec<Rect<f32>> = offsets.iter().enumerate()
            .map(|(i, &offset)| Rect(scale_vec(offset, image_dim), scale_vec(dims[i], image_dim)))
            .collect();

        let vertices: Vec<Rect<f32>> = builder.draw_offsets.iter().enumerate()
            .map(|(i, &offset)| Rect(offset.map(|x| x as f32), dims[i].map(|x| x as f32)))
            .collect();

        assert!(texcoords.len() == vertices.len());

        return Atlas {
            image: image,
            vertices: vertices,
            texcoords: texcoords,
        };

        fn scale_vec(pixel_vec: V2<int>, image_dim: V2<u32>) -> V2<f32> {
            V2(pixel_vec.0 as f32 / image_dim.0 as f32,
              pixel_vec.1 as f32 / image_dim.1 as f32)
        }
    }
}
