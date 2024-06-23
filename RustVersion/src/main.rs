use beacon::color_distance::{calculate_distance, PreciseRGB, RGB};
use crossbeam::thread;
use std::collections::HashMap;

struct Results<'a> {
    distance: f64,
    final_color: PreciseRGB,
    panes: Vec<&'a str>,
}

impl<'a> Results<'a> {
    pub fn new(distance: f64, final_color: PreciseRGB, panes: Vec<&'a str>) -> Self {
        Results {
            distance,
            final_color,
            panes,
        }
    }

    pub fn print(&self) {
        println!("Distance: {}", self.distance);
        println!("Calculated Color: {:?}", self.final_color);
        println!("Final Panes: {:?}", self.panes);
    }
}

fn main() {
    let colors: HashMap<&str, RGB> = [
        ("white", RGB::new_from_array([249, 255, 254])),
        ("light_gray", RGB::new_from_array([157, 157, 151])),
        ("gray", RGB::new_from_array([71, 79, 82])),
        ("black", RGB::new_from_array([29, 29, 33])),
        ("brown", RGB::new_from_array([131, 84, 50])),
        ("red", RGB::new_from_array([176, 46, 38])),
        ("orange", RGB::new_from_array([249, 128, 29])),
        ("yellow", RGB::new_from_array([254, 216, 61])),
        ("lime", RGB::new_from_array([128, 199, 31])),
        ("green", RGB::new_from_array([94, 124, 22])),
        ("cyan", RGB::new_from_array([22, 156, 156])),
        ("light_blue", RGB::new_from_array([58, 179, 218])),
        ("blue", RGB::new_from_array([60, 68, 170])),
        ("purple", RGB::new_from_array([137, 50, 184])),
        ("magenta", RGB::new_from_array([199, 78, 189])),
        ("pink", RGB::new_from_array([243, 139, 170])),
    ]
    .iter()
    .cloned()
    .collect();

    let target_color = RGB::new_from_number(0x00ffff);
    let results = find_closest_panes(target_color, colors);
    Results::print(&results)
}

fn find_closest_panes(target_color: RGB, available_colors: HashMap<&str, RGB>) -> Results {
    let mut results = vec![];

    thread::scope(|s| {
        for starting_color in available_colors.keys() {
            let available_colors = available_colors.clone(); // Clone to move ownership into closure

            // Spawn a thread for each item to process concurrently
            let handle = s.spawn(move |_| {
                generate_combinations(
                    1,
                    5,
                    &mut f64::INFINITY,
                    &mut vec![starting_color],
                    available_colors.clone(),
                    &mut vec![],
                    target_color,
                )
            });

            // Collect the thread's result
            results.push(handle.join().unwrap()); // unwrap() here assumes no thread panics
        }
    })
    .unwrap();

    let mut min_tuple: (f64, Vec<&str>) = (0.0, vec![]);
    let mut min_value = f64::INFINITY;

    for tuple in &results {
        let (value, _) = tuple;
        if *value < min_value {
            min_value = *value;
            min_tuple = tuple.clone();
        }
    }

    let (_, final_panes) = min_tuple;

    Results::new(
        min_value,
        convert_panes_to_rgb(final_panes.clone(), available_colors),
        final_panes,
    )
}

fn generate_combinations<'a>(
    current_depth: usize,
    max_depth: usize,
    distance: &mut f64,
    current_combination: &mut Vec<&'a str>, // Change to Vec<&'a str>
    available_colors: HashMap<&'a str, beacon::color_distance::RGB>,
    most_similar: &mut Vec<&'a str>,
    target_color: RGB,
) -> (f64, Vec<&'a str>) {
    if current_depth == max_depth {
        // Calculate current color and its distance from target
        let current_color = convert_panes_to_rgb(current_combination.clone(), available_colors);
        let dist = calculate_distance(current_color, target_color);

        // Update most_similar and distance if current combination is closer
        if dist < *distance {
            *distance = dist;
            most_similar.clone_from(current_combination)
        }

        // Return the current distance for comparison
        return (dist, most_similar.to_vec());
    }

    let mut min_distance = f64::INFINITY;

    for (key, _) in available_colors.clone().iter() {
        // Append the current possibility to the combination
        current_combination.push(key);

        // Recursive call to generate combinations at the next depth
        let (dist, _) = generate_combinations(
            current_depth + 1,
            max_depth,
            distance,
            current_combination,
            available_colors.clone(),
            most_similar,
            target_color,
        );

        // Track the minimum distance found in the recursive calls
        if dist < min_distance {
            min_distance = dist;
        }

        // Backtrack: Remove the last added possibility to try the next one
        current_combination.pop();
    }

    (min_distance, most_similar.to_vec()) // Return the minimum distance found in this depth level
}

fn convert_panes_to_rgb(panes: Vec<&str>, available_colors: HashMap<&str, RGB>) -> PreciseRGB {
    let n = panes.len();

    if n == 0 {
        return PreciseRGB::new(0.0, 0.0, 0.0);
    }

    let mut sum_colors = PreciseRGB::new(0.0, 0.0, 0.0);

    for (i, pane) in panes.iter().enumerate().skip(1) {
        let weight = 2u32.pow((i - 1) as u32) as f64;
        let color = available_colors.get(pane).unwrap();
        sum_colors.red += weight * color.red as f64;
        sum_colors.green += weight * color.green as f64;
        sum_colors.blue += weight * color.blue as f64;
    }

    let first_color = available_colors.get(&panes[0]).unwrap();
    sum_colors.red += first_color.red as f64;
    sum_colors.green += first_color.green as f64;
    sum_colors.blue += first_color.blue as f64;

    let scaling_factor = 1.0 / (2u32.pow((n - 1) as u32) as f64);

    PreciseRGB::new(
        scaling_factor * sum_colors.red,
        scaling_factor * sum_colors.green,
        scaling_factor * sum_colors.blue,
    )
}
