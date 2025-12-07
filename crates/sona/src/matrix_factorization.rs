//! Matrix Factorization using Alternating Least Squares (ALS)
//!
//! Implements ALS algorithm for collaborative filtering with implicit feedback.
//! Decomposes user-item interaction matrix into user and item latent factors.

use anyhow::{Context, Result};
use ndarray::{Array1, Array2};
use std::collections::HashMap;
use uuid::Uuid;

/// ALS configuration parameters
#[derive(Debug, Clone)]
pub struct ALSConfig {
    /// Number of latent factors (embedding dimension)
    pub latent_factors: usize,
    /// Regularization parameter (lambda)
    pub regularization: f32,
    /// Number of iterations
    pub iterations: usize,
    /// Confidence scaling for implicit feedback
    pub alpha: f32,
}

impl Default for ALSConfig {
    fn default() -> Self {
        Self {
            latent_factors: 64,
            regularization: 0.1,
            iterations: 10,
            alpha: 40.0,
        }
    }
}

/// Sparse user-item interaction matrix
#[derive(Debug, Clone)]
pub struct SparseMatrix {
    /// (user_index, item_index) -> rating
    pub entries: HashMap<(usize, usize), f32>,
    pub num_users: usize,
    pub num_items: usize,
}

impl SparseMatrix {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            num_users: 0,
            num_items: 0,
        }
    }

    pub fn insert(&mut self, user_idx: usize, item_idx: usize, value: f32) {
        self.entries.insert((user_idx, item_idx), value);
        self.num_users = self.num_users.max(user_idx + 1);
        self.num_items = self.num_items.max(item_idx + 1);
    }

    pub fn get(&self, user_idx: usize, item_idx: usize) -> f32 {
        *self.entries.get(&(user_idx, item_idx)).unwrap_or(&0.0)
    }
}

impl Default for SparseMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// ALS-based matrix factorization
pub struct MatrixFactorization {
    config: ALSConfig,
    /// User latent factors: [num_users x latent_factors]
    pub user_factors: Option<Array2<f32>>,
    /// Item latent factors: [num_items x latent_factors]
    pub item_factors: Option<Array2<f32>>,
    /// User ID to matrix index mapping
    pub user_id_map: HashMap<Uuid, usize>,
    /// Item ID to matrix index mapping
    pub item_id_map: HashMap<Uuid, usize>,
    /// Reverse mapping: index to user ID
    pub user_index_map: HashMap<usize, Uuid>,
    /// Reverse mapping: index to item ID
    pub item_index_map: HashMap<usize, Uuid>,
}

impl MatrixFactorization {
    pub fn new(config: ALSConfig) -> Self {
        Self {
            config,
            user_factors: None,
            item_factors: None,
            user_id_map: HashMap::new(),
            item_id_map: HashMap::new(),
            user_index_map: HashMap::new(),
            item_index_map: HashMap::new(),
        }
    }

    /// Build sparse matrix from user-item interactions
    pub fn build_matrix(&mut self, interactions: Vec<(Uuid, Uuid, f32)>) -> Result<SparseMatrix> {
        let mut matrix = SparseMatrix::new();
        self.user_id_map.clear();
        self.item_id_map.clear();
        self.user_index_map.clear();
        self.item_index_map.clear();

        let mut user_counter = 0;
        let mut item_counter = 0;

        for (user_id, item_id, rating) in interactions {
            // Map user ID to index
            let user_idx = *self.user_id_map.entry(user_id).or_insert_with(|| {
                let idx = user_counter;
                self.user_index_map.insert(idx, user_id);
                user_counter += 1;
                idx
            });

            // Map item ID to index
            let item_idx = *self.item_id_map.entry(item_id).or_insert_with(|| {
                let idx = item_counter;
                self.item_index_map.insert(idx, item_id);
                item_counter += 1;
                idx
            });

            matrix.insert(user_idx, item_idx, rating);
        }

        Ok(matrix)
    }

    /// Train ALS model on sparse matrix
    pub fn fit(&mut self, matrix: &SparseMatrix) -> Result<()> {
        let k = self.config.latent_factors;
        let lambda = self.config.regularization as f64;
        let alpha = self.config.alpha;

        // Initialize user and item factors randomly
        let mut user_factors = Array2::<f32>::zeros((matrix.num_users, k));
        let mut item_factors = Array2::<f32>::zeros((matrix.num_items, k));

        // Random initialization
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for i in 0..matrix.num_users {
            for j in 0..k {
                user_factors[[i, j]] = rng.gen_range(-0.1..0.1);
            }
        }
        for i in 0..matrix.num_items {
            for j in 0..k {
                item_factors[[i, j]] = rng.gen_range(-0.1..0.1);
            }
        }

        // ALS iterations
        for iteration in 0..self.config.iterations {
            // Update user factors
            for u in 0..matrix.num_users {
                let user_items: Vec<(usize, f32)> = matrix
                    .entries
                    .iter()
                    .filter(|((user_idx, _), _)| *user_idx == u)
                    .map(|((_, item_idx), &rating)| (*item_idx, rating))
                    .collect();

                if !user_items.is_empty() {
                    user_factors.row_mut(u).assign(&self.solve_user(
                        u,
                        &user_items,
                        &item_factors,
                        lambda,
                        alpha,
                    )?);
                }
            }

            // Update item factors
            for i in 0..matrix.num_items {
                let item_users: Vec<(usize, f32)> = matrix
                    .entries
                    .iter()
                    .filter(|((_, item_idx), _)| *item_idx == i)
                    .map(|((user_idx, _), &rating)| (*user_idx, rating))
                    .collect();

                if !item_users.is_empty() {
                    item_factors.row_mut(i).assign(&self.solve_item(
                        i,
                        &item_users,
                        &user_factors,
                        lambda,
                        alpha,
                    )?);
                }
            }

            if iteration % 2 == 0 {
                let loss = self.compute_loss(&matrix, &user_factors, &item_factors);
                tracing::debug!("ALS iteration {}: loss = {:.4}", iteration, loss);
            }
        }

        self.user_factors = Some(user_factors);
        self.item_factors = Some(item_factors);

        Ok(())
    }

    /// Solve least squares system A * x = b using Cholesky decomposition
    /// For positive definite matrix A (which we guarantee by adding regularization)
    fn solve_least_squares(a: &Array2<f64>, b: &Array1<f64>) -> Result<Array1<f64>> {
        let n = a.nrows();

        // Perform Cholesky decomposition: A = L * L^T
        let mut l = Array2::<f64>::zeros((n, n));

        for i in 0..n {
            for j in 0..=i {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[[i, k]] * l[[j, k]];
                }

                if i == j {
                    let diag = a[[i, i]] - sum;
                    if diag <= 0.0 {
                        anyhow::bail!("Matrix is not positive definite");
                    }
                    l[[i, j]] = diag.sqrt();
                } else {
                    l[[i, j]] = (a[[i, j]] - sum) / l[[j, j]];
                }
            }
        }

        // Forward substitution: L * y = b
        let mut y = Array1::<f64>::zeros(n);
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..i {
                sum += l[[i, j]] * y[j];
            }
            y[i] = (b[i] - sum) / l[[i, i]];
        }

        // Backward substitution: L^T * x = y
        let mut x = Array1::<f64>::zeros(n);
        for i in (0..n).rev() {
            let mut sum = 0.0;
            for j in (i + 1)..n {
                sum += l[[j, i]] * x[j];
            }
            x[i] = (y[i] - sum) / l[[i, i]];
        }

        Ok(x)
    }

    /// Solve for user factors (least squares)
    fn solve_user(
        &self,
        _user_idx: usize,
        user_items: &[(usize, f32)],
        item_factors: &Array2<f32>,
        lambda: f64,
        alpha: f32,
    ) -> Result<Array1<f32>> {
        let k = self.config.latent_factors;
        let mut a = Array2::<f64>::zeros((k, k));
        let mut b = Array1::<f64>::zeros(k);

        // Build A and b for least squares: A * x = b
        for (item_idx, rating) in user_items {
            let item_vec = item_factors.row(*item_idx);
            let confidence = 1.0 + alpha * rating;

            // A += confidence * item_vec^T * item_vec
            for i in 0..k {
                for j in 0..k {
                    a[[i, j]] += (confidence * item_vec[i] * item_vec[j]) as f64;
                }
            }

            // b += confidence * rating * item_vec
            for i in 0..k {
                b[i] += (confidence * rating * item_vec[i]) as f64;
            }
        }

        // Add regularization: A += lambda * I
        for i in 0..k {
            a[[i, i]] += lambda;
        }

        // Solve A * x = b using least squares
        let x = Self::solve_least_squares(&a, &b)
            .context("Failed to solve least squares for user factors")?;

        Ok(x.mapv(|v| v as f32))
    }

    /// Solve for item factors (least squares)
    fn solve_item(
        &self,
        _item_idx: usize,
        item_users: &[(usize, f32)],
        user_factors: &Array2<f32>,
        lambda: f64,
        alpha: f32,
    ) -> Result<Array1<f32>> {
        let k = self.config.latent_factors;
        let mut a = Array2::<f64>::zeros((k, k));
        let mut b = Array1::<f64>::zeros(k);

        // Build A and b for least squares: A * x = b
        for (user_idx, rating) in item_users {
            let user_vec = user_factors.row(*user_idx);
            let confidence = 1.0 + alpha * rating;

            // A += confidence * user_vec^T * user_vec
            for i in 0..k {
                for j in 0..k {
                    a[[i, j]] += (confidence * user_vec[i] * user_vec[j]) as f64;
                }
            }

            // b += confidence * rating * user_vec
            for i in 0..k {
                b[i] += (confidence * rating * user_vec[i]) as f64;
            }
        }

        // Add regularization: A += lambda * I
        for i in 0..k {
            a[[i, i]] += lambda;
        }

        // Solve A * x = b using least squares
        let x = Self::solve_least_squares(&a, &b)
            .context("Failed to solve least squares for item factors")?;

        Ok(x.mapv(|v| v as f32))
    }

    /// Compute reconstruction loss
    fn compute_loss(
        &self,
        matrix: &SparseMatrix,
        user_factors: &Array2<f32>,
        item_factors: &Array2<f32>,
    ) -> f32 {
        let mut loss = 0.0;
        let mut count = 0;

        for ((u, i), &rating) in &matrix.entries {
            let prediction = user_factors.row(*u).dot(&item_factors.row(*i));
            loss += (rating - prediction).powi(2);
            count += 1;
        }

        if count > 0 {
            loss / count as f32
        } else {
            0.0
        }
    }

    /// Predict rating for user-item pair
    pub fn predict(&self, user_id: Uuid, item_id: Uuid) -> Result<f32> {
        let user_idx = self
            .user_id_map
            .get(&user_id)
            .context("User not in training set")?;
        let item_idx = self
            .item_id_map
            .get(&item_id)
            .context("Item not in training set")?;

        let user_factors = self
            .user_factors
            .as_ref()
            .context("Model not trained yet")?;
        let item_factors = self
            .item_factors
            .as_ref()
            .context("Model not trained yet")?;

        Ok(user_factors
            .row(*user_idx)
            .dot(&item_factors.row(*item_idx)))
    }

    /// Get user embedding
    pub fn get_user_embedding(&self, user_id: Uuid) -> Result<Vec<f32>> {
        let user_idx = self
            .user_id_map
            .get(&user_id)
            .context("User not in training set")?;

        let user_factors = self
            .user_factors
            .as_ref()
            .context("Model not trained yet")?;

        Ok(user_factors.row(*user_idx).to_vec())
    }

    /// Get item embedding
    pub fn get_item_embedding(&self, item_id: Uuid) -> Result<Vec<f32>> {
        let item_idx = self
            .item_id_map
            .get(&item_id)
            .context("Item not in training set")?;

        let item_factors = self
            .item_factors
            .as_ref()
            .context("Model not trained yet")?;

        Ok(item_factors.row(*item_idx).to_vec())
    }

    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_matrix() {
        let mut matrix = SparseMatrix::new();
        matrix.insert(0, 0, 1.0);
        matrix.insert(0, 1, 2.0);
        matrix.insert(1, 0, 3.0);

        assert_eq!(matrix.num_users, 2);
        assert_eq!(matrix.num_items, 2);
        assert_eq!(matrix.get(0, 0), 1.0);
        assert_eq!(matrix.get(0, 1), 2.0);
        assert_eq!(matrix.get(1, 0), 3.0);
        assert_eq!(matrix.get(1, 1), 0.0);
    }

    #[test]
    fn test_build_matrix() {
        let mut mf = MatrixFactorization::new(ALSConfig::default());
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let item1 = Uuid::new_v4();
        let item2 = Uuid::new_v4();

        let interactions = vec![
            (user1, item1, 1.0),
            (user1, item2, 2.0),
            (user2, item1, 3.0),
        ];

        let matrix = mf.build_matrix(interactions).unwrap();

        assert_eq!(matrix.num_users, 2);
        assert_eq!(matrix.num_items, 2);
        assert_eq!(mf.user_id_map.len(), 2);
        assert_eq!(mf.item_id_map.len(), 2);
    }

    #[test]
    fn test_als_fit() {
        let mut mf = MatrixFactorization::new(ALSConfig {
            latent_factors: 4,
            regularization: 0.1,
            iterations: 5,
            alpha: 1.0,
        });

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let item1 = Uuid::new_v4();
        let item2 = Uuid::new_v4();

        let interactions = vec![
            (user1, item1, 1.0),
            (user1, item2, 1.0),
            (user2, item1, 1.0),
        ];

        let matrix = mf.build_matrix(interactions).unwrap();
        let result = mf.fit(&matrix);

        assert!(result.is_ok());
        assert!(mf.user_factors.is_some());
        assert!(mf.item_factors.is_some());

        let user_factors = mf.user_factors.as_ref().unwrap();
        let item_factors = mf.item_factors.as_ref().unwrap();

        assert_eq!(user_factors.nrows(), 2);
        assert_eq!(user_factors.ncols(), 4);
        assert_eq!(item_factors.nrows(), 2);
        assert_eq!(item_factors.ncols(), 4);
    }

    #[test]
    fn test_predict() {
        let mut mf = MatrixFactorization::new(ALSConfig {
            latent_factors: 8,
            regularization: 0.1,
            iterations: 10,
            alpha: 40.0,
        });

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let item1 = Uuid::new_v4();
        let item2 = Uuid::new_v4();
        let item3 = Uuid::new_v4();

        let interactions = vec![
            (user1, item1, 1.0),
            (user1, item2, 1.0),
            (user2, item1, 1.0),
            (user2, item3, 1.0),
        ];

        let matrix = mf.build_matrix(interactions).unwrap();
        mf.fit(&matrix).unwrap();

        // Predict known interactions
        let pred1 = mf.predict(user1, item1).unwrap();
        assert!(pred1 > 0.0);

        // Predict unseen interaction
        let pred2 = mf.predict(user1, item3).unwrap();
        assert!(pred2.abs() < 10.0); // Should be reasonable
    }

    #[test]
    fn test_get_embeddings() {
        let mut mf = MatrixFactorization::new(ALSConfig {
            latent_factors: 4,
            regularization: 0.1,
            iterations: 5,
            alpha: 1.0,
        });

        let user1 = Uuid::new_v4();
        let item1 = Uuid::new_v4();

        let interactions = vec![(user1, item1, 1.0)];

        let matrix = mf.build_matrix(interactions).unwrap();
        mf.fit(&matrix).unwrap();

        let user_emb = mf.get_user_embedding(user1).unwrap();
        let item_emb = mf.get_item_embedding(item1).unwrap();

        assert_eq!(user_emb.len(), 4);
        assert_eq!(item_emb.len(), 4);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((MatrixFactorization::cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((MatrixFactorization::cosine_similarity(&c, &d) - 0.0).abs() < 1e-6);

        let e = vec![1.0, 1.0, 0.0];
        let f = vec![1.0, 1.0, 0.0];
        assert!((MatrixFactorization::cosine_similarity(&e, &f) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_unknown_user_item() {
        let mut mf = MatrixFactorization::new(ALSConfig::default());
        let user1 = Uuid::new_v4();
        let item1 = Uuid::new_v4();

        let interactions = vec![(user1, item1, 1.0)];

        let matrix = mf.build_matrix(interactions).unwrap();
        mf.fit(&matrix).unwrap();

        let unknown_user = Uuid::new_v4();
        let result = mf.predict(unknown_user, item1);
        assert!(result.is_err());

        let unknown_item = Uuid::new_v4();
        let result = mf.predict(user1, unknown_item);
        assert!(result.is_err());
    }
}
