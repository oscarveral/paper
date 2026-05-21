use rug::Integer;
use rug::integer::IsPrime;
use rug::rand::RandState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests;

static SEED_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A struct that represents an individual in ElGamal public key cryptosystem.
pub struct Individual {
    /// The secret of the individual.
    secret: Integer,
    /// The prime number used in the cryptosystem.
    pub p: Integer,
    /// The generator of the multiplicative group of integers modulo p.
    pub alpha: Integer,
    /// The stored public key of the other party (y_x).
    pub other_y: Option<Integer>,
    /// The computed Diffie-Hellman shared secret.
    pub shared_secret: Option<Integer>,
    /// The internal random state used for generating ephemeral keys.
    pub rand_state: RandState<'static>,
}

impl Individual {
    /// Creates a new individual with an explicitly provided secret.
    pub fn new(secret: Integer, p: Integer, alpha: Integer) -> Self {
        let mut rand_state = RandState::new();
        let counter = SEED_COUNTER.fetch_add(1, Ordering::SeqCst);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seed = time.wrapping_add(counter as u128);
        rand_state.seed(&Integer::from(seed));

        Individual {
            secret,
            p,
            alpha,
            other_y: None,
            shared_secret: None,
            rand_state,
        }
    }

    /// Creates a new individual by dynamically generating a random secret.
    pub fn new_random(p: Integer, alpha: Integer) -> Self {
        let mut rand_state = RandState::new();
        let counter = SEED_COUNTER.fetch_add(1, Ordering::SeqCst);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seed = time.wrapping_add(counter as u128);
        rand_state.seed(&Integer::from(seed));

        let bits = p.significant_bits();
        let mut secret;

        loop {
            secret = Integer::from(Integer::random_bits(bits, &mut rand_state));
            if secret > 0 && secret < p {
                break;
            }
        }

        Individual {
            secret,
            p,
            alpha,
            other_y: None,
            shared_secret: None,
            rand_state,
        }
    }

    /// Stores the other individual's public key element (y_x)
    pub fn store_other_y(&mut self, y: Integer) {
        self.other_y = Some(y);
    }

    /// Computes the public key element to be shared (y = alpha^secret mod p).
    pub fn compute_y(&self) -> Integer {
        self.alpha.clone().pow_mod(&self.secret, &self.p).unwrap()
    }

    /// Computes the Diffie-Hellman shared secret using the stored other_y.
    pub fn compute_shared_secret(&mut self) {
        let y = self.other_y.as_ref().unwrap();
        self.shared_secret = Some(y.clone().pow_mod(&self.secret, &self.p).unwrap());
    }

    /// Encrypts a message m using the stored recipient's public key.
    pub fn encrypt(&mut self, m: &Integer) -> (Integer, Integer) {
        let y = self.other_y.as_ref().unwrap();
        if *m < 0 || *m >= self.p {
            panic!("encryption failed: message must be between 0 and p - 1 inclusive");
        }
        let bits = self.p.significant_bits();
        let mut k;
        loop {
            k = Integer::from(Integer::random_bits(bits, &mut self.rand_state));
            if k > 0 && k < self.p {
                break;
            }
        }
        let k_large = y.clone().pow_mod(&k, &self.p).unwrap();
        let c1 = self.alpha.clone().pow_mod(&k, &self.p).unwrap();
        let c2 = (k_large * m.clone()) % &self.p;
        (c1, c2)
    }

    /// Decrypts a ciphertext tuple (c1, c2) intended for this individual.
    pub fn decrypt(&self, c1: &Integer, c2: &Integer) -> Integer {
        let k_large = c1.clone().pow_mod(&self.secret, &self.p).unwrap();
        let k_inv = k_large.invert(&self.p).unwrap();
        (c2.clone() * k_inv) % &self.p
    }

    /// Signs a document m using this individual's private secret.
    pub fn sign(&mut self, m: &Integer) -> (Integer, Integer) {
        if *m < 0 || *m >= self.p {
            panic!("signing failed: message must be between 0 and p - 1 inclusive");
        }
        let p_minus_1: Integer = self.p.clone() - 1;
        let bits = p_minus_1.significant_bits();
        let k: Integer;
        loop {
            let candidate = Integer::from(Integer::random_bits(bits, &mut self.rand_state));
            if candidate > 0 && candidate < p_minus_1 {
                let gcd = candidate.clone().gcd(&p_minus_1);
                if gcd == 1 {
                    k = candidate;
                    break;
                }
            }
        }
        let r = self.alpha.clone().pow_mod(&k, &self.p).unwrap();
        let k_inv = k.invert(&p_minus_1).unwrap();
        let xr = self.secret.clone() * r.clone();
        let mut s = ((m.clone() - xr) * k_inv) % &p_minus_1;
        if s < 0 {
            s += p_minus_1;
        }

        (r, s)
    }

    /// Verifies a signature (r, s) for a document m using the signer's public key y.
    pub fn verify_signature(&self, m: &Integer, r: &Integer, s: &Integer, y: &Integer) -> bool {
        if *r <= 0 || *r >= self.p {
            return false;
        }
        let p_minus_1 = self.p.clone() - 1;
        let mut s_normalized = s.clone() % &p_minus_1;
        if s_normalized < 0 {
            s_normalized += p_minus_1;
        }
        let left = self.alpha.clone().pow_mod(m, &self.p).unwrap();
        let y_r = y.clone().pow_mod(r, &self.p).unwrap();
        let r_s = r.clone().pow_mod(&s_normalized, &self.p).unwrap();
        let right = (y_r * r_s) % &self.p;
        left == right
    }
}

/// A struct that represents the ElGamal cryptosystem parameters.
pub struct Parameters {
    pub p: Integer,
    pub alpha: Integer,
}

impl Parameters {
    pub fn new(p: Integer, alpha: Integer) -> Self {
        Parameters { p, alpha }
    }

    pub fn from_bits(bit_length: u32) -> Self {
        let mut rand = RandState::new();
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        rand.seed(&Integer::from(seed));
        let mut q: Integer;
        let mut p: Integer;
        loop {
            let mut rand_q = Integer::from(Integer::random_bits(bit_length - 1, &mut rand));
            rand_q.set_bit(bit_length - 2, true);
            q = rand_q.next_prime();
            p = (q.clone() * 2) + 1;
            if p.is_probably_prime(30) != IsPrime::No {
                break;
            }
        }
        let mut alpha = Integer::from(2);
        loop {
            let cond1 = alpha.clone().pow_mod(&Integer::from(2), &p).unwrap() != 1;
            let cond2 = alpha.clone().pow_mod(&q, &p).unwrap() != 1;
            if cond1 && cond2 {
                break;
            }
            alpha += 1;
        }
        Parameters { p, alpha }
    }
}
