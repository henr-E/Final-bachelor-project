use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn gauss_seidel_iteration(
    a_: &Arc<Vec<Vec<f64>>>,
    b: &Arc<Vec<f64>>,
    x: &mut Vec<f64>,
    start: usize,
    end: usize,
) {
    for i in start..end {
        let mut sum = 0.0;
        for j in 0..a_.len() {
            if i != j {
                sum += a_[i][j] * x[j];
            }
        }
        let new_x = (b[i] - sum) / a_[i][i];
        x[i] = new_x;
    }
}

fn parallel_gauss_seidel(
    _a: Vec<Vec<f64>>,
    b: Vec<f64>,
    g: usize,
    max_iterations: i32,
    tolerance: f64,
) -> Option<Vec<f64>> {
    let m = _a.len() / g;
    let a = Arc::new(_a);
    let b = Arc::new(b);
    let x = Arc::new(Mutex::new(vec![0.0; b.len()]));
    let (sender, receiver) = mpsc::channel::<()>();
    let mut prev_x = vec![0.0; b.len()]; // To store the previous solution for convergence check

    for _ in 0..max_iterations {
        for i in 0..m {
            let sender_clone = sender.clone();
            let a_clone = Arc::clone(&a);
            let b_clone = Arc::clone(&b);
            let x_clone = Arc::clone(&x);
            thread::spawn(move || {
                let mut x_local = x_clone.lock().unwrap();
                let start = i * g;
                let end = std::cmp::min(start + g, b_clone.len());
                gauss_seidel_iteration(&a_clone, &b_clone, &mut x_local, start, end);

                sender_clone.send(()).unwrap();
            });
        }

        for _ in 0..m {
            receiver.recv().unwrap();
        }

        let current_x = x.lock().unwrap().clone();
        if prev_x
            .iter()
            .zip(current_x.iter())
            .all(|(prev, curr)| (curr - prev).abs() < tolerance)
        {
            println!("Convergence achieved");
            return Some(current_x);
        } else {
            prev_x = current_x;
        }
    }

    None
}
