use std;

/// Computes the signed distance function from a plane given as the _zero_ level set of a
/// linear function on a tetrahedron at 4 points with unit coordinates starting at (0, 0, 0) and
/// ending at (1, 1, 1), and in between exactly one coordinate changes from 0 to 1.
///
/// Inputs are `u`, the values at the vertices.
///
/// The function returns the values of the (non-signed) distance function or `None` if the zero
/// level set does not pass through the tetrahedron.
pub fn tetrahedron_dist(u: [f64; 4]) -> Option<[f64; 4]> {
    let mut u = u;
    let tiny = 1e-15;

    let mut n_pos = 0;
    for u in &mut u {
        if *u >= 0. {
            *u += tiny;
            n_pos += 1;
        }
    }
    // check if sign differs (level set goes throught the tetrahedron
    if n_pos == 0 || n_pos == 4 {
        // premature optimization: n_pos & 3 == 0 :)
        return None;
    }

    let g_norm_rcp = 1. / u.windows(2).fold(0., |sum, x| sum + (x[1] - x[0]).powi(2)).sqrt();

    for u in u.iter_mut() {
        *u = u.abs() * g_norm_rcp;
    }
    Some(u)
}

/// Initializes the distance around the free boundary.
///
/// Based on the level set function with values `u` given on a regular grid, it computes the
/// distance from the _zero_ level set in the nodes of the triangles through which the level set
/// passes.  Stores the result in the preallocated slice `d`.
///
/// Nodes away from the boundary have their value set to `std::f64::MAX`.
///
/// Splits every square into two triangles and computes the distance on each of them.
pub fn init_dist_3d(d: &mut [f64], u: &[f64], dim: (usize, usize, usize)) {
    let (nx, ny, nz) = dim;
    assert_eq!(nx * ny * nz, u.len());
    assert_eq!(nx * ny * nz, d.len());

    for d in &mut *d {
        *d = std::f64::MAX;
    }

    // split each cube into 6 tetrahedrons
    let ids = [[(0, 0, 0), (1, 0, 0), (1, 1, 0), (1, 1, 1)],
               [(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 1, 1)],
               [(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 1, 1)],
               [(0, 0, 0), (0, 1, 0), (0, 1, 1), (1, 1, 1)],
               [(0, 0, 0), (0, 0, 1), (1, 0, 1), (1, 1, 1)],
               [(0, 0, 0), (0, 0, 1), (0, 1, 1), (1, 1, 1)]];

    for i in 1..nx {
        for j in 1..ny {
            for k in 1..nz {
                let s = i * ny * nz + j * nz + k;
                let mut v = [0.; 4];

                for idx in ids.iter() {
                    for m in 0..4 {
                        v[m] = u[s - idx[m].0 * ny * nz - idx[m].1 * nz - idx[m].2];
                    }

                    let r = tetrahedron_dist(v);
                    if let Some(r) = r {
                        for m in 0..4 {
                            let q = s - idx[m].0 * ny * nz - idx[m].1 * nz - idx[m].2;
                            d[q] = d[q].min(r[m]);
                        }
                    }
                }
            }
        }
    }
}
/// Computes the signed distance function from a line segment given as the _zero_ level set of a
/// linear function on an isosceles right triangle.
///
/// Inputs are `u`, the values at the vertices. The vertex 0 is the one with the right angle.
///
/// The function returns the values of the signed distance function or `None` if the zero level set
/// does not pass through the triangle.
pub fn triangle_dist(u: [f64; 3]) -> Option<[f64; 3]> {
    let mut u = u;
    // normalize so that u[0] >= 0.
    if u[0] < 0. {
        for u in &mut u {
            *u = -*u;
        }
    }

    // gradient vector
    let gx = u[1] - u[0];
    let gy = u[2] - u[0];
    let g_norm = (gx * gx + gy * gy).sqrt();

    if u[1] >= 0. {
        if u[2] >= 0. {
            // well isn't this ugly, we need to handle possible zeros
            match (u[0], u[1], u[2]) {
                (0., 0., 0.) => Some([0., 0., 0.]),
                (_, 0., 0.) => Some([(0.5f64).sqrt(), 0., 0.]),
                (0., _, 0.) => Some([0., 1., 0.]),
                (0., 0., _) => Some([0., 0., 1.]),
                (0., _, _) => Some([0., 1., 1.]),
                (_, 0., _) => Some([1., 0., (2f64).sqrt()]),
                (_, _, 0.) => Some([1., (2f64).sqrt(), 0.]),
                _ => None,
            }
        } else {
            // u[2] < 0.
            // intersect position
            let i02 = u[0] / (u[0] - u[2]);
            let i12 = (2f64).sqrt() * u[1] / (u[1] - u[2]);
            // find the direction of the gradient
            // to deduce the vertex that is closest to the line
            if gx <= 0. {
                // 0
                Some([u[0] / g_norm, i12, 1. - i02])
            } else if gx > -gy {
                // 1
                Some([i02, u[1] / g_norm, (2f64).sqrt() - i12])
            } else {
                // 2
                Some([i02, i12, -u[2] / g_norm])
            }
        }
    } else if u[2] >= 0. {
        // u[1] < 0.
        // intersect position
        let i01 = u[0] / (u[0] - u[1]);
        let i12 = (2f64).sqrt() * u[1] / (u[1] - u[2]);
        // find the direction of the gradient
        // to deduce the vertex that is closest to the line
        if gy <= 0. {
            // 0
            Some([u[0] / g_norm, 1. - i01, (2f64).sqrt() - i12])
        } else if -gx > gy {
            // 1
            Some([i01, -u[1] / g_norm, (2f64).sqrt() - i12])
        } else {
            // 2
            Some([i01, i12, u[2] / g_norm])
        }
    } else {
        // u[2] < 0.
        // intersect position
        let i10 = u[1] / (u[1] - u[0]);
        let i20 = u[2] / (u[2] - u[0]);

        Some([u[0] / g_norm, i10, i20])
    }

}

/// Initializes the distance around the free boundary.
///
/// Based on the level set function with values `u` given on a regular grid, it computes the
/// distance from the _zero_ level set in the nodes of the triangles through which the level set
/// passes.  Stores the result in the preallocated slice `d`.
///
/// Nodes away from the boundary have their value set to `std::f64::MAX`.
///
/// Splits every square into two triangles and computes the distance on each of them.
pub fn init_dist(d: &mut [f64], u: &[f64], dim: (usize, usize)) {
    let (nx, ny) = dim;
    assert_eq!(nx * ny, u.len());
    assert_eq!(nx * ny, d.len());

    for d in &mut *d {
        *d = std::f64::MAX;
    }

    for j in 1..ny {
        for i in 1..nx {
            let s = j * nx + i;
            let r = triangle_dist([u[s - nx - 1], u[s - nx], u[s - 1]]);
            if let Some(e) = r {
                d[s - nx - 1] = e[0].min(d[s - nx - 1]);
                d[s - nx] = e[1].min(d[s - nx]);
                d[s - 1] = e[2].min(d[s - 1]);
            }
            let r = triangle_dist([u[s], u[s - nx], u[s - 1]]);
            if let Some(e) = r {
                d[s] = e[0].min(d[s]);
                d[s - nx] = e[1].min(d[s - nx]);
                d[s - 1] = e[2].min(d[s - 1]);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_triangles() {
        assert_eq!(triangle_dist([0., 0., 0.]), Some([0., 0., 0.]));
        assert_eq!(triangle_dist([-1., 0., 0.]),
                   Some([(0.5f64).sqrt(), 0., 0.]));
        assert_eq!(triangle_dist([0., 1., 0.]), Some([0., 1., 0.]));
        assert_eq!(triangle_dist([0., -1., -1.]), Some([0., 1., 1.]));
        assert_eq!(triangle_dist([0., 1., 1.]), Some([0., 1., 1.]));
        assert_eq!(triangle_dist([1., 1., 0.]), Some([1., (2f64).sqrt(), 0.]));
    }
}
