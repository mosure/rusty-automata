use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d,
            TextureDimension,
            TextureFormat,
        },
    }
};


fn convert_to_bytes(data: Vec<Vec<f32>>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for row in data {
        for value in row {
            // clip value to [-1, 1]
            let clipped_value = value.max(-1.0).min(1.0);
            // map [-1, 1] to [0, 255]
            let byte_value = ((clipped_value + 1.0) * 0.5 * 255.0).round() as u8;
            bytes.push(byte_value);
        }
    }
    bytes
}

fn sort_by_density(mut data: Vec<Vec<f32>>, subset_size: usize) -> Vec<Vec<f32>> {
    // Sort the outer Vec based on the "density" of the start and end of the inner Vecs
    data.sort_by(|a, b| {
        let mean_start_a = a.iter().take(subset_size).sum::<f32>() / (subset_size as f32);
        let mean_end_a = a.iter().rev().take(subset_size).sum::<f32>() / (subset_size as f32);
        let density_a = mean_start_a - mean_end_a;

        let mean_start_b = b.iter().take(subset_size).sum::<f32>() / (subset_size as f32);
        let mean_end_b = b.iter().rev().take(subset_size).sum::<f32>() / (subset_size as f32);
        let density_b = mean_start_b - mean_end_b;

        density_a.partial_cmp(&density_b).unwrap()
    });

    data
}

fn evaluate_activation_functions(
    activation_functions: Vec<&dyn Fn(f32) -> f32>,
    start: Option<f32>,
    stop: Option<f32>,
    resolution: Option<u32>,
) -> Vec<Vec<f32>> {
    let start = start.unwrap_or(-1.0);
    let stop = stop.unwrap_or(1.0);
    let resolution = resolution.unwrap_or(8192);

    let step = (stop - start) / resolution as f32;

    let mut result: Vec<Vec<f32>> = Vec::new();

    for activation_function in activation_functions {
        let mut row: Vec<f32> = Vec::new();

        for x in 0..resolution {
            let x = start + (x as f32 * step);
            let y = activation_function(x);
            row.push(y);
        }

        result.push(row);
    }

    result
}


/// assumes activation functions have input and output range [-1, 1]
pub fn generate_activation_texture() -> Image {
    let resolution = 8192;

    let mut values = evaluate_activation_functions(
        vec![
            // identity
            &|x| x,
            // inverse
            &|x| -x,
            // absolute
            &|x| x.abs(),
            // square
            &|x| x.powi(2),
            // sigmoid
            &|x| 1.0 / (1.0 + (-x).exp()),
            // tanh
            &|x| (2.0 / (1.0 + (-2.0 * x).exp())) - 1.0,
            // relu
            &|x| if x > 0.0 { x } else { 0.0 },
            // softplus
            &|x| (1.0 + x.exp()).ln(),
            // softsign
            &|x| x / (1.0 + x.abs()),
            // swish
            &|x| x / (1.0 + (-x).exp()),
            // mish
            &|x| x * (1.0 + (-x).exp()).ln_1p().tanh(),
            // bent identity
            &|x| ((x.powi(2) + 1.0).sqrt() - 1.0) / 2.0 + x,
            // sinc
            &|x| if x == 0.0 { 1.0 } else { x.sin() / x },
            // gaussian
            &|x| (-x.powi(2)).exp(),
            // soft exponential
            &|x| if x > 0.0 { (1.0 + x.exp()).ln() } else { -((-x).exp() + 1.0).ln() },
            // soft clipping
            &|x| if x > 1.0 { 1.0 } else if x < -1.0 { -1.0 } else { x },
            // soft clipping (cubic)
            &|x| if x > 1.0 { 1.0 } else if x < -1.0 { -1.0 } else { x - (x.powi(3) / 3.0) },
            // sinusoid
            &|x| (x * std::f32::consts::PI).sin(),
            // sinc
            &|x| if x == 0.0 { 1.0 } else { (x * std::f32::consts::PI).sin() / (x * std::f32::consts::PI) },
            // bipolar gaussian
            &|x| (-x.powi(2)).exp() - (-1.0 as f32).exp(),
            // bipolar sigmoid
            &|x| (1.0 / (1.0 + (-x).exp())) - 0.5,
            // stepwise linear
            &|x| if x < -0.5 { -1.0 } else if x < 0.5 { x + 0.5 } else { 1.0 },
            // high frequency activation
            &|x| (x * 100.0).sin(),
            // high frequency activation (absolute)
            &|x| (x * 100.0).sin().abs(),
        ],
        None,
        None,
        Some(resolution),
    );
    values = sort_by_density(
        values,
        (resolution / 2) as usize
    );

    let size = Extent3d {
        width: resolution,
        height: values.len() as u32,
        depth_or_array_layers: 1,
    };

    Image::new(
        size,
        TextureDimension::D2,
        convert_to_bytes(values),
        TextureFormat::R8Unorm,
    )
}
