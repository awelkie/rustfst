use std::cmp::Ordering;

use anyhow::Result;

use crate::algorithms::{ArcMapper, ClosureType};
use crate::arc::Arc;
use crate::fst_traits::{CoreFst, ExpandedFst, FstIteratorMut};
use crate::{Label, StateId};

/// Trait defining the methods to modify a wFST.
pub trait MutableFst:
    ExpandedFst + for<'a> MutableArcIterator<'a> + for<'b> FstIteratorMut<'b>
{
    /// Creates an empty wFST.
    fn new() -> Self;

    /// The state with identifier `state_id` is now the start state.
    /// Note that only one start state is allowed in this implementation. Calling this function twice
    /// will mean losing the first start state.
    /// If the `state_id` doesn't exist an error is raised.
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// # use rustfst::Arc;
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.start(), None);
    ///
    /// fst.set_start(s1);
    /// assert_eq!(fst.start(), Some(s1));
    ///
    /// fst.set_start(s2);
    /// assert_eq!(fst.start(), Some(s2));
    /// ```
    fn set_start(&mut self, state_id: StateId) -> Result<()>;
    unsafe fn set_start_unchecked(&mut self, state_id: StateId);

    /// The state with identifier `state_id` is now a final state with a weight `final_weight`.
    /// If the `state_id` doesn't exist an error is raised.
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// # use rustfst::Arc;
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.final_weight(s1).unwrap(), None);
    /// assert_eq!(fst.final_weight(s2).unwrap(), None);
    ///
    /// fst.set_final(s1, BooleanWeight::one());
    /// assert_eq!(fst.final_weight(s1).unwrap(), Some(&BooleanWeight::one()));
    /// assert_eq!(fst.final_weight(s2).unwrap(), None);
    ///
    /// fst.set_final(s2, BooleanWeight::one());
    /// assert_eq!(fst.final_weight(s1).unwrap(), Some(&BooleanWeight::one()));
    /// assert_eq!(fst.final_weight(s2).unwrap(), Some(&BooleanWeight::one()));
    /// ```
    fn set_final<S: Into<Self::W>>(&mut self, state_id: StateId, final_weight: S) -> Result<()>;
    unsafe fn set_final_unchecked<S: Into<Self::W>>(&mut self, state_id: StateId, final_weight: S);

    /// Adds a new state to the current FST. The identifier of the new state is returned
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.num_states(), 0);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 1);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 2);
    ///
    /// ```
    fn add_state(&mut self) -> StateId;
    fn add_states(&mut self, n: usize);

    /// Removes a state from an FST. It also removes all the arcs starting from another state and
    /// reaching this state. An error is raised if the state `state_id` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 1);
    ///
    /// fst.del_state(s1);
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_state(&mut self, state_id: StateId) -> Result<()>;

    // TODO: Need to define a correct behaviour is the same state is present multiple times in the iterator
    /// Removes multiple states from an FST. If one of the states doesn't exist, an error is raised.
    ///
    /// # Warning
    ///
    /// This method modifies the id of the states that are left in the FST. Id that were used before
    /// calling this function should no longer be used.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 2);
    ///
    /// let states_to_remove = vec![s1, s2];
    /// fst.del_states(states_to_remove.into_iter());
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T) -> Result<()>;

    /// Remove all the states in the FST. As a result, all the arcs are also removed,
    /// as well as the start state and all the fina states.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 2);
    ///
    /// fst.del_all_states();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_all_states(&mut self);

    unsafe fn del_arcs_id_sorted_unchecked(&mut self, state: StateId, to_del: &Vec<usize>);

    /// Adds an arc to the FST. The arc will start in the state `source`.
    ///
    /// # Errors
    ///
    /// An error is raised if the state `source` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{ProbabilityWeight, Semiring};
    /// # use rustfst::Arc;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_arcs(s1)?, 0);
    /// fst.add_arc(s1, Arc::new(3, 5, 1.2, s2));
    /// assert_eq!(fst.num_arcs(s1)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn add_arc(&mut self, source: StateId, arc: Arc<Self::W>) -> Result<()>;
    unsafe fn add_arc_unchecked(&mut self, source: StateId, arc: Arc<Self::W>);

    /// Adds an arc to the FST. The arc will start in the state `source`.
    ///
    /// # Errors
    ///
    /// An error is raised if the state `source` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Arc;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_arcs(s1)?, 0);
    /// fst.emplace_arc(s1, 3, 5, 1.2, s2);
    /// assert_eq!(fst.num_arcs(s1)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn emplace_arc<S: Into<Self::W>>(
        &mut self,
        source: StateId,
        ilabel: Label,
        olabel: Label,
        weight: S,
        nextstate: StateId,
    ) -> Result<()> {
        self.add_arc(source, Arc::new(ilabel, olabel, weight, nextstate))
    }

    unsafe fn emplace_arc_unchecked<S: Into<Self::W>>(
        &mut self,
        source: StateId,
        ilabel: Label,
        olabel: Label,
        weight: S,
        nextstate: StateId,
    ) {
        self.add_arc_unchecked(source, Arc::new(ilabel, olabel, weight, nextstate))
    }

    unsafe fn set_arcs_unchecked(&mut self, source: StateId, arcs: Vec<Arc<Self::W>>);

    /// Remove the final weight of a specific state.
    fn delete_final_weight(&mut self, source: StateId) -> Result<()>;
    unsafe fn delete_final_weight_unchecked(&mut self, source: StateId);

    /// Deletes all the arcs leaving a state.
    fn delete_arcs(&mut self, source: StateId) -> Result<()>;

    /// Remove all arcs leaving a state and return them.
    fn pop_arcs(&mut self, source: StateId) -> Result<Vec<Arc<Self::W>>>;
    unsafe fn pop_arcs_unchecked(&mut self, source: StateId) -> Vec<Arc<Self::W>>;

    /// Retrieves a mutable reference to the final weight of a state (if the state is a final one).
    fn final_weight_mut(
        &mut self,
        state_id: StateId,
    ) -> Result<Option<&mut <Self as CoreFst>::W>>;

    unsafe fn final_weight_unchecked_mut(
        &mut self,
        state_id: StateId,
    ) -> Option<&mut <Self as CoreFst>::W>;

    /// Takes the final weight out of the fst, leaving a None in its place.
    ///
    /// # Errors
    ///
    /// An error is raised if the state with id `state_id` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Arc;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// fst.set_final(s1, 1.2)?;
    ///
    /// assert_eq!(fst.final_weight(s1)?, Some(&ProbabilityWeight::new(1.2)));
    /// let weight = fst.take_final_weight(s1)?;
    /// assert_eq!(weight, Some(ProbabilityWeight::new(1.2)));
    /// assert_eq!(fst.final_weight(s1)?, None);
    /// # Ok(())
    /// # }
    /// ```
    fn take_final_weight(&mut self, state_id: StateId) -> Result<Option<Self::W>>;

    /// Takes the final weight out of the fst, leaving a None in its place.
    /// This version leads to `undefined behaviour` if the state doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Arc;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// fst.set_final(s1, 1.2)?;
    ///
    /// assert_eq!(fst.final_weight(s1)?, Some(&ProbabilityWeight::new(1.2)));
    /// let weight = unsafe {fst.take_final_weight_unchecked(s1)};
    /// assert_eq!(weight, Some(ProbabilityWeight::new(1.2)));
    /// assert_eq!(fst.final_weight(s1)?, None);
    /// # Ok(())
    /// # }
    /// ```
    unsafe fn take_final_weight_unchecked(&mut self, state_id: StateId) -> Option<Self::W>;

    fn sort_arcs_unchecked<F: Fn(&Arc<Self::W>, &Arc<Self::W>) -> Ordering>(
        &mut self,
        state: StateId,
        f: F,
    );

    unsafe fn unique_arcs_unchecked(&mut self, state: StateId);

    unsafe fn sum_arcs_unchecked(&mut self, state: StateId);

    /// This operation computes the concatenative closure.
    /// If A transduces string `x` to `y` with weight `a`,
    /// then the closure transduces `x` to `y` with weight `a`,
    /// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
    ///  If closure_star then the empty string is transduced to itself with weight `1` as well.
    fn closure(&mut self, closure_type: ClosureType) {
        crate::algorithms::closure(self, closure_type)
    }

    /// Maps an arc using a `ArcMapper` object.
    fn arc_map<M: ArcMapper<Self::W>>(&mut self, mapper: &mut M) -> Result<()> {
        crate::algorithms::arc_map(self, mapper)
    }
}

/// Iterate over mutable arcs in a wFST.
pub trait MutableArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    type IterMut: Iterator<Item = &'a mut Arc<Self::W>>;
    fn arcs_iter_mut(&'a mut self, state_id: StateId) -> Result<Self::IterMut>;
    unsafe fn arcs_iter_unchecked_mut(&'a mut self, state_id: StateId) -> Self::IterMut;
}
