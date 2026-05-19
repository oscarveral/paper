use crate::Parameters;
use rug::Integer;
use rug::integer::Order;

#[test]
fn test_key_generation() {
    let params = Parameters::generate(512);
    // Ensure primes were generated and are distinct.
    assert!(params.p > 0);
    assert!(params.q > 0);
    assert_ne!(params.p, params.q);
    let (encryptor, decryptor) = params.key_gen();
    // Ensure standard properties of the keys exist.
    assert!(encryptor.e > 0);
    assert!(encryptor.n > 0);
    assert!(decryptor.d > 0);
    assert_eq!(
        encryptor.n, decryptor.n,
        "public and private modulus n should match"
    );
}

#[test]
fn test_encrypt_decrypt_small_number() {
    let params = Parameters::generate(512);
    let (encryptor, decryptor) = params.key_gen();
    // Test with a small number that easily fits in a single block
    let original_message = Integer::from(42);
    let encrypted = encryptor.encrypt(original_message.clone());
    let decrypted = decryptor.decrypt(encrypted);
    assert_eq!(
        original_message, decrypted,
        "decrypted small number should match original"
    );
}

#[test]
fn test_encrypt_decrypt_large_message() {
    // Use a 512-bit key.
    let params = Parameters::generate(512);
    let (encryptor, decryptor) = params.key_gen();
    // Create a 100-byte message to force multi-block encryption.
    let large_msg_bytes = vec![0xABu32; 100];
    let original_message = Integer::from_digits(&large_msg_bytes, Order::Msf);
    let encrypted = encryptor.encrypt(original_message.clone());
    assert!(
        encrypted.len() > 1,
        "large message should have been split into multiple blocks"
    );
    let decrypted = decryptor.decrypt(encrypted);
    assert_eq!(
        original_message, decrypted,
        "decrypted multi-block message should match original"
    );
}

#[test]
fn test_string_message() {
    let params = Parameters::generate(2048);
    let (encryptor, decryptor) = params.key_gen();
    let text = "Hello, RSA! This is a secret message.";
    // Convert string to bytes, then to an integer.
    let original_message = Integer::from_digits(text.as_bytes(), Order::Msf);
    // Encrypt and decrypt.
    let encrypted = encryptor.encrypt(original_message);
    let decrypted = decryptor.decrypt(encrypted);
    // Convert the decrypted integer back to bytes, then to a string.
    let decrypted_bytes = decrypted.to_digits::<u8>(Order::Msf);
    let decrypted_text = String::from_utf8(decrypted_bytes).expect("failed to parse valid UTF-8");
    assert_eq!(
        text, decrypted_text,
        "string text should survive the round trip intact"
    );
}
