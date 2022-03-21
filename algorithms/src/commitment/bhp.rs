// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{crh::BHPCRH, hash_to_curve::hash_to_curve, CommitmentError, CommitmentScheme, CRH};
use snarkvm_curves::{AffineCurve, ProjectiveCurve};
use snarkvm_fields::{ConstraintFieldError, Field, PrimeField, ToConstraintField};
use snarkvm_utilities::BitIteratorLE;

use itertools::Itertools;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BHPCommitment<G: ProjectiveCurve, const INPUT_SIZE: usize> {
    bhp_crh: BHPCRH<G, INPUT_SIZE>,
    random_base: Vec<G>,
}

impl<G: ProjectiveCurve, const INPUT_SIZE: usize> CommitmentScheme for BHPCommitment<G, INPUT_SIZE> {
    type Output = <G::Affine as AffineCurve>::BaseField;
    type Parameters = (BHPCRH<G, INPUT_SIZE>, Vec<G>);
    type Randomness = G::ScalarField;

    fn setup(message: &str) -> Self {
        // First, compute the bases.
        let bhp_crh = BHPCRH::<G, INPUT_SIZE>::setup(message);

        // Next, compute the random base.
        let (generator, _, _) = hash_to_curve::<G::Affine>(&format!("{message} for random base"));
        let mut base = generator.into_projective();

        let num_scalar_bits = G::ScalarField::size_in_bits();
        let mut random_base = Vec::with_capacity(num_scalar_bits);
        for _ in 0..num_scalar_bits {
            random_base.push(base);
            base.double_in_place();
        }
        assert_eq!(random_base.len(), num_scalar_bits);

        Self { bhp_crh, random_base }
    }

    fn commit(&self, input: &[bool], randomness: &Self::Randomness) -> Result<Self::Output, CommitmentError> {
        let mut output = self.bhp_crh.hash_bits_inner(input)?;

        // Compute h^r.
        let scalar_bits = BitIteratorLE::new(randomness.to_repr()).take(G::ScalarField::size_in_bits());
        for (bit, power) in scalar_bits.zip_eq(&self.random_base) {
            if bit {
                output += power
            }
        }

        let affine = output.into_affine();
        debug_assert!(affine.is_in_correct_subgroup_assuming_on_curve());
        Ok(affine.to_x_coordinate())
    }

    fn parameters(&self) -> Self::Parameters {
        (self.bhp_crh.clone(), self.random_base.clone())
    }

    fn window() -> (usize, usize) {
        BHPCRH::<G, INPUT_SIZE>::window()
    }
}

impl<F: Field, G: ProjectiveCurve + ToConstraintField<F>, const INPUT_SIZE: usize> ToConstraintField<F>
    for BHPCommitment<G, INPUT_SIZE>
{
    #[inline]
    fn to_field_elements(&self) -> Result<Vec<F>, ConstraintFieldError> {
        Ok(Vec::new())
    }
}
