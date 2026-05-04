use crate::date::Date;

/// Simple natural cubic spline implementation for 1D interpolation.
pub struct NaturalCubicSpline {
    xs: Vec<f64>,
    ys: Vec<f64>,
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
}

impl NaturalCubicSpline {
    /// Build a natural cubic spline from sorted x and y arrays. Returns None if invalid.
    pub fn from_points(xs: &[f64], ys: &[f64]) -> Option<Self> {
        let n = xs.len();
        if n < 2 || ys.len() != n {
            return None;
        }

        if !xs.iter().all(|v| v.is_finite()) || !ys.iter().all(|v| v.is_finite()) {
            return None;
        }

        if n == 2 {
            let h = xs[1] - xs[0];
            if h <= 0.0 {
                return None;
            }
            let b = vec![(ys[1] - ys[0]) / h];
            return Some(NaturalCubicSpline {
                xs: xs.to_vec(),
                ys: ys.to_vec(),
                b,
                c: vec![0.0],
                d: vec![0.0],
            });
        }

        let mut h = vec![0.0; n - 1];
        for i in 0..n - 1 {
            let step = xs[i + 1] - xs[i];
            if step <= 0.0 {
                return None;
            }
            h[i] = step;
        }

        let mut alpha = vec![0.0; n];
        for i in 1..n - 1 {
            alpha[i] = (3.0 / h[i]) * (ys[i + 1] - ys[i]) - (3.0 / h[i - 1]) * (ys[i] - ys[i - 1]);
        }

        let mut l = vec![0.0; n];
        let mut mu = vec![0.0; n];
        let mut z = vec![0.0; n];

        l[0] = 1.0;
        mu[0] = 0.0;
        z[0] = 0.0;

        for i in 1..n - 1 {
            l[i] = 2.0 * (xs[i + 1] - xs[i - 1]) - h[i - 1] * mu[i - 1];
            if l[i].abs() < f64::EPSILON {
                return None;
            }
            mu[i] = h[i] / l[i];
            z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
        }

        l[n - 1] = 1.0;
        z[n - 1] = 0.0;

        let mut c = vec![0.0; n];
        let mut b = vec![0.0; n - 1];
        let mut d = vec![0.0; n - 1];

        for j in (0..n - 1).rev() {
            c[j] = z[j] - mu[j] * c[j + 1];
            b[j] = (ys[j + 1] - ys[j]) / h[j] - h[j] * (c[j + 1] + 2.0 * c[j]) / 3.0;
            d[j] = (c[j + 1] - c[j]) / (3.0 * h[j]);
        }

        Some(NaturalCubicSpline {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            b,
            c: c.into_iter().take(n - 1).collect(),
            d,
        })
    }

    /// Evaluate the spline at given x. Returns None when x is outside the interpolation range.
    pub fn evaluate(&self, x: f64) -> Option<f64> {
        if x.is_nan() {
            return None;
        }

        let n = self.xs.len();
        if n == 0 {
            return None;
        }

        let first = self.xs[0];
        let last = self.xs[n - 1];
        if x < first - 1e-9 || x > last + 1e-9 {
            return None;
        }

        let mut i = match self
            .xs
            .binary_search_by(|v| v.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Less))
        {
            Ok(idx) => idx,
            Err(idx) => {
                if idx == 0 {
                    0
                } else {
                    idx - 1
                }
            }
        };

        if i >= n - 1 {
            i = n - 2;
        }

        let dx = x - self.xs[i];
        let a = self.ys[i];
        Some(a + self.b[i] * dx + self.c[i] * dx * dx + self.d[i] * dx * dx * dx)
    }

    /// Helper: build spline from Date->value points (dates are converted to days since epoch)
    pub fn from_date_points(dates: &[Date], values: &[f64]) -> Option<Self> {
        let xs: Vec<f64> = dates.iter().map(|d| d.days_since_epoch() as f64).collect();
        Self::from_points(&xs, values)
    }
}

#[cfg(test)]
mod tests {
    use super::NaturalCubicSpline;
    use crate::date::Date;

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(diff < 1e-6, "expected {expected}, got {actual}");
    }

    #[test]
    fn from_points_rejects_invalid_inputs() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0];
        assert!(NaturalCubicSpline::from_points(&xs, &ys).is_none());

        let xs = [0.0, 0.0, 1.0];
        let ys = [0.0, 1.0, 2.0];
        assert!(NaturalCubicSpline::from_points(&xs, &ys).is_none());

        let xs = [0.0, 1.0, f64::NAN];
        let ys = [0.0, 1.0, 2.0];
        assert!(NaturalCubicSpline::from_points(&xs, &ys).is_none());
    }

    #[test]
    fn evaluate_returns_none_outside_range() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0, 0.0];
        let spline = NaturalCubicSpline::from_points(&xs, &ys).expect("spline");
        assert!(spline.evaluate(-0.5).is_none());
        assert!(spline.evaluate(2.5).is_none());
    }

    #[test]
    fn linear_two_point_spline_is_straight_line() {
        let xs = [2.0, 6.0];
        let ys = [10.0, 18.0];
        let spline = NaturalCubicSpline::from_points(&xs, &ys).expect("spline");
        let mid = spline.evaluate(4.0).expect("value");
        assert_close(mid, 14.0);
    }

    #[test]
    fn spline_hits_control_points() {
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [0.0, 2.0, 1.0, 3.0];
        let spline = NaturalCubicSpline::from_points(&xs, &ys).expect("spline");
        for (x, y) in xs.iter().zip(ys.iter()) {
            let v = spline.evaluate(*x).expect("value");
            assert_close(v, *y);
        }
    }

    #[test]
    fn from_date_points_matches_epoch_days() {
        let d1 = Date::new(2024, 1, 1);
        let d2 = Date::new(2024, 1, 8);
        let dates = [d1.clone(), d2.clone()];
        let values = [5.0, 12.0];
        let spline = NaturalCubicSpline::from_date_points(&dates, &values).expect("spline");
        let mid_day = (d1.days_since_epoch() + 3) as f64;
        let mid = spline.evaluate(mid_day).expect("value");
        assert_close(mid, 8.0);
    }
}
