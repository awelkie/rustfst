use std::fmt::Debug;

use anyhow::Result;

use bitflags::bitflags;
pub use generic_matcher::GenericMatcher;
pub use multi_eps_matcher::{MultiEpsMatcher, MultiEpsMatcherFlags};
pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL, NO_LABEL};
use crate::{Label, StateId};
use std::rc::Rc;

mod generic_matcher;
mod multi_eps_matcher;
mod sorted_matcher;

bitflags! {
    pub struct MatcherFlags: u32 {
        const REQUIRE_MATCH =  1u32 << 0;
        const INPUT_LOOKAHEAD_MATCHER =  1u32 << 4;
        const OUTPUT_LOOKAHEAD_MATCHER =  1u32 << 5;
        const LOOKAHEAD_WEIGHT =  1u32 << 6;
        const LOOKAHEAD_PREFIX =  1u32 << 7;
        const LOOKAHEAD_NON_EPSILONS =  1u32 << 8;
        const LOOKAHEAD_EPSILONS =  1u32 << 9;
        const LOOKAHEAD_NON_EPSILON_PREFIX =  1u32 << 10;

        const LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits |
            Self::OUTPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_NON_EPSILONS.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;

        const ILABEL_LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;

        const OLABEL_LOOKAHEAD_FLAGS = Self::OUTPUT_LOOKAHEAD_MATCHER.bits |
            Self::LOOKAHEAD_WEIGHT.bits |
            Self::LOOKAHEAD_PREFIX.bits |
            Self::LOOKAHEAD_EPSILONS.bits |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits;
    }
}

pub static REQUIRE_PRIORITY: usize = std::usize::MAX;

#[derive(Copy, Debug, PartialOrd, PartialEq, Clone)]
/// Specifies matcher action
pub enum MatchType {
    /// Match input label
    MatchInput,
    /// Match output label
    MatchOutput,
    /// Match input or output label
    MatchBoth,
    /// Match anything
    MatchNone,
    /// Otherwise, match unknown
    MatchUnknown,
}

// Use this to avoid autoref
#[derive(Clone)]
pub enum IterItemMatcher<W: Semiring> {
    Arc(*const Arc<W>),
    EpsLoop,
}

impl<W: Semiring> IterItemMatcher<W> {
    pub fn into_arc(self, state: StateId, match_type: MatchType) -> Result<Arc<W>> {
        match self {
            IterItemMatcher::Arc(arc) => Ok(unsafe { (*arc).clone() }),
            IterItemMatcher::EpsLoop => eps_loop(state, match_type),
        }
    }
}

pub fn eps_loop<W: Semiring>(state: StateId, match_type: MatchType) -> Result<Arc<W>> {
    let arc = match match_type {
        MatchType::MatchInput => Arc::new(NO_LABEL, EPS_LABEL, W::one(), state),
        MatchType::MatchOutput => Arc::new(EPS_LABEL, NO_LABEL, W::one(), state),
        _ => bail!("Unsupported match_type : {:?}", match_type),
    };
    Ok(arc)
}

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<W: Semiring>: Debug {
    type F: ExpandedFst<W = W>;

    type Iter: Iterator<Item = IterItemMatcher<W>> + Clone;

    fn new(fst: Rc<Self::F>, match_type: MatchType) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter>;
    fn final_weight(&self, state: StateId) -> Result<Option<*const W>>;
    fn match_type(&self) -> MatchType;
    fn flags(&self) -> MatcherFlags;

    /// Indicates preference for being the side used for matching in
    /// composition. If the value is kRequirePriority, then it is
    /// mandatory that it be used. Calling this method without passing the
    /// current state of the matcher invalidates the state of the matcher.
    fn priority(&self, state: StateId) -> Result<usize>;

    fn fst(&self) -> Rc<Self::F>;
}
