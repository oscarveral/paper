# A Method for Obtaining Digital Signatures and Public-Key Cryptosystems

This implementation is based on the RSA encryption scheme, which was introduced by Rivest, Shamir, and Adleman in their seminal 1978 paper. The RSA algorithm is a widely used public-key cryptosystem that enables secure data transmission and digital signatures. This implementation includes key generation, encryption, and decryption processes, demonstrating the core principles of RSA.

## Functionality

The implementation provides the following functionalities:
- Key generation: Create public and private keys for encryption and decryption.
- Encryption of plaintext messages using the public key.
- Decryption of ciphertexts using the private key.

Digital signatures are not included as part of this implementation, as they are trivially derived from the encryption and decryption processes. The focus is on the core RSA algorithm, which serves as the foundation for both encryption and digital signatures.

## Citations

```bibtex
@article{10.1145/359340.359342,
    author = {Rivest, R. L. and Shamir, A. and Adleman, L.},
    title = {A method for obtaining digital signatures and public-key cryptosystems},
    year = {1978},
    issue_date = {Feb. 1978},
    publisher = {Association for Computing Machinery},
    address = {New York, NY, USA},
    volume = {21},
    number = {2},
    issn = {0001-0782},
    url = {https://doi.org/10.1145/359340.359342},
    doi = {10.1145/359340.359342},
    journal = {Commun. ACM},
    month = feb,
    pages = {120–126},
    numpages = {7},
    keywords = {security, public-key cryptosystems, privacy, prime number, message-passing, factorization, electronic mail, electronic funds transfer, digital signatures, cryptography, authentication}
}
```