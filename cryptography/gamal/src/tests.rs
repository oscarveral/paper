use crate::{Individual, Parameters};
use rug::Integer;

#[test]
fn encryption() {
    let params = Parameters::from_bits(256);
    let mut alice = Individual::new_random(params.p.clone(), params.alpha.clone());
    let mut bob = Individual::new_random(params.p.clone(), params.alpha.clone());
    let y_a = alice.compute_y();
    let y_b = bob.compute_y();
    alice.store_other_y(y_b);
    bob.store_other_y(y_a);
    alice.compute_shared_secret();
    bob.compute_shared_secret();
    assert_eq!(alice.shared_secret, bob.shared_secret);
    let m = Integer::from(424242);
    let (c1, c2) = alice.encrypt(&m);
    let decrypted_m = bob.decrypt(&c1, &c2);
    assert_eq!(m, decrypted_m);
}

#[test]
fn signature() {
    let params = Parameters::from_bits(256);
    let mut alice = Individual::new_random(params.p.clone(), params.alpha.clone());
    let bob = Individual::new_random(params.p.clone(), params.alpha.clone());
    let pub_alice = alice.compute_y();
    let document = Integer::from(1337);
    let (r, s) = alice.sign(&document);
    let is_valid = bob.verify_signature(&document, &r, &s, &pub_alice);
    assert!(is_valid);
    let fake_document = Integer::from(9999);
    let is_valid_forgery = bob.verify_signature(&fake_document, &r, &s, &pub_alice);
    assert!(!is_valid_forgery);
}
