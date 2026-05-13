# Fully Homomorphic Encryption over the Integers

Implementation of the fully homomorphic encryption scheme over the integers based on the paper by Dijk, Gentry, Halevi, and Vaikuntanathan (2010). This implementation provides a basic framework for performing arbitrary computations on encrypted data without the need for decryption, demonstrating the core principles of fully homomorphic encryption.

## Functionality

The implementation includes the following functionalities:
- Key generation: Create public and private keys for encryption and decryption.
- Encryption of plaintext messages using the public key.
- Decryption of ciphertexts using the private key.
- Homomorphic addition and multiplication operations.
- Squashed bootstrap circuit to allow for infinite homomorphic operations.

The specific variant implemented an the exact multiple of $p$ for the $x_0$ public key element, allowing for modular reduction during operation.

## Citations

```bibtex
@inproceedings{eurocrypt-2010-24019,
    title={Fully Homomorphic Encryption over the Integers},
    booktitle={Advances in Cryptology - EUROCRYPT 2010, 29th Annual International Conference on the Theory and Applications of Cryptographic Techniques},
    series={Lecture Notes in Computer Science},
    publisher={Springer},
    volume={6110},
    pages={24-43},
    url={https://www.iacr.org/archive/eurocrypt2010/66320254/66320254.pdf},
    doi={10.1007/978-3-642-13190-5_2},
    author={Marten van Dijk and Craig Gentry and Shai Halevi and Vinod Vaikuntanathan},
    year=2010
}
@misc{cryptoeprint:2009/616,
    author = {Marten van Dijk and Craig Gentry and Shai Halevi and Vinod Vaikuntanathan},
    title = {Fully Homomorphic Encryption over the Integers},
    howpublished = {Cryptology {ePrint} Archive, Paper 2009/616},
    year = {2009},
    url = {https://eprint.iacr.org/2009/616}
}
```