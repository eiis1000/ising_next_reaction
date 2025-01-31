// This project is a simulation of the Ising model in 2D using next-reaction.
mod ising_evolve;
mod ising_plot;
mod ising_store;

use ising_evolve::*;
use ising_plot::*;
use ising_store::*;
use rand::prelude::*;
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;

fn main() {
    let in_width: usize = get_input("Enter min width", 30);
    let width = in_width.next_power_of_two();
    if in_width != width {
        println!("Using actual width {width}.")
    }
    let in_height = get_input("Enter min height", width);
    let height = in_height.next_power_of_two();
    if in_height != height {
        println!("Using actual height {height}.")
    }
    let min_temp: f32 = get_input("Enter minimum temperature", 2.0);
    let max_temp: f32 = get_input("Enter maximum temperature", 2.6);
    let num_temp: u8 = get_input("Enter number of temperatures", 20);
    let num_shots: u8 = get_input("Enter number of shots", 5);
    let tmax: f32 = get_input("Enter tmax", 10000.0);
    let subtimes: usize = get_input("Enter subtimes", 10);

    let mut data: Vec<(f32, f64)> = Vec::new();

    for i in 0..=num_temp {
        let temp = min_temp + (max_temp - min_temp) * i as f32 / num_temp as f32;
        for shot in 0..num_shots {
            let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(shot.into());
            // update_temperature_display(temp);
            let mean_abs_mag = evolve_with(width, height, 1. / temp, tmax, subtimes, &mut rng);
            data.push((temp, mean_abs_mag));
        }
    }

    let filename =
        format!("{width}_{tmax}_{subtimes}_{min_temp}_{max_temp}_{num_temp}_{num_shots}");
    plot_simulation_data(&data, (filename.clone() + ".png").as_str()).unwrap();
    let mut file = File::create((filename + ".txt").as_str()).expect("Unable to create file");
    for (temp, mean_abs_mag) in &data {
        writeln!(file, "{}, {}", temp, mean_abs_mag).expect("Unable to write data");
    }
}

fn evolve_with(
    width: usize,
    height: usize,
    beta: f32,
    tmax: f32,
    subtimes: usize,
    rng: &mut impl Rng,
) -> f64 {
    println!("Building grid...");
    let mut ising = Ising::new(width, height, rng);
    println!("Done, built of size {}.", ising.get_size().2);

    println!("Building manager at temperature {}...", 1. / beta);
    let mut manager = IsingEvolutionManager::new(&mut ising, beta, rng);
    println!("Done.");

    let subsim_tmax = tmax / subtimes as f32;
    for st in 0..(subtimes - 1) {
        let n_flips = manager.evolve_ising_until(subsim_tmax * (st + 1) as f32, |_| ());
        println!(
            "{}\nabove was st {}, t={} of T={} with {} flipped and magnetization {}",
            manager.get_ising(),
            st,
            manager.get_time(),
            1. / beta,
            n_flips,
            manager.get_ising().magnetization()
        );
        // update_temperature_display(1.0 / beta);
    }

    let mut mags: Vec<f64> = Vec::new();
    let n_flips = manager.evolve_ising_until(tmax, |ising| {
        mags.push(ising.magnetization());
        // update_temperature_display(1.0 / beta);
    });
    println!(
        "{}\nabove was final at t={} of T={} with {} flipped and magnetization {}",
        manager.get_ising(),
        manager.get_time(),
        1. / beta,
        n_flips,
        manager.get_ising().magnetization()
    );
    let mean_abs_mag = mags.iter().map(|x| x.abs()).sum::<f64>() / mags.len() as f64;
    println!("The mean final absolute magnetization was {}", mean_abs_mag);
    // update_temperature_display(1.0 / beta);
    mean_abs_mag
}

fn get_input<T: FromStr + std::fmt::Display>(prompt: &str, default: T) -> T {
    print!("{} (default {}): ", prompt, default);
    stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().parse::<T>().unwrap_or_else(|_| {
        // eprintln!("Invalid input, try again.");
        // get_input(prompt, default)
        default
    })
}
