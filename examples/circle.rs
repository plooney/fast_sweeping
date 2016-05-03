extern crate fast_sweeping;
extern crate gnuplot;

use gnuplot::{Figure, Caption, Color, Fix, AxesCommon, PlotOption, DashType, Coordinate, TextColor};
use fast_sweeping::*;

fn main() {
    let n = 64;
    let mut u = vec![0.; (n + 1) * (n + 1)];

    let h = 1f64 / n as f64;

    let r = 0.3;

    for i in 0..(n+1) {
        for j in 0..(n+1) {
            let x = i as f64 * h - 0.5;
            let y = j as f64 * h - 0.5;
            u[i + j * (n + 1)] = (x * x + y * y).sqrt() - r;
        }
    }

    let orig = u.clone();

    let mut d = vec![0f64; (n + 1) * (n + 1)];

        init_dist(&mut d, &u, (n+1, n+1));
        fast_sweep_dist(&mut d, (n+1, n+1));

        for i in 0..d.len() {
            d[i] *= h;
            if u[i] < 0. {
                d[i] = -d[i];
            }
        }

        for i in 0..d.len() {
            u[i] = d[i];
        }

    let mut err = d.clone();
    for (err, u) in err.iter_mut().zip(orig.iter()) {
        *err -= *u;
    }

    let mut fg = Figure::new();

    fg.axes3d()
        .set_view_map()
        .surface(&err, n+1, n+1, None, &[])
        // .surface(&d, n+1, n+1, None, &[])
        // .surface(&u, n+1, n+1, None, &[])
        ;
    fg.show();
}
