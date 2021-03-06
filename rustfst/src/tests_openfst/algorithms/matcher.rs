use anyhow::Result;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::arc_compares::{ilabel_compare, olabel_compare};
use crate::algorithms::arc_sort;
use crate::algorithms::compose::matchers::{MatchType, Matcher, SortedMatcher};
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;
use crate::{Arc, Label, StateId, NO_LABEL, NO_STATE_ID};

#[derive(Serialize, Deserialize, Debug)]
struct SerializedArc {
    ilabel: i32,
    olabel: i32,
    weight: String,
    nextstate: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatcherOperationResult {
    state: usize,
    label: usize,
    match_type: usize,
    arcs: Vec<SerializedArc>,
}

pub struct MatcherTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    label: Label,
    state: StateId,
    match_type: MatchType,
    arcs: Vec<Arc<F::W>>,
}

impl MatcherOperationResult {
    pub fn parse<F>(&self) -> MatcherTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        MatcherTestData {
            label: self.label,
            state: self.state,
            match_type: match self.match_type {
                1 => MatchType::MatchInput,
                2 => MatchType::MatchOutput,
                _ => panic!("Unsupported match_type : {:?}", self.match_type),
            },
            arcs: self
                .arcs
                .iter()
                .map(|s| {
                    let ilabel = if s.ilabel == -1 {
                        NO_LABEL
                    } else {
                        s.ilabel as usize
                    };

                    let olabel = if s.olabel == -1 {
                        NO_LABEL
                    } else {
                        s.olabel as usize
                    };

                    let nextstate = if s.nextstate == -1 {
                        NO_STATE_ID
                    } else {
                        s.nextstate as usize
                    };

                    Arc::new(
                        ilabel,
                        olabel,
                        F::W::parse_text(s.weight.as_str()).unwrap().1,
                        nextstate,
                    )
                })
                .collect(),
        }
    }
}

pub fn test_sorted_matcher<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + AllocableFst,
    F::W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    unimplemented!()
    // let mut fst_isorted = test_data.raw.clone();
    // arc_sort(&mut fst_isorted, ilabel_compare);
    //
    // let mut fst_osorted = test_data.raw.clone();
    // arc_sort(&mut fst_osorted, olabel_compare);
    //
    // for matcher_data in &test_data.matcher {
    //     let fst = match matcher_data.match_type {
    //         MatchType::MatchInput => &fst_isorted,
    //         MatchType::MatchOutput => &fst_osorted,
    //         _ => bail!("Unsupported match_type : {:?}", matcher_data.match_type),
    //     };
    //
    //     let matcher = SortedMatcher::new(fst, matcher_data.match_type)?;
    //     let arcs: Vec<Arc<_>> = matcher
    //         .iter(matcher_data.state, matcher_data.label)?
    //         .map(|f| {
    //             f.into_arc(matcher_data.state, matcher_data.match_type)
    //                 .unwrap()
    //         })
    //         .collect();
    //
    //     assert_eq!(
    //         arcs,
    //         matcher_data.arcs.iter().cloned().collect_vec(),
    //         "Test matcher failed {:?} {:?} {:?}",
    //         matcher_data.state,
    //         matcher_data.label,
    //         matcher_data.match_type
    //     );
    // }
    // Ok(())
}
