use rand::Rng;
use rand::rngs::ThreadRng;
use rug::Integer;

#[cfg(test)]
mod tests;

/// A simple RSA encryptor that uses the public key (e, n) to encrypt messages.
pub struct Encryptor {
    pub e: Integer,
    pub n: Integer,
}

impl Encryptor {
    /// Creates a new Encryptor with the given public key (e, n).
    pub fn new(e: Integer, n: Integer) -> Self {
        Encryptor { e, n }
    }

    /// Encrypts a message using the RSA encryption algorithm.
    pub fn encrypt(&self, message: Integer) -> Vec<Integer> {
        // Find a safe block size in bits to guarantee every block is strictly less than n.
        let chunk_bits = self.n.significant_bits() - 1;
        let mut blocks = Vec::new();
        let mut temp_msg = message;
        // Create a bitmask to extract exactly chunk_bits at a time.
        let mask = (Integer::from(1) << chunk_bits) - 1;
        if temp_msg == 0 {
            // Handle the edge case of encrypting a literal 0.
            blocks.push(Integer::new().pow_mod(&self.e, &self.n).unwrap());
        } else {
            // Extract chunks from least significant to most significant.
            while temp_msg > 0 {
                // Extract the lowest bits.
                let block = Integer::from(&temp_msg & &mask);
                // Encrypt the block and store it.
                blocks.push(block.pow_mod(&self.e, &self.n).unwrap());
                // Shift the message right to process the next chunk.
                temp_msg >>= chunk_bits;
            }
        }
        blocks
    }
}

/// A simple RSA decryptor that uses the private key (d, n) to decrypt messages.
pub struct Decryptor {
    pub d: Integer,
    pub n: Integer,
}

impl Decryptor {
    /// Creates a new Decryptor with the given private key (d, n).
    pub fn new(d: Integer, n: Integer) -> Self {
        Decryptor { d, n }
    }

    /// Decrypts a list of encrypted blocks using the RSA decryption algorithm.
    pub fn decrypt(&self, blocks: Vec<Integer>) -> Integer {
        let chunk_bits = self.n.significant_bits() - 1;
        let mut message = Integer::new();
        for (i, block) in blocks.into_iter().enumerate() {
            // Decrypt the block.
            let decrypted = block.pow_mod(&self.d, &self.n).unwrap();
            // Shift the decrypted block left, back to its original bit position.
            let shifted = decrypted << (i as u32 * chunk_bits);
            // Recombine it into the final message.
            message += shifted;
        }
        message
    }
}

/// A struct to hold the parameters for RSA key generation.
pub struct Parameters {
    pub p: Integer,
    pub q: Integer,
}
impl Parameters {
    /// Generates RSA parameters by creating two large prime numbers p and q.
    pub fn generate(bit_size: usize) -> Self {
        if bit_size < 512 {
            panic!("bit size must be at least 512 for secure RSA keys");
        }
        let half_bits = (bit_size / 2) as u32;
        let bytes_len = half_bits.div_ceil(8) as usize;
        // Helper closure to generate a secure prime of exactly half_bits.
        let generate_prime = || -> Integer {
            // Generate cryptographically secure random bytes using OS noise.
            let mut buffer = vec![0u8; bytes_len];
            ThreadRng::default().fill_bytes(&mut buffer);
            // Convert the random bytes into a large Integer.
            let mut candidate = Integer::from_digits(&buffer, rug::integer::Order::Msf);
            // Truncate extra bits in case half_bits isn't a perfect multiple of 8.
            candidate = candidate.keep_bits(half_bits);
            // Set the top two bits to 1 to ensure that p * q is exactly bit_size long.
            candidate.set_bit(half_bits - 1, true);
            candidate.set_bit(half_bits - 2, true);
            // Force the number to be odd before searching for a prime.
            candidate.set_bit(0, true);
            // Find the next prime strictly greater than our random candidate.
            candidate.next_prime()
        };
        let p = generate_prime();
        let mut q = generate_prime();
        // Ensure p and q are not identical.
        while p == q {
            q = generate_prime();
        }
        Parameters { p, q }
    }

    /// Generates the public and private keys (Encryptor and Decryptor) based on the parameters p and q.
    pub fn key_gen(self) -> (Encryptor, Decryptor) {
        // Calculate the modulus n = p * q.
        let n = Integer::from(&self.p * &self.q);
        // Calculate Euler's totient function phi(n) = (p - 1) * (q - 1).
        let p_minus_1 = Integer::from(&self.p - 1);
        let q_minus_1 = Integer::from(&self.q - 1);
        let phi = p_minus_1 * q_minus_1;
        // Choose the public exponent e.
        let mut e = Integer::from(65537);
        // Ensure e and phi(n) are coprime (gcd == 1). If not, increment e by 2 and try again.
        while Integer::from(e.gcd_ref(&phi)) != 1 {
            e += 2;
        }
        // Calculate the private exponent d, which is the modular inverse of e mod phi(n).
        let d = Integer::from(
            e.invert_ref(&phi)
                .expect("failed to calculate modular inverse for d"),
        );
        // Return the initialized public and private components.
        (Encryptor::new(e, n.clone()), Decryptor::new(d, n))
    }
}
