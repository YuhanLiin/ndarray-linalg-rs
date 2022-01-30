//! Cholesky decomposition of positive definite matrices

use crate::{triangular::IntoTriangular, LinalgError, Result};

use ndarray::{Array2, ArrayBase, Data, DataMut, Ix2};
use num_traits::{real::Real, NumAssignOps, NumRef};

/// Cholesky decomposition of a positive definite matrix
pub trait CholeskyInplace {
    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix in place.
    /// The upper triangle portion is not zeroed out.
    fn cholesky_inplace_dirty(&mut self) -> Result<&mut Self>;

    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix, passing by
    /// value.
    /// The upper triangle portion is not zeroed out.
    fn cholesky_into_dirty(mut self) -> Result<Self>
    where
        Self: Sized,
    {
        self.cholesky_inplace_dirty()?;
        Ok(self)
    }

    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix in place.
    fn cholesky_inplace(&mut self) -> Result<&mut Self>;

    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix, passing by
    /// value.
    fn cholesky_into(mut self) -> Result<Self>
    where
        Self: Sized,
    {
        self.cholesky_inplace()?;
        Ok(self)
    }
}

impl<A, S> CholeskyInplace for ArrayBase<S, Ix2>
where
    A: Real + NumRef + NumAssignOps,
    S: DataMut<Elem = A>,
{
    fn cholesky_inplace_dirty(&mut self) -> Result<&mut Self> {
        let m = self.nrows();
        let n = self.ncols();
        if m != n {
            return Err(LinalgError::NotSquare { rows: m, cols: n });
        }

        // TODO change accesses to uget and uget_mut
        for j in 0..n {
            let mut d = A::zero();
            for k in 0..j {
                let mut s = A::zero();
                for i in 0..k {
                    s += *self.get((k, i)).unwrap() * *self.get((j, i)).unwrap();
                }
                s = (*self.get((j, k)).unwrap() - s) / self.get((k, k)).unwrap();
                *self.get_mut((j, k)).unwrap() = s;
                d += s * s;
            }
            d = *self.get((j, j)).unwrap() - d;

            if d < A::zero() {
                return Err(LinalgError::NotPositiveDefinite);
            }

            *self.get_mut((j, j)).unwrap() = d.sqrt();
        }
        Ok(self)
    }

    fn cholesky_inplace(&mut self) -> Result<&mut Self> {
        self.cholesky_inplace_dirty()?;
        self.into_lower_triangular()?;
        Ok(self)
    }
}

/// Cholesky decomposition of a positive definite matrix
pub trait Cholesky {
    type Output;

    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix without modifying
    /// or consuming the original.
    /// The upper triangle portion is not zeroed out.
    fn cholesky_dirty(&self) -> Result<Self::Output>;

    /// Computes decomposition `A = L * L.t` where L is a lower-triangular matrix without modifying
    /// or consuming the original.
    fn cholesky(&self) -> Result<Self::Output>;
}

impl<A, S> Cholesky for ArrayBase<S, Ix2>
where
    A: Real + NumRef + NumAssignOps,
    S: Data<Elem = A>,
{
    type Output = Array2<A>;

    fn cholesky_dirty(&self) -> Result<Self::Output> {
        let arr = self.to_owned();
        arr.cholesky_into_dirty()
    }

    fn cholesky(&self) -> Result<Self::Output> {
        let arr = self.to_owned();
        arr.cholesky_into()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use ndarray::array;

    use super::*;

    #[test]
    fn decompose() {
        let arr = array![[25., 15., -5.], [15., 18., 0.], [-5., 0., 11.]];
        let lower = array![[5.0, 0.0, 0.0], [3.0, 3.0, 0.0], [-1., 1., 3.]];

        let chol = arr.cholesky().unwrap();
        assert_abs_diff_eq!(chol, lower, epsilon = 1e-4);
        assert_abs_diff_eq!(chol.dot(&chol.t()), arr, epsilon = 1e-4);
    }

    #[test]
    fn bad_matrix() {
        let row = array![[1., 2., 3.], [3., 4., 5.]];
        assert!(matches!(
            row.cholesky(),
            Err(LinalgError::NotSquare { rows: 2, cols: 3 })
        ));

        let non_pd = array![[1., 2.], [2., 1.]];
        let res = non_pd.cholesky_into();
        assert!(matches!(res, Err(LinalgError::NotPositiveDefinite)));
    }

    #[test]
    fn corner_cases() {
        let empty = Array2::<f64>::zeros((0, 0));
        assert_eq!(empty.cholesky().unwrap(), empty);

        let one = array![[1.]];
        assert_eq!(one.cholesky().unwrap(), one);
    }
}
