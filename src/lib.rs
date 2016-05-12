//! The fast sweeping method for the computation of the signed distance function in 2D.
//!
//! ## References
//!
//! [1] Zhao, Hongkai A fast sweeping method for eikonal equations. Math. Comp. 74 (2005), no. 250,
//! 603–627.

mod level_set;
mod eikonal;

/// Computes the signed distance from the _zero_ level set of the _linear_ function given by the
/// values of `u` on a regular grid of dimensions `dim` and stores the result to a preallocated
/// array `d`.
///
/// `h` is the distance between neighboring nodes.
///
/// Returns `std::f64::MAX` if all `u` are positive (`-std::f64::MAX` if all `u` are negative).
pub fn signed_distance(d: &mut [f64], u: &[f64], dim: (usize, usize), h: f64) {
    assert_eq!(dim.0 * dim.1, u.len());
    assert_eq!(dim.0 * dim.1, d.len());
    level_set::init_dist(d, u, dim);
    eikonal::fast_sweep_dist(d, dim);

    // compute the signed distance function from the solution of the eikonal equation
    for i in 0..d.len() {
        if u[i] < 0. {
            d[i] = -d[i] * h;
        } else {
            d[i] *= h;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    extern crate quickcheck;
    extern crate ndarray;
    use self::quickcheck::quickcheck;
    use self::ndarray::prelude::*;
    use self::ndarray::Si;

    fn check_line(gx: f64, gy: f64, c: f64, n: usize, tol: f64, print: bool) -> bool {
        let xs = OwnedArray::linspace(0., 1., n);
        let ys = OwnedArray::linspace(0., 1., n);
        let u_array = {
            let mut u_array = xs.broadcast((n, n)).unwrap().to_owned();
            u_array.zip_mut_with(&ys.broadcast((n, n)).unwrap().t(),
                                 |x, y| *x = *x * gx + *y * gy + c);
            u_array
        };
        let u = u_array.as_slice().unwrap();

        let d = {
            let mut d = vec![0f64; n * n];
            signed_distance(&mut d, &u, (n, n), 1. / (n - 1) as f64);
            OwnedArray::from_shape_vec((n, n), d).unwrap()
        };
        if print {
            println!("{}", u_array);
            println!("{}", d);
        }
        d.all_close(&u_array, tol)
    }

    #[test]
    fn it_works_for_x_axis_line() {
        fn prop(y: f64) -> bool {
            check_line(0., 1., -((y - y.floor()) * 0.9 + 0.05), 9, 0.00001, false)
        }
        quickcheck(prop as fn(f64) -> bool);
    }

    #[test]
    fn it_works_for_y_axis_line() {
        fn prop(x: f64) -> bool {
            check_line(1., 0., -((x - x.floor()) * 0.9 + 0.05), 9, 0.00001, false)
        }
        quickcheck(prop as fn(f64) -> bool);
    }

    #[test]
    fn it_works_for_diagonal() {
        assert!(check_line((0.5f64).sqrt(),
                           (0.5f64).sqrt(),
                           -(0.5f64).sqrt(),
                           9,
                           1e-6,
                           false));
        assert!(check_line(-(0.5f64).sqrt(), (0.5f64).sqrt(), 0., 9, 1e-6, false));
    }

    #[test]
    fn it_preserves_lines() {
        fn prop(ta: f64) -> bool {
            let n = 17;
            let ta = (ta - ta.floor()) * 2. * ::std::f64::consts::PI;
            let (gy, gx) = ta.sin_cos();
            let c = -(gx + gy) * 0.5;

            let xs = OwnedArray::linspace(0., 1., n);
            let ys = OwnedArray::linspace(0., 1., n);
            let u_array = {
                let mut u_array = xs.broadcast((n, n)).unwrap().to_owned();
                u_array.zip_mut_with(&ys.broadcast((n, n)).unwrap().t(),
                                     |x, y| *x = *x * gx + *y * gy + c);
                u_array
            };
            let u = u_array.as_slice().unwrap();

            let d = {
                let mut d = vec![0f64; n * n];
                signed_distance(&mut d, &u, (n, n), 1. / (n - 1) as f64);
                OwnedArray::from_shape_vec((n, n), d).unwrap()
            };
            let d2 = {
                let mut d2 = vec![0f64; n * n];
                signed_distance(&mut d2, d.as_slice().unwrap(), (n, n), 1. / (n - 1) as f64);
                OwnedArray::from_shape_vec((n, n), d2).unwrap()
            };
            // check only elements away from the boundary
            let s = &[Si(2, Some(-2), 1), Si(2, Some(-2), 1)];
            d.slice(s).all_close(&d2.slice(s), 0.001)
        }
        quickcheck(prop as fn(f64) -> bool);
    }

}
