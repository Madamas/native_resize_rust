use image::{math::utils::clamp, GenericImage, ImageBuffer, Pixel, Primitive};
use num_traits::NumCast;
use packed_simd::f32x4;

struct Filter<'a> {
	/// The filter's filter function.
	/// Pass function into struct as Box::new(triangle_kernel)
	pub kernel: Box<dyn Fn(f32) -> f32 + 'a>,

	/// The window on which this filter operates.
	pub support: f32,
}

fn triangle_kernel(x: f32) -> f32 {
	if x.abs() < 1.0 {
		1.0 - x.abs()
	} else {
		0.0
	}
}

// Sample the rows of the supplied image using the provided filter.
// The height of the image remains unchanged.
// ```new_width``` is the desired width of the new image
// ```filter``` is the filter to use for sampling.
// TODO: Do we really need the 'static bound on `I`? Can we avoid it?
fn horizontal_sample<I, P, S>(
	image: &I,
	new_width: u32,
	filter: &mut Filter,
) -> ImageBuffer<P, Vec<S>>
where
	I: GenericImage<Pixel = P> + 'static,
	P: Pixel<Subpixel = S> + 'static,
	S: Primitive + 'static,
{
	let (width, height) = image.dimensions();
	let mut out = ImageBuffer::new(new_width, height);

	for y in 0..height {
		let max = S::max_value();
		let max: f32 = NumCast::from(max).unwrap();

		let ratio = width as f32 / new_width as f32;

		for outx in 0..new_width {
			let inputx = (outx as f32 + 0.5) * ratio - 0.5;

			let left = (inputx - filter.support).ceil() as i64;
			let left = clamp(left, 0, width as i64 - 1) as u32;

			let right = {
				let real_right = inputx + filter.support;
				if real_right.fract() == 0.0 {
					(real_right - 1.0) as i64
				} else {
					real_right.floor() as i64
				}
			};
			let right = clamp(right, 0, width as i64 - 1) as u32;

			let mut sum = 0.;

			let mut t = (0., 0., 0., 0.);

			for i in left..right + 1 {
				let w = (filter.kernel)(i as f32 - inputx);
				sum += w;

				let x0 = clamp(i, 0, width - 1);
				let p = image.get_pixel(x0, y);

				let (k1, k2, k3, k4) = p.channels4();
				let vec: (f32, f32, f32, f32) = (
					NumCast::from(k1).unwrap(),
					NumCast::from(k2).unwrap(),
					NumCast::from(k3).unwrap(),
					NumCast::from(k4).unwrap(),
				);

				t.0 += vec.0 * w;
				t.1 += vec.1 * w;
				t.2 += vec.2 * w;
				t.3 += vec.3 * w;
			}

			let f32vec = f32x4::new(t.0, t.1, t.2, t.3);

			let (t1, t2, t3, t4) = (t.0 / sum, t.1 / sum, t.2 / sum, t.3 / sum);
			let t = Pixel::from_channels(
				NumCast::from(clamp(t1, 0.0, max)).unwrap(),
				NumCast::from(clamp(t2, 0.0, max)).unwrap(),
				NumCast::from(clamp(t3, 0.0, max)).unwrap(),
				NumCast::from(clamp(t4, 0.0, max)).unwrap(),
			);

			out.put_pixel(outx, y, t);
		}
	}

	out
}

// Sample the columns of the supplied image using the provided filter.
// The width of the image remains unchanged.
// ```new_height``` is the desired height of the new image
// ```filter``` is the filter to use for sampling.
// TODO: Do we really need the 'static bound on `I`? Can we avoid it?
fn vertical_sample<I, P, S>(
	image: &I,
	new_height: u32,
	filter: &mut Filter,
) -> ImageBuffer<P, Vec<S>>
where
	I: GenericImage<Pixel = P> + 'static,
	P: Pixel<Subpixel = S> + 'static,
	S: Primitive + 'static,
{
	let (width, height) = image.dimensions();
	let mut out = ImageBuffer::new(width, new_height);

	for x in 0..width {
		let max = S::max_value();
		let max: f32 = NumCast::from(max).unwrap();

		let ratio = height as f32 / new_height as f32;

		for outy in 0..new_height {
			// For an explanation of this algorithm, see the comments
			// in horizontal_sample.

			let inputy = (outy as f32 + 0.5) * ratio - 0.5;

			let left = (inputy - filter.support).ceil() as i64;
			let left = clamp(left, 0, height as i64 - 1) as u32;

			let right = {
				// A point above a pixel is NOT part of that pixel.
				let real_right = inputy + filter.support;
				if real_right.fract() == 0.0 {
					(real_right - 1.0) as i64
				} else {
					real_right.floor() as i64
				}
			};
			let right = clamp(right, 0, height as i64 - 1) as u32;

			let mut sum = 0.;

			let mut t = (0., 0., 0., 0.);

			for i in left..right + 1 {
				let w = (filter.kernel)(i as f32 - inputy);
				sum += w;

				let y0 = clamp(i, 0, height - 1);
				let p = image.get_pixel(x, y0);

				let (k1, k2, k3, k4) = p.channels4();
				let vec: (f32, f32, f32, f32) = (
					NumCast::from(k1).unwrap(),
					NumCast::from(k2).unwrap(),
					NumCast::from(k3).unwrap(),
					NumCast::from(k4).unwrap(),
				);

				t.0 += vec.0 * w;
				t.1 += vec.1 * w;
				t.2 += vec.2 * w;
				t.3 += vec.3 * w;
			}

			let (t1, t2, t3, t4) = (t.0 / sum, t.1 / sum, t.2 / sum, t.3 / sum);
			let t = Pixel::from_channels(
				NumCast::from(clamp(t1, 0.0, max)).unwrap(),
				NumCast::from(clamp(t2, 0.0, max)).unwrap(),
				NumCast::from(clamp(t3, 0.0, max)).unwrap(),
				NumCast::from(clamp(t4, 0.0, max)).unwrap(),
			);

			out.put_pixel(x, outy, t);
		}
	}

	out
}

/// Resize the supplied image to the specified dimensions.
/// ```nwidth``` and ```nheight``` are the new dimensions.
/// ```filter``` is the sampling filter to use.
// TODO: Do we really need the 'static bound on `I`? Can we avoid it?
pub fn resize<I: GenericImage + 'static>(
	image: &I,
	nwidth: u32,
	nheight: u32,
) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
where
	I::Pixel: 'static,
	<I::Pixel as Pixel>::Subpixel: 'static,
{
	let mut method = Filter {
		kernel: Box::new(triangle_kernel),
		support: 1.0,
	};

	let tmp = vertical_sample(image, nheight, &mut method);
	horizontal_sample(&tmp, nwidth, &mut method)
}
