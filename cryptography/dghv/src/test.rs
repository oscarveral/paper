use crate::Parameters;

#[test]
fn parameter_generation() {
    for security in 2..=128 {
        let params = Parameters::with_security_and_depth(security, None, false);
        assert!(params.is_some());
        let params = params.unwrap();
        let estimated_lambda = params.get_estimated_security();
        assert!(estimated_lambda.is_some());
        assert!(estimated_lambda.unwrap() >= security);
        for depth in 1..=3 {
            let params = Parameters::with_security_and_depth(security, Some(depth), false);
            assert!(params.is_some());
            let params = params.unwrap();
            let estimated_lambda = params.get_estimated_security();
            assert!(estimated_lambda.is_some());
            assert!(estimated_lambda.unwrap() >= security);
            let estimated_depth = params.get_estimated_depth();
            assert!(estimated_depth >= depth);
        }
    }
}

#[test]
fn key_generation() {
    for security in 2..=4 {
        let params = Parameters::with_security_and_depth(security, None, false);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, _) = r.unwrap();
        for _ in 0..10 {
            let m = rand::random::<bool>();
            let c = enc.encrypt(m);
            let decrypted = dec.decrypt(c);
            assert_eq!(m, decrypted);
        }
    }
}

#[test]
fn add() {
    for security in 2..=4 {
        let params = Parameters::with_security_and_depth(security, None, false);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, eval) = r.unwrap();
        for _ in 0..10 {
            let m1 = rand::random::<bool>();
            let m2 = rand::random::<bool>();
            let c1 = enc.encrypt(m1);
            let c2 = enc.encrypt(m2);
            let c_sum = eval.add(c1, c2);
            let decrypted_sum = dec.decrypt(c_sum);
            assert_eq!(m1 ^ m2, decrypted_sum);
        }
    }
}

#[test]
fn mul() {
    for security in 2..=4 {
        let params = Parameters::with_security_and_depth(security, Some(1), false);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, eval) = r.unwrap();
        for _ in 0..10 {
            let m1 = rand::random::<bool>();
            let m2 = rand::random::<bool>();
            let c1 = enc.encrypt(m1);
            let c2 = enc.encrypt(m2);
            let c_prod = eval.mul(c1, c2);
            let decrypted_prod = dec.decrypt(c_prod);
            assert_eq!(m1 & m2, decrypted_prod);
        }
    }
}

#[test]
fn depth() {
    for security in 2..=4 {
        let depth = security;
        let params = Parameters::with_security_and_depth(security, Some(depth), false);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, eval) = r.unwrap();
        let mut b = rand::random::<bool>();
        let mut c = enc.encrypt(b);
        for _ in 0..depth {
            let b_tmp = rand::random::<bool>();
            let c_tmp = enc.encrypt(b_tmp);
            c = eval.mul(c, c_tmp);
            let decrypted = dec.decrypt(c.clone());
            assert_eq!(b & b_tmp, decrypted);
            b = b & b_tmp;
        }
    }
}

#[test]
fn bootstrap() {
    for security in 2..=3 {
        let params = Parameters::with_security_and_depth(security, Some(1), true);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, eval) = r.unwrap();
        for _ in 0..4 {
            let m = rand::random::<bool>();
            let c = enc.encrypt(m);
            let decrypted_before = dec.decrypt(c.clone());
            assert_eq!(m, decrypted_before);
            let c_bootstrapped = eval.bootstrap(c);
            let decrypted_after = dec.decrypt(c_bootstrapped);
            assert_eq!(m, decrypted_after);
        }
    }
}

#[test]
fn bootstrap_depth() {
    for security in 2..=3 {
        let depth = 2;
        let params = Parameters::with_security_and_depth(security, Some(depth), true);
        assert!(params.is_some());
        let params = params.unwrap();
        let r = params.key_generation();
        assert!(r.is_some());
        let (enc, dec, eval) = r.unwrap();
        let mut b = rand::random::<bool>();
        let mut c = enc.encrypt(b);
        // Consume the depth with multiplications.
        for _ in 0..depth {
            let b_tmp = rand::random::<bool>();
            let c_tmp = enc.encrypt(b_tmp);
            c = eval.mul(c, c_tmp);
            let decrypted = dec.decrypt(c.clone());
            assert_eq!(b & b_tmp, decrypted);
            b = b & b_tmp;
        }
        // Now bootstrap and check that we can still decrypt correctly.
        let c_bootstrapped = eval.bootstrap(c);
        let decrypted_after = dec.decrypt(c_bootstrapped);
        assert_eq!(b, decrypted_after);
    }
}
