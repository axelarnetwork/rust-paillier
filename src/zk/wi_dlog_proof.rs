#![allow(non_snake_case)]
/*
    zk-paillier

    Copyright 2018 by Kzen Networks

    zk-paillier is free software: you can redistribute
    it and/or modify it under the terms of the GNU General Public
    License as published by the Free Software Foundation, either
    version 3 of the License, or (at your option) any later version.

    @license GPL-3.0+ <https://github.com/KZen-networks/zk-paillier/blob/master/LICENSE>
*/

// use curv::arithmetic::traits::{Modulo, Samplable};
// use curv::cryptographic_primitives::proofs::ProofError;
// use curv::BigInt;
use crate::{bigint, BigInt};
use serde::{Deserialize, Serialize};
use std::iter;

// Witness Indistinguishable Proof of knowledge of discrete log with composite modulus.
// We follow the Girault’s proof from Pointcheval paper (figure1):
// https://www.di.ens.fr/david.pointcheval/Documents/Papers/2000_pkcA.pdf
// The prover wants to prove knowledge of a secret s given a public v = g^-{s} mod N for composite N

const K: usize = 128;
const K_PRIME: usize = 128;
const SAMPLE_S: usize = 256;

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub struct CompositeDLogProof {
    pub x: BigInt,
    pub y: BigInt,
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub struct DLogStatement {
    pub N: BigInt,
    pub g: BigInt,
    pub ni: BigInt,
}

impl CompositeDLogProof {
    pub fn prove(statement: &DLogStatement, secret: &BigInt) -> CompositeDLogProof {
        //   pub fn prove(statement: &DLogStatement, secret: &BigInt, dk: &DecryptionKey) -> DLogProof{

        //   let one = BigInt::one();
        //  let phi = (&dk.p - &one) * (&dk.q - &one);
        //   let r = BigInt::sample_below(&phi);

        let R = BigInt::from(2).pow((K + K_PRIME + SAMPLE_S) as u32);
        let r = bigint::sample_below(&R);
        let x = statement.g.powm(&r, &statement.N);
        let e = super::compute_digest(
            iter::once(&x)
                .chain(iter::once(&statement.g))
                .chain(iter::once(&statement.N)),
        );
        let y = bigint::mod_add(
            &r,
            &(bigint::mod_mul(&e, secret, &statement.N)),
            &statement.N,
        );
        CompositeDLogProof { x, y }
    }

    pub fn verify(&self, statement: &DLogStatement) -> Result<(), &'static str> {
        //assert N > 2^k
        assert!(statement.N > BigInt::from(2).pow(K as u32));

        //test that g, ni in multiplecative group Z_N*
        assert_eq!(statement.g.gcd(&statement.N), BigInt::one());
        assert_eq!(statement.ni.gcd(&statement.N), BigInt::one());

        let e = super::compute_digest(
            iter::once(&self.x)
                .chain(iter::once(&statement.g))
                .chain(iter::once(&statement.N)),
        );
        let ni_e = statement.ni.powm(&e, &statement.N);
        let g_y = statement.g.powm(&self.y, &statement.N);
        let g_y_ni_e = bigint::mod_mul(&g_y, &ni_e, &statement.N);

        // x=? g^yv^e modN
        if self.x == g_y_ni_e {
            Ok(())
        } else {
            Err("verify fail")
        }
    }
}

pub fn legendre_symbol(a: &BigInt, p: &BigInt) -> i32 {
    let p_minus_1: BigInt = p - BigInt::one();
    let pow = bigint::mod_mul(&p_minus_1, &BigInt::from(2).invert(p).unwrap(), p);
    let ls = a.powm(&pow, p);
    if ls == BigInt::one() {
        1
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{KeyGeneration, Paillier};
    // use paillier::KeyGeneration;
    // use paillier::Paillier;

    #[test]
    fn test_correct_dlog_proof() {
        // should be safe primes (not sure if there is actual attack)
        let (ek, dk) = Paillier::keypair(&mut rand::thread_rng()).keys();
        let one = BigInt::one();
        let S = BigInt::from(2).pow(SAMPLE_S as u32);
        // Per definition 3 in the paper we need to make sure h1 is asymmetric basis:
        // Jacobi symbol should be -1.
        let mut h1 = bigint::sample_range(&one, &(&ek.n - &one));
        let mut jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        while jacobi_symbol != -1 {
            h1 = bigint::sample_range(&one, &(&ek.n - &one));
            jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        }
        let secret = bigint::sample_below(&S);
        let h2 = h1.powm(&(-&secret), &ek.n);
        let statement = DLogStatement {
            N: ek.n,
            g: h1,
            ni: h2,
        };
        let proof = CompositeDLogProof::prove(&statement, &secret);
        let v = proof.verify(&statement);
        assert!(v.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_bad_dlog_proof() {
        let (ek, dk) = Paillier::keypair(&mut rand::thread_rng()).keys();
        let one = BigInt::one();
        let S = BigInt::from(2).pow(SAMPLE_S as u32);
        // Per definition 3 in the paper we need to make sure h1 is asymmetric basis:
        // Jacobi symbol should be -1.
        let mut h1 = bigint::sample_range(&one, &(&ek.n - &one));
        let mut jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        while jacobi_symbol != -1 {
            h1 = bigint::sample_range(&one, &(&ek.n - &one));
            jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        }
        let secret = bigint::sample_below(&S);
        // here we use "+secret", instead of "-secret".
        let h2 = h1.powm(&(secret), &ek.n);
        let statement = DLogStatement {
            N: ek.n,
            g: h1,
            ni: h2,
        };
        let proof = CompositeDLogProof::prove(&statement, &secret);
        let v = proof.verify(&statement);
        assert!(v.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_bad_dlog_proof_2() {
        let (ek, dk) = Paillier::keypair(&mut rand::thread_rng()).keys();
        let one = BigInt::one();
        let S = BigInt::from(2).pow(SAMPLE_S as u32);
        // Per definition 3 in the paper we need to make sure h1 is asymmetric basis:
        // Jacobi symbol should be -1.
        let mut h1 = bigint::sample_range(&one, &(&ek.n - &one));
        let mut jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        while jacobi_symbol != -1 {
            h1 = bigint::sample_range(&one, &(&ek.n - &one));
            jacobi_symbol = legendre_symbol(&h1, &dk.p) * legendre_symbol(&h1, &dk.q);
        }
        let secret = bigint::sample_below(&S);
        // here we let h2 to be sampled in random
        let h2 = bigint::sample_range(&one, &(&ek.n - &one));

        let statement = DLogStatement {
            N: ek.n,
            g: h1,
            ni: h2,
        };
        let proof = CompositeDLogProof::prove(&statement, &secret);
        let v = proof.verify(&statement);
        assert!(v.is_ok());
    }
}
