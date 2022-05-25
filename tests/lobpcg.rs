use approx::assert_abs_diff_eq;
use ndarray::prelude::*;
use ndarray_rand::{rand_distr::StandardNormal, RandomExt};
use proptest::prelude::*;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

use ndarray_linalg_rs::eigh::*;
use ndarray_linalg_rs::lobpcg::*;

mod common;

/// Eigenvalue structure in high dimensions
///
/// This test checks that the eigenvalues are following the Marchensko-Pastur law. The data is
/// standard uniformly distributed (i.e. E(x) = 0, E^2(x) = 1) and we have twice the amount of
/// data when compared to features. The probability density of the eigenvalues should then follow
/// a special densitiy function, described by the Marchenko-Pastur law.
///
/// See also https://en.wikipedia.org/wiki/Marchenko%E2%80%93Pastur_distribution
#[test]
fn test_marchenko_pastur() {
    // create random number generator
    let mut rng = Xoshiro256Plus::seed_from_u64(3);

    // generate normal distribution random data with N >> p
    let data = Array2::random_using((1000, 500), StandardNormal, &mut rng) / 1000f64.sqrt();

    let res = TruncatedSvd::new_with_rng(data, Order::Largest, Xoshiro256Plus::seed_from_u64(42))
        .precision(1e-3)
        .decompose(500)
        .unwrap();

    let sv = res.values().mapv(|x: f64| x * x);

    // we have created a random spectrum and can apply the Marchenko-Pastur law
    // with variance 1 and p/n = 0.5
    let (a, b) = (
        1. * (1. - 0.5f64.sqrt()).powf(2.0),
        1. * (1. + 0.5f64.sqrt()).powf(2.0),
    );

    // check that the spectrum has correct boundaries
    assert_abs_diff_eq!(b, sv[0], epsilon = 0.1);
    assert_abs_diff_eq!(a, sv[sv.len() - 1], epsilon = 0.1);

    // estimate density empirical and compare with Marchenko-Pastur law
    let mut i = 0;
    'outer: for th in Array1::linspace(0.1f64, 2.8, 28).slice(s![..;-1]) {
        let mut count = 0;
        while sv[i] >= *th {
            count += 1;
            i += 1;

            if i == sv.len() {
                break 'outer;
            }
        }

        let x = th + 0.05;
        let mp_law = ((b - x) * (x - a)).sqrt() / std::f64::consts::PI / x;
        let empirical = count as f64 / 500. / ((2.8 - 0.1) / 28.);

        assert_abs_diff_eq!(mp_law, empirical, epsilon = 0.05);
    }
}

fn run_lobpcg_eig_test(arr: Array2<f64>, num: usize, ordering: Order) {
    let (eigvals, _) = arr.eigh().unwrap().sort_eig(ordering == Order::Largest);
    let res = TruncatedEig::new_with_rng(arr.clone(), ordering, Xoshiro256Plus::seed_from_u64(42))
        .precision(1e-3)
        .decompose(num)
        .unwrap_or_else(|e| e.1.unwrap());

    assert_abs_diff_eq!(eigvals.slice(s![..num]), res.eigvals, epsilon = 1e-5);
    common::check_eigh(&arr, &res.eigvals, &res.eigvecs);
}

fn generate_order() -> impl Strategy<Value = Order> {
    prop_oneof![Just(Order::Largest), Just(Order::Smallest)]
}

prop_compose! {
    pub fn hpd_arr_num()(arr in common::hpd_arr())
        (num in (1..arr.ncols()), arr in Just(arr)) -> (Array2<f64>, usize) {
        (arr, num)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn lobpcg_eig_test((arr, num) in hpd_arr_num(), ordering in generate_order()) {
        run_lobpcg_eig_test(arr, num, ordering);
    }
}

#[test]
fn problematic_eig_matrix() {
    let arr = array![
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 7854.796948298437, 2495.5155877621937],
        [0.0, 0.0, 2495.5155877621937, 5995.696530257453]
    ];
    run_lobpcg_eig_test(arr, 3, Order::Largest);
}
