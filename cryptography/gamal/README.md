# A Public-Key Cryptosystem and a Signature Scheme Based on Discrete Logarithms

This implementation is based on the ElGamal encryption scheme, which was introduced by Taher ElGamal in his 1985 paper. The ElGamal algorithm is a public-key cryptosystem that relies on the difficulty of the discrete logarithm problem for its security. This implementation includes key generation, encryption, decryption, and digital signature processes, demonstrating the core principles of the ElGamal scheme.

## Functionality

The implementation provides the following functionalities:
- Key generation: Create public and private keys for encryption and decryption.
- Encryption of plaintext messages using the public key.
- Decryption of ciphertexts using the private key.
- Sign messages using the private key and verify signatures using the public key.

## Citations

```bibtex
@inproceedings{10.1007/3-540-39568-7_2,
	author="ElGamal, Taher",
	editor="Blakley, George Robert
	and Chaum, David",
	title="A Public Key Cryptosystem and a Signature Scheme Based on Discrete Logarithms",
	booktitle="Advances in Cryptology",
	year="1985",
	publisher="Springer Berlin Heidelberg",
	address="Berlin, Heidelberg",
	pages="10--18",
	isbn="978-3-540-39568-3"
}
@article{1057074,
	author={Elgamal, T.},
	journal={IEEE Transactions on Information Theory}, 
	title={A public key cryptosystem and a signature scheme based on discrete logarithms}, 
	year={1985},
	volume={31},
	number={4},
	pages={469-472},
	keywords={Ciphers;Public key cryptography;Galois fields;Polynomials;Ions;Digital signatures;Roads;Information systems;Generators;Finite element analysis},
	doi={10.1109/TIT.1985.1057074},
	ISSN={0018-9448},
}
```