// This code is an AI-generated plotter to give me an idea
// of what my results look like without needing to use Python.

use anyhow::{anyhow, Result};
use noisy_float::types::N32;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot_simulation_data(data: &[(f32, f64)], output_path: &str) -> Result<()> {
    // Group data by x-value using noisy_float for safe comparison
    let mut groups: HashMap<N32, Vec<f64>> = HashMap::new();
    for &(x, y) in data {
        let x_key = N32::new(x);
        groups.entry(x_key).or_default().push(y);
    }

    // Sort x values and prepare plot data (convert x to f64 for plotting)
    let mut sorted_x: Vec<N32> = groups.keys().cloned().collect();
    sorted_x.sort();
    let plot_data: Vec<(f64, f64, f64)> = sorted_x
        .iter()
        .map(|&x_key| {
            let ys = &groups[&x_key];
            let mean = ys.iter().sum::<f64>() / ys.len() as f64;
            let variance =
                ys.iter().map(|y| (y - mean).powi(2)).sum::<f64>() / (ys.len() - 1) as f64;
            let std_dev = variance.sqrt();
            (f64::from(x_key), mean, std_dev)
        })
        .collect();

    // Create the plot
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = plot_data
        .first()
        .ok_or_else(|| anyhow!("No data available"))?
        .0;
    let x_max = plot_data
        .last()
        .ok_or_else(|| anyhow!("No data available"))?
        .0;
    let y_min = plot_data
        .iter()
        .map(|(_, y, _)| y)
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = plot_data
        .iter()
        .map(|(_, y, _)| y)
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption("Simulation Results", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            (x_min..x_max).log_scale(), // Remove log_scale if not needed
            y_min..y_max,
        )?;

    chart
        .configure_mesh()
        .x_labels(5)
        .y_labels(5)
        .x_desc("X")
        .y_desc("Y")
        .draw()?;

    // Draw the mean line
    chart.draw_series(LineSeries::new(
        plot_data.iter().map(|&(x, y, _)| (x, y)),
        &RED,
    ))?;

    // Add standard deviation as error bars
    chart.draw_series(plot_data.iter().map(|&(x, y, sd)| {
        let y_low = y - sd;
        let y_high = y + sd;
        ErrorBar::new_vertical(x, y_low, y, y_high, RED.filled(), 10)
    }))?;

    root.present()?;
    Ok(())
}

fn _old_plot_data(data: &Vec<(f32, f64)>) -> Result<()> {
    let root_area = BitMapBackend::new("output.png", (640, 480)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let min_x = *data
        .iter()
        .map(|(x, _)| x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| anyhow!("No data available"))?;
    let max_x = *data
        .iter()
        .map(|(x, _)| x)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| anyhow!("No data available"))?;
    let min_y = *data
        .iter()
        .map(|(_, y)| y)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| anyhow!("No data available"))?;
    let max_y = *data
        .iter()
        .map(|(_, y)| y)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| anyhow!("No data available"))?;

    let mut chart = ChartBuilder::on(&root_area)
        .caption(
            "Mean Absolute Magnetization vs Temperature",
            ("sans-serif", 50),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|(temp, mam)| (*temp, *mam)),
            &RED,
        ))?
        .label("Mean Absolute Magnetization")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
