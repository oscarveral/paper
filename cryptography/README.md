# Cryptography

This directory contains implementations of cryptographic primitives, encryption schemes, and related protocols. Each implementation is based on a specific research paper, with the goal of translating theoretical concepts into working code to gain a deeper understanding of the underlying mechanics.

## Fully Homomorphic Encryption.

- **[Fully Homomorphic Encryption over the Integers](dghv/README.md)**: This implementation is based on the paper by Dijk, Gentry, Halevi, and Vaikuntanathan (2010). It provides a basic implementation of a fully homomorphic encryption scheme that allows for arbitrary computations on encrypted data without decryption.

## Traditional Encryption Schemes.

- **[A Method for Obtaining Digital Signatures and Public-Key Cryptosystems](rsa/README.md)**: This implementation covers the RSA encryption scheme, which is widely used for secure data transmission. It includes key generation, encryption, and decryption processes.