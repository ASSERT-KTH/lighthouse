use risc0_zkvm::guest::env;
use safe_arith::SafeArith;
use nalgebra::Matrix2;

fn main() {
    // TODO: Implement your guest code here

    // read the input
    let mut input: u32 = env::read();



    input = lighthouse_sub(input);

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&input);
}


#[inline(never)]
fn lighthouse_sub(value: u32) -> u32 {
    let answer_1 = fibonacci_1(value);
    let answer_2 = fibonacci_2(value);
    let answer_3 = fibonacci_3(value);
    assert_eq!(answer_1, answer_2);
    assert_eq!(answer_1, answer_3);
    for i in 0..10 {
        value.safe_sub(1u32).expect("Failed to safely subtract 32 from input");
    }
    value
}



/// Implementation #1 of the nth Fibonacci number calculation.
///
/// This implementation a straight-forward, closely following the standard description of the
/// algorithm. It provides the baseline for comparison with any optimizations.
///
/// NOTE: Marked with #[inline(never)] to make sure this function is easy to see in the profile.
#[inline(never)]
pub fn fibonacci_1(n: u32) -> u64 {
    let (mut a, mut b) = (0, 1);
    if n <= 1 {
        return n as u64;
    }
    let mut i = 2;
    while i <= n {
        let c = a + b;
        a = b;
        b = c;
        i += 1;
    }
    b
}

/// Implementation #2 of the nth Fibonacci number calculation.
///
/// This implementation is equivalent to the first implementation, but has manual loop unrolling
/// applied. Instead of computing one number in the sequence per iteration, it computes 5 numbers.
/// This reduces the overhead of incrementing the loop counter relative to the useful work done.
///
/// NOTE: Marked with #[inline(never)] to make sure this function is easy to see in the profile.
#[inline(never)]
pub fn fibonacci_2(n: u32) -> u64 {
    let (mut a, mut b) = (0, 1);
    if n <= 1 {
        return n as u64;
    }
    let mut i = 2;
    while i <= n {
        if i + 5 <= n {
            let c = a + b;
            let d = b + c;
            let e = c + d;
            let f = d + e;
            let g = e + f;
            a = f;
            b = g;
            i += 5;
        } else {
            let c = a + b;
            a = b;
            b = c;
            i += 1;
        }
    }
    b
}

/// Implementation #3 of the nth Fibonacci number calculation.
///
/// This implementation leverages a translation of the problem of computing the nth Fibonacci
/// number to an equivalent matrix exponentiation problem. This allows use to use repeated
/// squaring, to calculate the nth Fibonacci number, requiring only log(n) steps. Instead of
/// implementing matrix multiplication ourselves, we import a linear algebra library.
///
/// NOTE: Marked with #[inline(never)] to make sure this function is easy to see in the profile.
#[inline(never)]
pub fn fibonacci_3(n: u32) -> u64 {
    Matrix2::new(1, 1, 1, 0).pow(n - 1)[(0, 0)]
}

