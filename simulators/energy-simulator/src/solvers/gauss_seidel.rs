/// A graph solver that uses an iterative method to solve the graph
///only knowing the nodes and its neighbours
struct IterativeGraphSolver {
    callback: Box<dyn Fn(usize) -> (f64, Vec<usize>)>,
}

#[allow(non_snake_case)] // More intuitive variable name: capital for matrix
struct Solver {
    A: Vec<Vec<f64>>,
    y: Vec<f64>,
    ///takes a usize (node index) as an argument and returns a tuple of type (f64, Vec<usize>)
    ///representing the value of the node and a list of its neighbors.
    callback: Box<dyn Fn(usize) -> (f64, Vec<usize>)>,
}
impl Solver {
    #[allow(non_snake_case)]
    fn new(A: Vec<Vec<f64>>, y: Vec<f64>) -> Solver {
        Solver {
            A: A,
            y: y,
            callback: Box::new(|_| (0.0, vec![])),
        }
    }
    fn iter(callback: Box<dyn Fn(usize) -> (f64, Vec<usize>)>) -> Solver {
        Solver {
            A: vec![vec![]],
            y: vec![],
            callback: callback,
        }
    }
    fn solve(&self, max_iterations: i32, tolerance: f64) -> Option<Vec<f64>> {
        let mut phi = vec![f64::default(); self.A.len()];

        for _iteration in 0..max_iterations {
            let mut converged = true;

            for i in 0..self.A.len() {
                let mut sigma = 0.0;

                for j in 0..self.A.len() {
                    if j != i {
                        sigma += self.A[i][j] * phi[j];
                    }
                }

                let new_phi_i = (self.y[i] - sigma) / self.A[i][i];

                if (phi[i] - new_phi_i).abs() > tolerance {
                    converged = false;
                }

                phi[i] = new_phi_i;
            }

            if converged {
                return Some(phi);
            }
        }

        None // Convergence not reached within max_iterations
    }
}

pub(crate) struct DistributedSolver {
    adjacency_list: Vec<Vec<usize>>,
    node_values: Vec<f64>,
}

impl DistributedSolver {
    pub fn new(adjacency_list: Vec<Vec<usize>>, node_values: Vec<f64>) -> Self {
        DistributedSolver {
            adjacency_list,
            node_values,
        }
    }
    pub fn dist_solver(&mut self, max_iterations: i32, tolerance: f64) -> Option<Vec<f64>> {
        let num_nodes = self.node_values.len();
        let mut phi = vec![f64::default(); num_nodes];

        for _iteration in 0..max_iterations {
            let mut converged = true;

            for i in 0..num_nodes {
                let mut sigma = 0.0;

                for &j in &self.adjacency_list[i] {
                    sigma += phi[j];
                }

                let new_phi_i = (self.node_values[i] - sigma) / self.adjacency_list[i].len() as f64;

                if (phi[i] - new_phi_i).abs() > tolerance {
                    converged = false;
                }

                phi[i] = new_phi_i;
            }

            if converged {
                return Some(phi);
            }
        }

        None // Convergence not reached within max_iterations
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_solver() {
        //for gaus seidel to converge matrix should be diagonally dominant or symmetric positive definite
        // Example usage
        let matrix_a = vec![
            vec![40.0, 9.0, 2.0, 1.3],
            vec![3.0, 50.0, 1.0, 4.9],
            vec![1.0, 10.0, 30.0, 4.3],
            vec![3.0, 9.0, 1.0, 209.3],
        ];

        let vector_b = vec![1.0, 100.0, 9.0, 189.9];
        let max_iterations = 900;
        let tolerance = 1e-6;
        let solver = Solver::new(matrix_a, vector_b);
        let mut sol_found = false;
        if let Some(solution) = solver.solve(max_iterations, tolerance) {
            sol_found = true;
        } else {
            sol_found = false;
        }
        assert!(sol_found);
    }
    #[test]
    fn test_distributed_solver() {
        let adjacency_list = vec![vec![1, 3], vec![0, 2, 3], vec![1, 3], vec![0, 1, 2]];
        let node_values = vec![10.0, 10.0, 10.0, 10.0];
        let max_iterations = 20;
        let tolerance = 1e-6;
        let mut solver = DistributedSolver::new(adjacency_list, node_values);
        let mut sol_found = false;
        if let Some(solution) = solver.dist_solver(max_iterations, tolerance) {
            sol_found = true;
        } else {
            sol_found = false;
        }
        assert!(sol_found);
    }
}
