use rand::rngs::OsRng;
use rand::seq::index;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rug::{Complete, Integer};
use std::cell::RefCell;

#[cfg(test)]
mod test;

thread_local! {
    /// Thread-local random number generator using ChaCha20, seeded from the operating system's RNG.
    static THREAD_RNG: RefCell<ChaCha20Rng> = {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        RefCell::new(ChaCha20Rng::from_seed(seed))
    };
}

/// Utility function to execute a closure with access to the thread-local random number generator.
fn with_thread_rng<T>(func: impl FnOnce(&mut ChaCha20Rng) -> T) -> T {
    THREAD_RNG.with(|cell| func(&mut cell.borrow_mut()))
}

/// Computes the ceiling of the base-2 logarithm of a given unsigned integer.
fn ceil_log2_u64(x: u64) -> u32 {
    if x == 0 {
        return 0;
    }
    (x - 1).next_power_of_two().trailing_zeros()
}

/// Computes the bit length of a given unsigned integer.
fn bit_length_u32(x: u32) -> u32 {
    u32::BITS - x.leading_zeros()
}

/// Compute floor(value^(1/n)) for unsigned integers using binary search.
fn nth_root_u64(value: u64, n: u32) -> u64 {
    if n == 0 {
        panic!("n must be greater than 0");
    }
    if value < 2 {
        return value;
    }
    let mut low = 1u64;
    let mut high = value;
    while low <= high {
        let mid = (low + high) / 2;
        match mid.checked_pow(n) {
            Some(pow) if pow == value => return mid,
            Some(pow) if pow < value => low = mid + 1,
            _ => high = mid - 1,
        }
    }
    high
}

/// Compute floor(value^(1/n)) for unsigned integers using binary search.
fn nth_root_u32(value: u32, n: u32) -> u32 {
    let result = nth_root_u64(value.into(), n);
    if result <= u32::MAX as u64 {
        result as u32
    } else {
        panic!("Result exceeds u32::MAX");
    }
}

/// Compute floor(sqrt(value)) for unsigned integers using binary search.
fn sqrt_u32(value: u32) -> u32 {
    nth_root_u32(value, 2)
}

/// Sample a random integer uniformly from the range [0, n).
fn sample_below(n: Integer) -> Integer {
    if n <= 0 {
        panic!("n must be greater than 0");
    }
    let bits = n.significant_bits();
    with_thread_rng(|rng| {
        let bytes_len = bits.div_ceil(8) as usize;
        let top_mask: u8 = if bits.is_multiple_of(8) {
            0xFF
        } else {
            (1 << (bits % 8)) - 1
        };
        let mut buf = vec![0u8; bytes_len];
        loop {
            rng.fill_bytes(&mut buf);
            if let Some(last) = buf.last_mut() {
                *last &= top_mask;
            }
            let val = Integer::from_digits(&buf, rug::integer::Order::Lsf);
            if val < n {
                return val;
            }
        }
    })
}

/// Generate a random odd integer between 2^(bits-1) and 2^bits - 1.
fn sample_odd_integer_with_bit_length(bits: u32) -> Integer {
    // Validate the bit length parameter.
    if bits < 2 {
        panic!("bits must be at least 2");
    }
    // Use the thread-local RNG.
    with_thread_rng(|rng| {
        // Sample random bits.
        let mut random_bytes = vec![0u8; bits.div_ceil(8) as usize];
        rng.fill_bytes(&mut random_bytes);
        // Set least significant bit to 1 to ensure the number is odd.
        if let Some(last_byte) = random_bytes.last_mut() {
            *last_byte |= 1;
        }
        // Set the most significant bit to ensure the number has the desired bit length.
        let msb_position = (bits - 1) % 8;
        random_bytes[0] |= 1 << msb_position;
        // Clear any bits above the target MSB to prevent the integer from exceeding desired length.
        random_bytes[0] &= ((1u16 << (msb_position + 1)) - 1) as u8;
        // Convert to a big integer using rug.
        Integer::from_digits(&random_bytes, rug::integer::Order::Msf)
    })
}

/// Sample an integer uniformly at random from the range [0, (2**bits) / divisor).
fn sample_bounded_integer_with_bit_length(bits: u64, divisor: &rug::Integer) -> Integer {
    if *divisor <= 0 {
        panic!("divisor must be greater than 0");
    }
    if divisor.is_even() {
        panic!("divisor must be odd");
    }
    if bits < 2 {
        panic!("bits must be at least 2");
    }
    let mut upper_bound: Integer = Integer::from(1) << (bits as usize);
    upper_bound += (divisor - 1i32).complete();
    upper_bound /= divisor;
    if upper_bound <= 0 {
        panic!("upper bound must be greater than 0");
    }
    with_thread_rng(|rng| {
        let target_bits = upper_bound.significant_bits();
        let bytes_needed = target_bits.div_ceil(8) as usize;
        let mut buf = vec![0u8; bytes_needed];
        let mut candidate = Integer::new();
        let bits_in_msb = if target_bits.is_multiple_of(8) {
            8
        } else {
            target_bits % 8
        };
        let msb_mask = ((1u16 << bits_in_msb) - 1) as u8;
        loop {
            rng.fill_bytes(&mut buf);
            if let Some(last) = buf.last_mut() {
                *last &= msb_mask;
            }
            candidate.assign_digits(&buf, rug::integer::Order::Lsf);
            if candidate < upper_bound {
                return candidate;
            }
        }
    })
}

/// Sample an integer from the open ball (-2**bits, 2**bits).
fn sample_integer_from_ball_with_bit_length(bits: u32) -> Integer {
    if bits < 2 {
        panic!("bits must be at least 2");
    }
    let total_bound: Integer = (Integer::from(1) << (bits.checked_add(1).unwrap())) - 1;
    with_thread_rng(|rng| {
        let target_bits = total_bound.significant_bits();
        let bytes_needed = target_bits.div_ceil(8) as usize;
        let mut buf = vec![0u8; bytes_needed];
        let mut candidate = Integer::new();
        let bits_in_msb = if target_bits.is_multiple_of(8) {
            8
        } else {
            target_bits % 8
        };
        let msb_mask = ((1u16 << bits_in_msb) - 1) as u8;
        loop {
            rng.fill_bytes(&mut buf);
            if let Some(last) = buf.last_mut() {
                *last &= msb_mask;
            }
            candidate.assign_digits(&buf, rug::integer::Order::Lsf);
            // Rejection sample to get a uniform value in the range [0, 2**(bits+1) - 2]
            if candidate < total_bound {
                // Shift the range to be centered around zero.
                let offset = (Integer::from(1) << bits) - 1;
                candidate -= offset;
                return candidate;
            }
        }
    })
}

/// Sample u_i in [0, modulus) with sum_{i in subset} u_i == target (mod modulus).
fn sample_constrained_integers(
    length: u64,
    subset: &[u64],
    modulus: Integer,
    target: Integer,
) -> Vec<Integer> {
    if modulus <= 0 {
        panic!("modulus must be greater than 0");
    }
    if subset.iter().any(|&idx| idx >= length) {
        panic!("subset indices must be within the range [0, length)");
    }
    if subset.is_empty() {
        panic!("subset must be non-empty");
    }
    if length == 0 {
        panic!("length must be greater than 0");
    }

    with_thread_rng(|rng| {
        let mut u: Vec<Integer> = Vec::with_capacity(length as usize);
        let bits = modulus.significant_bits();
        let bytes_len = bits.div_ceil(8) as usize;
        let top_mask: u8 = if bits.is_multiple_of(8) {
            0xFF
        } else {
            (1 << (bits % 8)) - 1
        };
        for _ in 0..length {
            let mut buf = vec![0u8; bytes_len];
            loop {
                rng.fill_bytes(&mut buf);
                // Mask the most significant byte to match bit-length, dropping the rejection rate.
                if let Some(last) = buf.last_mut() {
                    *last &= top_mask;
                }

                // Construct rug::Integer from little-endian bytes.
                let val = Integer::from_digits(&buf, rug::integer::Order::Lsf);
                if val < modulus {
                    u.push(val);
                    break;
                }
            }
        }
        let adjust_index = *subset.last().unwrap() as usize;
        let mut current_sum = Integer::from(0);
        for &idx in subset {
            current_sum += &u[idx as usize];
        }
        current_sum %= &modulus;
        let mut diff = target - current_sum;
        diff %= &modulus;
        if diff < 0 {
            diff += &modulus;
        }
        u[adjust_index] += diff;
        u[adjust_index] %= &modulus;

        u
    })
}

/// Sample a non-empty random subset of the set {0, 1, ..., n-1}.
fn sample_subset(n: u64) -> Vec<u64> {
    if n == 0 {
        panic!("n must be greater than 0");
    }
    let subset_size: Integer = sample_below((Integer::from(1) << n as usize) - 1) + 1;
    (0..n)
        .filter(|i| {
            subset_size.get_bit(*i as u32)
        })
        .collect::<Vec<u64>>()
}

/// Compute the quotient of a and b, rounding to the nearest integer.
fn quotient(numerator: Integer, denominator: Integer) -> Integer {
    if denominator.is_zero() {
        panic!("denominator must be greater than 0");
    }
    let sign = if numerator.is_negative() == denominator.is_negative() {
        1
    } else {
        -1
    };
    let n = numerator.abs();
    let d = denominator.abs();
    let whole = (&n / &d).complete();
    let remainder = (&n % &d).complete();
    let twice_remainder = (&remainder * 2i32).complete();
    let rounded = if twice_remainder < d {
        whole
    } else if twice_remainder > d {
        whole + 1
    } else if whole.is_even() {
        whole
    } else {
        whole + 1
    };
    rounded * sign
}

/// Compute the remainder a - q*b where q is quotient(a, b).
fn remainder(numerator: Integer, denominator: Integer) -> Integer {
    if denominator.is_zero() {
        panic!("denominator must be greater than 0");
    }
    let q = quotient(numerator.clone(), denominator.clone());
    numerator - q * denominator
}

/// Sample a random bit vector of the given length with the specified Hamming weight.
/// Output is a list of indices where the bits are 1.
fn sample_bit_vector_with_hamming_weight(length: u64, weight: u32) -> Vec<u64> {
    if weight == 0 || length == 0 || weight as u64 > length {
        panic!("invalid parameters");
    }
    with_thread_rng(|rng| {
        let sample = index::sample(rng, length as usize, weight as usize);
        sample.into_iter().map(|idx| idx as u64).collect()
    })
}

/// Rescale a fixed-point value between fractional precisions.
fn fixed_point_rescale(value: Integer, from_bits: u64, to_bits: u64) -> Integer {
    if from_bits == to_bits {
        return value;
    }
    if from_bits > to_bits {
        return value >> (from_bits - to_bits) as usize;
    }
    value << (to_bits - from_bits) as usize
}

/// Round a fixed-point value to the nearest integer.
fn fixed_point_round_to_int(value: Integer, frac_bits: u64) -> Integer {
    if frac_bits == 0 {
        return value;
    }
    (value + (Integer::from(1) << (frac_bits - 1) as usize)) >> frac_bits as usize
}

/// DGHV parameter struct.
#[derive(Default)]
pub struct Parameters {
    /// Bit lenght of the integers on the public key.
    gamma: u64,
    /// Bit lenght used for the secret key.
    eta: u32,
    /// Bit lenght of the generated noise.
    rho: u32,
    /// Bit lenght of the fresh ciphertext noise.
    rho_prime: u32,
    /// Number of integers in the public key.
    tau: u64,
    /// Sparsity parameter for bootstrapping.
    kappa: u64,
    /// Number of elements in the secret key.
    theta: u32,
    /// Number of samples for sparse secret key generation.
    big_theta: u64,
    // If enabled boostrapping.
    bootstrap_enabled: bool,
}

impl Parameters {
    /// Creates a new [`Parameters`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gamma parameter, which determines the bit length of the integers in the public key.
    pub fn gamma(self, gamma: u64) -> Self {
        Self { gamma, ..self }
    }

    /// Set the eta parameter, which determines the bit length used for the secret key.
    pub fn eta(self, eta: u32) -> Self {
        Self { eta, ..self }
    }

    /// Set the rho parameter, which determines the bit length of the generated noise.
    pub fn rho(self, rho: u32) -> Self {
        Self { rho, ..self }
    }

    /// Set the rho_prime parameter, which determines the bit length of the fresh ciphertext noise.
    pub fn rho_prime(self, rho_prime: u32) -> Self {
        Self { rho_prime, ..self }
    }

    /// Set the tau parameter, which determines the number of integers in the public key.
    pub fn tau(self, tau: u64) -> Self {
        Self { tau, ..self }
    }

    /// Set the kappa parameter, which determines the sparsity for bootstrapping.
    pub fn kappa(self, kappa: u64) -> Self {
        Self { kappa, ..self }
    }

    /// Set the theta parameter, which determines the number of elements in the secret key.
    pub fn theta(self, theta: u32) -> Self {
        Self { theta, ..self }
    }

    /// Set the big_theta parameter, which determines the number of samples for sparse secret key generation.
    pub fn big_theta(self, big_theta: u64) -> Self {
        Self { big_theta, ..self }
    }

    /// Create a new [`Parameters``] instance with the specified security level and depth.
    /// depth is the desired multiplicative depth before bootstrapping. If None,
    /// it is derived from the baseline eta = lambda^2 using the p/8 noise bound. This influence
    /// the effective lambda security level.
    pub fn with_security_and_depth(
        security: u32,
        depth: Option<u32>,
        enable_bootstrap: bool,
    ) -> Option<Self> {
        // Validate the security level and depth parameters.
        if security < 2 {
            return None;
        }
        if let Some(d) = depth {
            if d == 0 {
                return None;
            }
        }

        // Conditionally calculate the hidden cost of the bootstrapping circuit
        let bootstrap_depth = if enable_bootstrap {
            Self::get_bootstrap_depth(security)
        } else {
            0
        };

        // Baseline asymptotic recommendations from the paper.
        let rho = security;
        let rho_prime = security * 2;

        let mut gamma: u64 = u64::from(security).checked_pow(5)?;
        let mut tau: u64 = gamma.checked_add(security.into())?;
        let mut eta = security.checked_pow(2)?;

        // Determine the target depth baseline.
        let initial_noise_bits_baseline = std::cmp::max(rho_prime, rho + ceil_log2_u64(tau));

        let target_depth = if let Some(d) = depth {
            d.checked_add(bootstrap_depth)?
        } else {
            let allowed_noise_bits = eta.checked_sub(4)?;
            if allowed_noise_bits < initial_noise_bits_baseline {
                bootstrap_depth
            } else {
                let ratio = allowed_noise_bits.checked_div(initial_noise_bits_baseline)?;
                let total = bit_length_u32(ratio).saturating_sub(1);
                std::cmp::max(total, bootstrap_depth)
            }
        };

        // Loop to stabilize the cyclic dependency between eta, gamma, and tau
        loop {
            let initial_noise_bits = std::cmp::max(rho_prime, rho + ceil_log2_u64(tau));
            let min_eta_for_depth = initial_noise_bits
                .checked_mul(1u32.checked_shl(target_depth)?)?
                .checked_add(4)?;

            eta = std::cmp::max(security.checked_pow(2)?, min_eta_for_depth);

            // Ensure gamma has enough room for eta
            let min_gamma = (eta as u64) + (rho as u64) + 4;
            let new_gamma = std::cmp::max(u64::from(security).checked_pow(5)?, min_gamma);

            if new_gamma == gamma {
                // The parameters have stabilized
                break;
            }

            gamma = new_gamma;
            tau = gamma.checked_add(security.into())?;
        }

        let kappa = gamma.checked_add(2)?;
        let theta = security;
        let big_theta = kappa.checked_mul(theta.into())?;

        Some(Self {
            gamma,
            eta,
            rho,
            rho_prime,
            tau,
            kappa,
            theta,
            big_theta,
            bootstrap_enabled: enable_bootstrap,
        })
    }

    /// Computes the estimated multiplicative depth before bootstrapping based on the parameters
    /// and the p/8 noise bound.
    pub fn get_estimated_depth(&self) -> u32 {
        let allowed_noise_bits = self.eta.saturating_sub(4);
        if allowed_noise_bits == 0 {
            return 0;
        }
        let subset_noise_bits = self.rho + ceil_log2_u64(self.tau);
        let initial_noise_bits = std::cmp::max(self.rho_prime, subset_noise_bits);
        if initial_noise_bits == 0 {
            return 0;
        }
        let ratio = allowed_noise_bits / initial_noise_bits;
        if ratio == 0 {
            return 0;
        }
        let total_depth = bit_length_u32(ratio).saturating_sub(1);
        let bootstrap_depth = if self.bootstrap_enabled {
            Self::get_bootstrap_depth(self.theta)
        } else {
            0
        };
        total_depth.saturating_sub(bootstrap_depth)
    }

    /// Computes a rough estimate of the security level (lambda) based on the parameters and noise growth.
    pub fn get_estimated_security(&self) -> Option<u32> {
        // This is a very rough estimate based on the parameters and the noise growth.
        let lambda_from_rho = self.rho;
        let lambda_from_eta = sqrt_u32(self.eta);
        let lambda_from_gamma = nth_root_u64(self.gamma, 5);
        Some(std::cmp::min(
            lambda_from_rho,
            std::cmp::min(lambda_from_eta, lambda_from_gamma.try_into().ok()?),
        ))
    }

    /// Computes the required bootstrapping depth to reach a noise level of theta.
    pub fn get_bootstrap_depth(theta: u32) -> u32 {
        let n = ceil_log2_u64(theta as u64) + 3;
        // The max carry chain length is bounded by the accumulator length.
        n + 1 + (ceil_log2_u64(theta as u64)) + 1
    }

    /// Generate the keys and the encryptor, decryptor and evaluator objects.
    pub fn key_generation(&self) -> Option<(Encryptor, Decryptor, Evaluator)> {
        // Sample the secret key p, which is an odd integer of eta bits.
        let mut p: Integer;
        // Sample the public key.
        let x: Vec<Integer>;
        let mut x0: Integer;
        loop {
            p = sample_odd_integer_with_bit_length(self.eta);
            // First, sample tau+1 multipliers (q) to find the largest for x0.
            let mut qs: Vec<Integer> = (0..=self.tau)
                .into_par_iter()
                .map(|_| Some(sample_bounded_integer_with_bit_length(self.gamma, &p)))
                .collect::<Option<Vec<Integer>>>()
                .unwrap();

            qs.sort_unstable();
            let q0 = qs.pop()?;

            // x0 is an exact multiple: q0 * p.
            x0 = q0 * &p;

            // Break if x0 is odd. Remainder check is inherently even since remainder(x0, p) == 0.
            if x0.is_odd() {
                // Generate the remaining tau elements with noise
                x = qs
                    .into_par_iter()
                    .map(|q| {
                        let r = sample_integer_from_ball_with_bit_length(self.rho);
                        q * &p + r
                    })
                    .collect();
                break;
            }
        }

        // Bootstrap elements.
        let xp = quotient(Integer::from(1) << (self.kappa as usize), p.clone());
        let s = sample_bit_vector_with_hamming_weight(self.big_theta, self.theta);
        let modulus = Integer::from(1) << (self.kappa as usize).checked_add(1)?;
        let u = sample_constrained_integers(self.big_theta, s.as_ref(), modulus, xp);

        // Create the encryptor, decryptor and evaluator objects.
        let enc = Encryptor::new(x, x0.clone(), self.rho_prime);
        let dec = Decryptor::new(s.clone(), u.clone(), self.kappa);

        // Encrypt the sparse subset selection array
        let s_enc: Vec<Integer> = (0..self.big_theta)
            .map(|i| enc.encrypt(s.contains(&i)))
            .collect();
        // Generate the encrypted one and zero for homomorphic operations.
        let enc_one = enc.encrypt(true);
        let enc_zero = enc.encrypt(false);

        // Create the evaluator.
        let eval = Evaluator::new()
            .gamma(self.gamma)
            .x0(x0)
            .s_enc(s_enc)
            .u(u)
            .kappa(self.kappa)
            .enc_one(enc_one)
            .enc_zero(enc_zero)
            .theta(self.theta);

        Some((enc, dec, eval))
    }
}

/// Encryptor struct for performing encryption operations.
pub struct Encryptor {
    /// Public key elements x_i.
    x: Vec<Integer>,
    /// Biggest public key element.
    x0: Integer,
    /// Noise bound for fresh ciphertexts.
    rho_prime: u32,
}

impl Encryptor {
    /// Creates a new [`Encryptor`] instance with the given public key elements and noise bound.
    pub fn new(x: Vec<Integer>, x0: Integer, rho_prime: u32) -> Self {
        Self { x, x0, rho_prime }
    }

    /// Encrypts a boolean message (0 or 1) and returns the resulting ciphertext as an integer.
    pub fn encrypt(&self, message: bool) -> Integer {
        let subset = sample_subset(self.x.len() as u64);
        let r = sample_integer_from_ball_with_bit_length(self.rho_prime);
        let sum = subset
            .iter()
            .fold(Integer::from(0), |acc, &idx| acc + &self.x[idx as usize]);
        let message = if message {
            Integer::from(1)
        } else {
            Integer::from(0)
        };
        remainder(message + 2 * r + 2 * sum, self.x0.clone())
    }
}

/// Decryptor struct for performing decryption operations.
pub struct Decryptor {
    /// Secret key element indices.
    s: Vec<u64>,
    /// Fixed point public key elements for bootstrapping.
    u: Vec<Integer>,
    /// Scale factor for key elements of bootstrapping.
    kappa: u64,
}

impl Decryptor {
    /// Creates a new [`Decryptor`] instance with the given secret key indices, bootstrapping elements, and scale factor.
    pub fn new(s: Vec<u64>, u: Vec<Integer>, kappa: u64) -> Self {
        Self { s, u, kappa }
    }

    /// Decrypts a given ciphertext and returns the original boolean message if decryption is successful.
    pub fn decrypt(&self, ciphertext: Integer) -> bool {
        // Choose z precision from the subset size: n = ceil(log2|S|) + 3.
        let n = ceil_log2_u64(self.s.len() as u64) + 3;
        // Work modulo 2^(n+1) to keep one integer bit and n fractional bits.
        let modulus = Integer::from(1) << (n.checked_add(1).unwrap());
        let z = self
            .u
            .par_iter()
            .map(|u_i| {
                let product = u_i * &ciphertext;
                let scaled = fixed_point_rescale(product.complete(), self.kappa, n as u64);
                scaled % &modulus
            })
            .collect::<Vec<Integer>>();

        let sum = self
            .s
            .iter()
            .fold(Integer::from(0), |acc, &idx| acc + &z[idx as usize]);
        let rounded = fixed_point_round_to_int(sum, n as u64);
        (ciphertext - rounded).is_odd()
    }
}

/// Evaluator struct for performing homomorphic evaluation operations.
#[derive(Default)]
pub struct Evaluator {
    /// Bit length of the integers in the public key.
    gamma: u64,
    /// Threshold for when to perform size reduction during homomorphic operations.
    size_reduction_threshold: Integer,
    /// Public key element for size reduction during homomorphic operations.
    x0: Integer,
    /// Encrypted private key elements for bootstrapping.
    s_enc: Vec<Integer>,
    /// Bootstrapping elements for homomorphic evaluation.
    u: Vec<Integer>,
    /// Scale factor for key elements of bootstrapping.
    kappa: u64,
    /// Encrypted one for homomorphic multiplication.
    enc_one: Integer,
    /// Encrypted zero for homomorphic addition.
    enc_zero: Integer,
    /// Number of elements in the secret key.
    theta: u32,
}

impl Evaluator {
    /// Creates a new [`Evaluator`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gamma parameter, which determines the bit length of the integers in the public key.
    pub fn gamma(self, gamma: u64) -> Self {
        let r = Self { gamma, ..self };
        let size_reduction_threshold: Integer = Integer::from(1) << r.gamma as usize;
        Self {
            size_reduction_threshold,
            ..r
        }
    }

    /// Set the x0 parameter, which contains the public key element for size reduction during homomorphic operations.
    pub fn x0(self, x0: Integer) -> Self {
        Self { x0, ..self }
    }

    /// Set the s_enc parameter, which contains the encrypted private key elements for bootstrapping.
    pub fn s_enc(self, s_enc: Vec<Integer>) -> Self {
        Self { s_enc, ..self }
    }

    /// Set the u parameter, which contains the bootstrapping elements for homomorphic evaluation.
    pub fn u(self, u: Vec<Integer>) -> Self {
        Self { u, ..self }
    }

    /// Set the kappa parameter, which determines the scale factor for key elements of bootstrapping.
    pub fn kappa(self, kappa: u64) -> Self {
        Self { kappa, ..self }
    }

    /// Set the enc_one parameter, which contains the encrypted one for homomorphic multiplication.
    pub fn enc_one(self, enc_one: Integer) -> Self {
        Self { enc_one, ..self }
    }

    /// Set the enc_zero parameter, which contains the encrypted zero for homomorphic addition.
    pub fn enc_zero(self, enc_zero: Integer) -> Self {
        Self { enc_zero, ..self }
    }

    /// Set the theta parameter, which determines the number of elements in the secret key.
    pub fn theta(self, theta: u32) -> Self {
        Self { theta, ..self }
    }

    /// Perform size reduction on a given ciphertext to prevent noise growth during homomorphic operations.
    fn rescale(&self, ciphertext: Integer) -> Integer {
        if ciphertext.abs_ref().complete() <= self.size_reduction_threshold {
            return ciphertext;
        }
        remainder(ciphertext, self.x0.clone())
    }

    /// Homomorphically add two ciphertexts and return the resulting ciphertext.
    pub fn add(&self, c1: Integer, c2: Integer) -> Integer {
        self.rescale(c1 + c2)
    }

    /// Homomorphically multiply two ciphertexts and return the resulting ciphertext.
    pub fn mul(&self, c1: Integer, c2: Integer) -> Integer {
        self.rescale(c1 * c2)
    }

    /// Evaluates a homomorphic half adder returning (sum, carry)
    fn half_adder(&self, a: Integer, b: Integer) -> (Integer, Integer) {
        let sum_bit = self.add(a.clone(), b.clone());
        let carry_out = self.mul(a, b);
        (sum_bit, carry_out)
    }

    /// Adds an encrypted bit at the specified position, tracking empty slots to prevent noise explosion.
    fn add_bit_at(
        &self,
        acc: &mut [Integer],
        acc_init: &mut [bool],
        mut bit: Integer,
        pos: usize,
    ) {
        for idx in pos..acc.len() {
            if !acc_init[idx] {
                // If the accumulator at this position is empty, just store the bit and stop rippling!
                acc[idx] = bit;
                acc_init[idx] = true;
                break;
            } else {
                // Collision! Add the bits and ripple the carry to the next position.
                let (sum_bit, carry_out) = self.half_adder(acc[idx].clone(), bit);
                acc[idx] = sum_bit;
                bit = carry_out;
            }
        }
    }

    /// Refreshes a ciphertext by evaluating the squashed decryption circuit homomorphically.
    pub fn bootstrap(&self, c: Integer) -> Integer {
        let n = ceil_log2_u64(self.theta as u64) + 3;
        let modulus = Integer::from(1) << (n + 1);

        let z: Vec<Integer> = self
            .u
            .iter()
            .map(|u_i| {
                let product = u_i * &c;
                let scaled = fixed_point_rescale(product.complete(), self.kappa, n as u64);
                scaled % &modulus
            })
            .collect();

        let sum_len = (n + 1 + ceil_log2_u64(self.theta as u64) + 1) as usize;
        let mut sum_bits = vec![self.enc_zero.clone(); sum_len];
        let mut sum_init = vec![false; sum_len];

        for (i, z_i) in z.iter().enumerate() {
            if *z_i == 0 {
                continue;
            }
            for bit_pos in 0..=(n as usize) {
                if z_i.get_bit(bit_pos as u32) {
                    self.add_bit_at(&mut sum_bits, &mut sum_init, self.s_enc[i].clone(), bit_pos);
                }
            }
        }

        if n > 0 {
            self.add_bit_at(
                &mut sum_bits,
                &mut sum_init,
                self.enc_one.clone(),
                (n - 1) as usize,
            );
        }

        // Safely extract the rounded bit if it was initialized.
        let rounded_bit = if sum_init[n as usize] {
            sum_bits[n as usize].clone()
        } else {
            self.enc_zero.clone()
        };

        if c.is_odd() {
            // If original ciphertext is odd.
            self.add(rounded_bit, self.enc_one.clone())
        } else {
            // If even, the result remains unchanged.
            rounded_bit
        }
    }
}
