// src/lib.rs

pub mod color_distance {
    /// Represents an RGB color with red, green, and blue components.
    #[derive(Debug, Clone, Copy)]
    pub struct RGB {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
    }

    impl RGB {
        /// Creates a new RGB color.
        pub fn new(red: u8, green: u8, blue: u8) -> Self {
            RGB { red, green, blue }
        }

        pub fn to_array(&self) -> [f64; 3] {
            [self.red as f64, self.green as f64, self.blue as f64]
        }

        pub fn new_from_array(all: [u8; 3]) -> Self {
            RGB {
                red: all[0],
                green: all[1],
                blue: all[2],
            }
        }

        pub fn new_from_number(number: u32) -> Self {
            let red = ((number >> 16) & 0xFF) as u8;
            let green = ((number >> 8) & 0xFF) as u8;
            let blue = (number & 0xFF) as u8;

            RGB { red, green, blue }
        }
    }

    #[derive(Debug, Clone)]
    pub struct PreciseRGB {
        pub red: f64,
        pub green: f64,
        pub blue: f64,
    }

    impl PreciseRGB {
        /// Creates a new RGB color.
        pub fn new(red: f64, green: f64, blue: f64) -> Self {
            PreciseRGB { red, green, blue }
        }

        pub fn to_array(&self) -> [f64; 3] {
            [self.red as f64, self.green as f64, self.blue as f64]
        }
    }

    /// Calculates the Euclidean distance between two RGB colors.
    ///
    /// # Arguments
    ///
    /// * `color` - The first RGB color.
    /// * `target` - The second RGB color.
    ///
    /// # Returns
    ///
    /// A float representing the distance between the two colors.
    ///
    use ndarray::{array, Array1};

    fn s_rgb_to_linear_rgb(c: &[f64; 3]) -> [f64; 3] {
        let c: Vec<f64> = c.iter().map(|&x| x / 255.0).collect();
        let result: Vec<f64> = c
            .iter()
            .map(|&x| {
                if x <= 0.04045 {
                    x / 12.92
                } else {
                    ((x + 0.055) / 1.055).powf(2.4)
                }
            })
            .collect();
        [result[0], result[1], result[2]]
    }

    fn linear_rgb_to_xyz(rgb: &[f64; 3]) -> [f64; 3] {
        let m = array![
            [0.4124564, 0.3575761, 0.1804375],
            [0.2126729, 0.7151522, 0.0721750],
            [0.0193339, 0.1191920, 0.9503041]
        ];
        let rgb = array![rgb[0], rgb[1], rgb[2]];
        let xyz = m.dot(&rgb);
        [xyz[0], xyz[1], xyz[2]]
    }

    fn xyz_to_lab(xyz: &[f64; 3]) -> [f64; 3] {
        let xyz_ref = array![0.95047, 1.00000, 1.08883];
        let xyz: Array1<f64> = array![xyz[0], xyz[1], xyz[2]] / &xyz_ref;
        let epsilon = 0.008856;
        let kappa = 903.3;

        let f = |t: f64| {
            if t > epsilon {
                t.powf(1.0 / 3.0)
            } else {
                (kappa * t + 16.0) / 116.0
            }
        };
        let f_xyz = xyz.mapv(f);
        let l = 116.0 * f_xyz[1] - 16.0;
        let a = 500.0 * (f_xyz[0] - f_xyz[1]);
        let b = 200.0 * (f_xyz[1] - f_xyz[2]);
        [l, a, b]
    }

    fn delta_e(lab1: &[f64; 3], lab2: &[f64; 3]) -> f64 {
        ((lab1[0] - lab2[0]).powi(2) + (lab1[1] - lab2[1]).powi(2) + (lab1[2] - lab2[2]).powi(2))
            .sqrt()
    }

    pub fn calculate_distance(color: PreciseRGB, target: RGB) -> f64 {
        // Convert sRGB to linear RGB
        let linear_rgb1 = s_rgb_to_linear_rgb(&PreciseRGB::to_array(&color));
        let linear_rgb2 = s_rgb_to_linear_rgb(&RGB::to_array(&target));

        // Convert linear RGB to XYZ
        let xyz1 = linear_rgb_to_xyz(&linear_rgb1);
        let xyz2 = linear_rgb_to_xyz(&linear_rgb2);

        // Convert XYZ to L*a*b*
        let lab1 = xyz_to_lab(&xyz1);
        let lab2 = xyz_to_lab(&xyz2);

        // Compute the Delta E distance
        delta_e(&lab1, &lab2)
    }
}
