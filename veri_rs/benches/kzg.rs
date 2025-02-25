use ark_bn254::{Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::UniformRand;
use ark_poly::{DenseMultilinearExtension, MultilinearExtension};
use ark_std::{sync::Arc, test_rng};
use csv::Writer;
use hp::{
    pcs::{
        prelude::{MultilinearKzgPCS, PolynomialCommitmentScheme},
        StructuredReferenceString,
    },
    MultilinearUniversalParams,
};
use std::{mem::size_of, time::Instant};

fn main() {
    let size = 23;
    let mut rng = test_rng();
    let uni_params = MultilinearKzgPCS::<Bn254>::gen_srs_for_testing(&mut rng, size).unwrap();
    let mut wtr = Writer::from_path("mkzg.csv").unwrap();
    wtr.write_record(["nv", "commit_time", "proof_size"])
        .unwrap();
    for nv in 6..size {
        let repetition = 10;
        let (commit_time, proof_size) = mkzg(nv, repetition, &uni_params);
        wtr.write_record([nv, commit_time, proof_size].map(|x| x.to_string()))
            .unwrap();
    }
}

fn mkzg(
    nv: usize,
    repetition: usize,
    uni_params: &MultilinearUniversalParams<Bn254>,
) -> (usize, usize) {
    let mut rng = test_rng();
    let poly = Arc::new(DenseMultilinearExtension::rand(nv, &mut rng));
    let (ck, vk) = uni_params.trim(nv).unwrap();

    let point: Vec<_> = (0..nv).map(|_| Fr::rand(&mut rng)).collect();

    // commit
    let (commit_time, com) = {
        let start = Instant::now();
        for _ in 0..repetition {
            let _commit = MultilinearKzgPCS::commit(&ck, &poly).unwrap();
        }

        (
            start.elapsed().as_micros() as usize / repetition,
            MultilinearKzgPCS::commit(&ck, &poly).unwrap(),
        )
    };

    // open
    let (proof, value) = {
        for _ in 0..repetition - 1 {
            let _open = MultilinearKzgPCS::open(&ck, &poly, &point).unwrap();
        }

        let open = MultilinearKzgPCS::open(&ck, &poly, &point).unwrap();

        open
    };

    let proof_size = proof.proofs.len() * size_of::<<Bn254 as Pairing>::G1Affine>();

    (commit_time, proof_size)
}
