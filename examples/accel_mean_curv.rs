extern crate fast_sweeping;
extern crate gnuplot;

use gnuplot::{Figure, Caption, Color, Fix, AxesCommon, PlotOption, DashType, Coordinate, TextColor,
            ContourStyle, AutoOption};
use fast_sweeping::*;

fn main() {
    let n = 64;
    let mut u = vec![0.; (n + 1) * (n + 1)];

    let h = 1f64 / n as f64;
    let t_max = 14;
    // let dt = h / 2.;
    let k = 128;
    let tau = h;

    let r = 0.3;

    // init with a circle of radius r
    for i in 0..(n+1) {
        for j in 0..(n+1) {
            let x = i as f64 * h - 0.5;
            let y = j as f64 * h - 0.5;
            u[i + j * (n + 1)] = (x * x + y * y).sqrt() - r;
        }
    }

    // initial data
    let orig = u.clone();
    let mut d_prev: Option<Vec<_>> = None;

    for t in 0..t_max {
        // compute distance function
        let mut d = vec![0f64; (n + 1) * (n + 1)];

            init_dist(&mut d, &u, (n+1, n+1));
            fast_sweep_dist(&mut d, (n+1, n+1));

            // create distance
            for i in 0..d.len() {
                d[i] *= h;
                if u[i] < 0. {
                    d[i] = -d[i];
                }
            }

            match d_prev {
                Some(dp) => {
                    for i in 0..u.len() {
                        u[i] = 2. * d[i] - dp[i];
                    }
                },
                None => u.clone_from(&d)
            }

        // solve wave equation for k timesteps of length tau
        let mut w = u.clone();
        for _ in 0..k {
            let mut v = u.clone();
            for j in 0..(n+1) {
                for i in 0..(n+1) {
                    let s = j * (n + 1) + i;
                    let uc = u[s];
                    let ul = if i == 0 { uc } else { u[s - 1] };
                    let ur = if i == n { uc } else { u[s + 1] };
                    let ut = if j == 0 { uc } else { u[s - n - 1] };
                    let ub = if j == n { uc } else { u[s + n + 1] };
                    v[s] = 2. * uc - w[s] + (tau * tau / h * h) * (ul + ur + ut + ub - 4. * uc);
                }
            }
            w.clone_from(&u);
            u.clone_from(&v);
        }

        d_prev = Some(d);
    }


    let mut fg = Figure::new();

    fg.axes3d()
        .set_aspect_ratio(AutoOption::Fix(1.))
        .set_view_map()
        .show_contours_custom(true, true, ContourStyle::Linear, AutoOption::Auto, &[0.])
        .surface(&u, n+1, n+1, None, &[])
        // .surface(&d, n+1, n+1, None, &[])
        // .surface(&u, n+1, n+1, None, &[])
        ;
    fg.show();
}