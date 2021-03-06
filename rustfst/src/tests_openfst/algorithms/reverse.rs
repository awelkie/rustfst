use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::FinalArc;
use crate::algorithms::MapFinalAction;
use crate::algorithms::WeightConverter;
use crate::algorithms::{reverse, weight_convert};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, CoreFst, MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::Arc;

use crate::tests_openfst::FstTestData;

pub struct ReverseWeightConverter {}

impl<SI, SO> WeightConverter<SI, SO> for ReverseWeightConverter
where
    SI: Semiring,
    SO: Semiring,
{
    fn arc_map(&mut self, arc: &Arc<SI>) -> Result<Arc<SO>> {
        let w = &arc.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };

        Ok(Arc::new(arc.ilabel, arc.olabel, rw, arc.nextstate))
    }

    fn final_arc_map(&mut self, final_arc: &FinalArc<SI>) -> Result<FinalArc<SO>> {
        let w = &final_arc.weight;
        let rw = unsafe { std::mem::transmute::<&SI, &SO>(w).clone() };
        Ok(FinalArc {
            ilabel: final_arc.ilabel,
            olabel: final_arc.olabel,
            weight: rw,
        })
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

pub fn test_reverse<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + AllocableFst + Display,
    F::W: 'static + SerializableSemiring + WeaklyDivisibleSemiring,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: SerializableSemiring,
{
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    let mut mapper = ReverseWeightConverter {};
    let fst_reverse_2: F = weight_convert(&fst_reverse, &mut mapper)?;
    assert_eq!(
        test_data.reverse,
        fst_reverse_2,
        "{}",
        error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
    );
    Ok(())
}
