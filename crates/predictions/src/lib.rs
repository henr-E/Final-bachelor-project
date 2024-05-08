use linfa_linalg::svd::SVD;
use ndarray::{self, s, Array1, Array2};
use ndarray_inverse::Inverse;
use serde::{Deserialize, Serialize};
use std::{cmp, f64::consts::PI, io, path::Path};

/// how much training data should be used. 0 < factor < 1
const TRAIN_DATA_SPLIT_SIZE: f64 = 0.95;
// usually order ends up ~60.
const MIN_ORDER: usize = 45;
const MAX_ORDER: usize = 128;

// calculates the inverse of the S vector, where S
// contains all singular values of a given matrix.
fn inverse_s(matrix: Array1<f64>) -> Array2<f64> {
    let mut matrix = matrix;
    matrix.map_mut(|item| {
        if *item != 0. {
            *item = 1. / *item;
        }
    });
    Array2::from_diag(&matrix)
}
fn moore_penrose_inv(matrix: &Array2<f64>) -> Option<Array2<f64>> {
    match matrix.t().dot(matrix).lu_inv() {
        Some(valid_inverse) => Some(valid_inverse.dot(matrix)),
        None => matrix
            .dot(&matrix.t())
            .lu_inv()
            .map(|valid_inverse| matrix.t().dot(&valid_inverse)),
    }
}

fn svd_inv(matrix: Array2<f64>) -> Option<Array2<f64>> {
    let svd = matrix.svd(true, true).ok()?;
    let u = svd.0?;
    let s = svd.1;
    let v_t = svd.2?;
    let s = inverse_s(s);
    let binding = v_t.t().dot(&s.t()).dot(&u.t());
    Some(binding)
}

/// Calculate the pseudo inverse of a given matrix.
///
/// The method used to use the SVD(U, S, V') of the matrix.
/// inverse = V * S' * U'.
///
/// The method first tries to calculates the Moore–Penrose inverse(for linearly independent columns).
/// https://en.wikipedia.org/wiki/Moore%E2%80%93Penrose_inverse#The_QR_method
fn pseudo_inverse(matrix: Array2<f64>) -> Option<Array2<f64>> {
    let item = moore_penrose_inv(&matrix);
    if item.is_some() {
        return item;
    }
    svd_inv(matrix)
}

fn first_order_differencing(input_data: &Array2<f64>) -> Array2<f64> {
    let mut z = Array2::<f64>::zeros((0, input_data.ncols()));
    for row_id in 0..input_data.nrows() - 1 {
        z.push_row((&(&input_data.row(row_id + 1) - &input_data.row(row_id))).into())
            .unwrap();
    }
    z
}
fn make_stationary(input_data: &Array2<f64>) -> Array2<f64> {
    // perform first order differencing.
    // row_i = row_i+1 - row_i
    first_order_differencing(input_data)
}

/// Calculates the corrected Akaike Information Criterion.
///
/// This is a scoring method to measure the model's performance.
/// n = number of observations.
/// k = number of paramerers in the model.
/// The lowest AICc is best.
// https://www.youtube.com/watch?v=-BR4WElPIXg
fn aicc(n: f64, k: f64, sse: f64) -> f64 {
    // the correction is for small sample sizes and large k's
    let correction = 2. * k + (2. * k * (k + 1.)) / (n - k - 1.);
    n * (sse / n).ln() + correction + n * (2.0 * PI).ln() + n
}

/// Calculates the sum of squared errors of 2 matrices.
fn sse(matrix1: &Array2<f64>, matrix2: &Array2<f64>) -> f64 {
    assert_eq!(matrix1.shape(), matrix2.shape());
    // difference and square all entries.
    (matrix1 - matrix2).map(|item| item.powf(2.)).sum()
}

fn fit_model(input_data: &Array2<f64>, order: usize) -> Option<VAR> {
    let last_entry = input_data.row(input_data.nrows() - 1).to_owned();
    let input_data = make_stationary(input_data);
    let last_data_entry = input_data
        .slice(s![input_data.shape()[0] - order.., ..])
        .to_owned();
    let observations = input_data.shape()[0];
    let variables = input_data.shape()[1];

    let k = order;
    let p = variables;
    let q = p * k + 1;
    let n = observations - k;
    let mut x = Array2::<f64>::zeros((n, q));
    let mut y = Array2::<f64>::zeros((0, p));

    assert_eq!(p, input_data.shape()[1]);

    for i in k..observations {
        let mut x_row = vec![1.];
        for j in 1..=k {
            let y_i = input_data.row(i - j);
            x_row.extend(y_i.iter());
        }
        let row = Array1::<f64>::from_shape_vec(q, x_row).ok()?;
        x.slice_mut(s![i - (k), ..]).assign(&row);
        // add i'th row to y.
        let y_i = input_data.row(i);
        y.push_row(y_i).ok()?;
    }
    assert_eq!(x.shape(), [n, q]);
    assert_eq!(y.shape(), [n, p]);
    // B^ = (X' * X)^-1 (X' * Y)
    let inverse = pseudo_inverse(x.t().dot(&x))?;
    let b_hat = inverse.dot(&x.t().dot(&y));
    // let residuals = y - x.dot(&b_hat);

    Some(VAR {
        coefficients: b_hat,
        x_t_minus_1: last_data_entry,
        last_entry,
        variables,
        order,
    })
}
/// Split a dataset into training and testing datasets.
///
/// The following property should hold: 0 < factor < 1
fn test_split(input_data: &Array2<f64>, factor: f64) -> (Array2<f64>, Array2<f64>) {
    let amt_data = input_data.nrows() as f64;
    let amt_training: usize = (amt_data * factor) as usize;
    let amt_training = cmp::max(amt_training, (amt_data - 40.) as usize);
    (
        input_data.slice(s![..amt_training, ..]).into_owned(),
        input_data.slice(s![amt_training.., ..]).into_owned(),
    )
}

/// Vector Auto Regression model.
///
/// This model tries to find a relation between a variable amount of quantities/features.
/// To estimate the relationship it will use lagged/past values of these features.
///
/// # Input data:
/// Usualy, the input data to this model must be [`stationary`](https://en.wikipedia.org/wiki/Stationary_process).
/// But the model tries to approximate such data by performing a `first_order_differencing`.
///
/// The input data must be in the form of a 2 dimentional matrix, [`ndarray::Array2`]. The amount
/// of rows indicate the total amount of `observations`. The amount of columns is equal to the
/// amount of `quantities/features`. The rows must be sorted from least recent to most recent
/// (row 0 is thus timestamp 0, ...).
///
/// # Example:
///
/// Take a look at the `vector_auto_regressor` example binary.
///
///# References:
/// Here is a list of resources used to implement the var model.
/// - [VAR wikipedia](https://en.wikipedia.org/wiki/Vector_autoregression)
/// - [Robust Estimation of the Vector Autoregressive Model by a Least Trimmed Squares procedure](https://lirias.kuleuven.be/retrieve/41362)
/// - [Vector autoregression models](https://kevinkotze.github.io/ts-7-var/)
/// - [Vector AutoRegressive Models](https://www.lem.sssup.it/phd/documents/Lesson17.pdf)
/// - [Vector Autoregression lecture ppt](https://www.fsb.miamioh.edu/lij14/672_s7.pdf)
/// - [Linear regression wikipedia](https://en.wikipedia.org/wiki/Linear_regression#Least-squares_estimation_and_related_techniques)
#[derive(Debug, Serialize, Deserialize)]
pub struct VAR {
    coefficients: Array2<f64>,
    x_t_minus_1: Array2<f64>,
    last_entry: Array1<f64>,
    variables: usize,
    order: usize,
}
// PERF: look at reusing the `make_stationary` method.
// This is unlikely to impact the overall performance much, since the SVD and matrix
// multiplications are more impactful.
impl VAR {
    /// Create a new auto regressor for an [`ndarray::Array2`].
    pub fn new(data: Vec<f64>, amt_variables: usize) -> Option<Self> {
        let input_data =
            Array2::from_shape_vec((data.len() / amt_variables, amt_variables), data).ok()?;
        let variables = input_data.ncols();
        let (train_data, test_data) = test_split(&input_data, TRAIN_DATA_SPLIT_SIZE);
        let mut best_order = MIN_ORDER;
        let mut best_score = f64::MAX;
        // try `order` between 4 and 74.
        // testing `order` has a O(mn^2) for svd, O(n^3) for each matrix multiplication.
        // thus testing larger `order` values will unlikely be fruitful.
        let max_possible_order = std::cmp::min(MAX_ORDER, train_data.shape()[0] - 1);
        for order in MIN_ORDER..max_possible_order {
            let mut model = fit_model(&train_data, order)?;
            let predictions = model.predict_n(test_data.nrows());
            let sse = sse(&predictions, &test_data);
            let aicc_score = aicc(train_data.nrows() as f64, variables as f64, sse);
            if aicc_score <= best_score {
                best_score = aicc_score;
                best_order = order;
            }
        }
        fit_model(&input_data, best_order)
    }
    /// Serialize the auto regressor to a given [`Path`].
    pub fn from_file(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        let var: VAR = bincode::deserialize(&bytes).ok()?;
        Some(var)
    }
    pub fn get_order(&self) -> usize {
        self.order
    }
    /// Deserialize an auto regressor from a given [`Path`].
    pub fn to_file(&self, path: &Path) -> io::Result<()> {
        let bytes = match bincode::serialize(&self) {
            Ok(bytes) => bytes,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        };
        std::fs::write(path, bytes)
    }
    fn timestamps_to_vec(&self, matrix: &Array2<f64>) -> Array1<f64> {
        let mut x_row = vec![1.];
        for row_id in (0..matrix.nrows()).rev() {
            x_row.extend(matrix.row(row_id).iter());
        }
        Array1::from_vec(x_row)
    }
    fn predict_next(&mut self) -> Array1<f64> {
        let xi = self.timestamps_to_vec(&self.x_t_minus_1);
        let yi = self.coefficients.t().dot(&xi);
        let mut new_matrix =
            Array2::<f64>::zeros((self.x_t_minus_1.shape()[0], self.x_t_minus_1.shape()[1]));
        for i in 0..self.order - 1 {
            for (j, value) in self.x_t_minus_1.row(i + 1).iter().enumerate() {
                new_matrix.row_mut(i)[j] = *value;
            }
        }
        for (i, item) in yi.iter().enumerate() {
            new_matrix.row_mut(self.order - 1)[i] = *item;
        }
        self.x_t_minus_1 = new_matrix;
        let result = yi + self.last_entry.view();
        self.last_entry = result.clone();
        result
    }
    fn predict_n(&mut self, n: usize) -> Array2<f64> {
        let mut output = Array2::zeros((0, self.variables));
        for _ in 0..n {
            let yi = self.predict_next();
            output.push_row(yi.view()).unwrap();
        }
        output
    }
    /// Predict the next timestep.
    pub fn get_next_prediction(&mut self) -> Vec<f64> {
        self.predict_next().to_vec()
    }
    /// Predict the next `n` timesteps.
    pub fn get_next_predictions(&mut self, n: usize) -> Vec<Vec<f64>> {
        self.predict_n(n)
            .rows()
            .into_iter()
            .map(|row| row.to_vec())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn matrix_eq(matrix1: &Array2<f64>, matrix2: &Array2<f64>) -> bool {
        (matrix1 - matrix2).iter().any(|i| *i < f64::EPSILON)
    }

    #[test]
    fn split_data_test() {
        let data = vec![
            5.5, 300.0, 4.0, 0.0, 0.0, 5.4, 300.0, 5.0, 0.0, 0.0, 5.0, 270.0, 5.0, 0.0, 0.0, 4.8,
            260.0, 7.0, 0.0, 0.0, 4.7, 270.0, 6.0, 0.0, 0.0, 5.0, 280.0, 7.0, 0.0, 0.0, 4.9, 280.0,
            4.0, 0.0, 0.0, 5.1, 280.0, 5.0, 0.0, 10.0, 5.3, 280.0, 4.0, 0.0, 39.0, 5.9, 300.0, 5.0,
            0.0, 121.0, 6.7, 320.0, 5.0, 0.0, 302.0, 6.6, 320.0, 4.0, 0.2, 285.0,
        ];
        let observations = data.len() / 5;
        let (train_data, test_data) = test_split(
            &Array2::from_shape_vec((observations, 5), data).unwrap(),
            0.85,
        );
        assert_eq!(train_data.nrows(), 10);
        assert_eq!(test_data.nrows(), 2);
        assert_eq!(test_data.nrows() + train_data.nrows(), observations);
    }

    #[test]
    fn pseudo_inverse_test() {
        let a = Array2::from_shape_vec(
            (4, 5),
            vec![
                1., 0., 0., 0., 2., 0., 0., 3., 0., 0., 0., 0., 0., 0., 0., 0., 4., 0., 0., 0.,
            ],
        )
        .unwrap();
        let a_inv = pseudo_inverse(a.clone()).unwrap();
        let expected_data = vec![
            0.2,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.25,
            0.,
            1. / 3.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.4,
            0.,
            0.,
            0.,
        ];
        let expected = Array2::from_shape_vec((5, 4), expected_data).unwrap();
        assert_eq!(a.t().shape(), a_inv.shape());
        assert_eq!(expected.shape(), a_inv.shape());
        assert!(matrix_eq(&a_inv, &expected));
    }
}
