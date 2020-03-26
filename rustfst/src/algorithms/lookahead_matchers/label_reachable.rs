use std::collections::hash_map::Entry;
use std::collections::HashMap;

use failure::Fallible;
use itertools::Itertools;

use crate::algorithms::arc_compares::{ilabel_compare, olabel_compare};
use crate::algorithms::lookahead_matchers::interval_set::IntervalSet;
use crate::algorithms::lookahead_matchers::state_reachable::StateReachable;
use crate::algorithms::{arc_sort, fst_convert_from_ref};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableArcIterator, MutableFst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, EPS_LABEL, NO_LABEL, UNASSIGNED};

#[derive(Debug, Clone)]
pub struct LabelReachableData {
    reach_input: bool,
    final_label: Label,
    label2index: HashMap<Label, Label>,
    interval_sets: Vec<IntervalSet>,
}

impl LabelReachableData {
    pub fn new(reach_input: bool) -> Self {
        Self {
            reach_input,
            final_label: NO_LABEL,
            label2index: HashMap::new(),
            interval_sets: Vec::new(),
        }
    }

    pub fn interval_set(&self, s: StateId) -> Fallible<&IntervalSet> {
        self.interval_sets
            .get(s)
            .ok_or_else(|| format_err!("Missing state {}", s))
    }

    pub fn final_label(&self) -> Label {
        self.final_label
    }

    pub fn label2index(&self) -> &HashMap<Label, Label> {
        &self.label2index
    }

    pub fn reach_input(&self) -> bool {
        self.reach_input
    }
}

#[derive(Debug)]
pub struct LabelReachable {
    data: LabelReachableData,
    label2state: HashMap<Label, StateId>,
    reach_fst_input: bool,
}

impl LabelReachable {
    pub fn new<F: Fst>(fst: &F, reach_input: bool) -> Fallible<Self>
    where
        F::W: 'static,
    {
        let mut fst: VectorFst<_> = fst_convert_from_ref(fst);

        let mut label_reachable = Self {
            data: LabelReachableData::new(reach_input),
            label2state: HashMap::new(),
            reach_fst_input: false,
        };

        let nstates = fst.num_states();
        label_reachable.transform_fst(&mut fst);
        label_reachable.find_intervals(&fst, nstates)?;

        Ok(label_reachable)
    }

    pub fn new_from_data(data: LabelReachableData) -> Self {
        Self {
            data,
            label2state: HashMap::new(),
            reach_fst_input: false,
        }
    }

    pub fn reach_input(&self) -> bool {
        self.data.reach_input
    }

    // Redirects labeled arcs (input or output labels determined by ReachInput())
    // to new label-specific final states. Each original final state is
    // redirected via a transition labeled with kNoLabel to a new
    // kNoLabel-specific final state. Creates super-initial state for all states
    // with zero in-degree.
    fn transform_fst<W: Semiring + 'static>(&mut self, fst: &mut VectorFst<W>) {
        let ins = fst.num_states();
        let mut ons = ins;
        let mut indeg = vec![0; ins];
        // Redirects labeled arcs to new final states.
        for s in 0..ins {
            for arc in unsafe { fst.arcs_iter_unchecked_mut(s) } {
                let label = if self.data.reach_input {
                    arc.ilabel
                } else {
                    arc.olabel
                };
                if label != EPS_LABEL {
                    arc.nextstate = match self.label2state.entry(label) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                }
                indeg[arc.nextstate] += 1;
            }

            if let Some(final_weight) = unsafe { fst.final_weight_unchecked(s) } {
                if !final_weight.is_zero() {
                    let nextstate = match self.label2state.entry(NO_LABEL) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                    unsafe {
                        fst.add_arc_unchecked(
                            s,
                            Arc::new(NO_LABEL, NO_LABEL, final_weight.clone(), nextstate),
                        )
                    };
                    indeg[nextstate] += 1;
                    unsafe { fst.delete_final_weight_unchecked(s) }
                }
            }
        }

        // Adds new final states to the FST.
        while fst.num_states() < ons {
            let s = fst.add_state();
            unsafe { fst.set_final_unchecked(s, W::one()) };
        }

        // Creates a super-initial state for all states with zero in-degree.
        let start = fst.add_state();
        unsafe { fst.set_start_unchecked(start) };
        for s in 0..start {
            if indeg[s] == 0 {
                unsafe { fst.add_arc_unchecked(start, Arc::new(0, 0, W::one(), s)) };
            }
        }
    }

    fn find_intervals<W: Semiring + 'static>(
        &mut self,
        fst: &VectorFst<W>,
        ins: StateId,
    ) -> Fallible<()> {
        let state_reachable = StateReachable::new(fst)?;
        let state2index = &state_reachable.state2index;
        let interval_sets = &mut self.data.interval_sets;
        *interval_sets = state_reachable.isets;
        interval_sets.resize_with(ins, IntervalSet::default);
        let label2index = &mut self.data.label2index;
        for (label, state) in self.label2state.iter() {
            let i = state2index[*state];
            label2index.insert(*label, i);
            if *label == NO_LABEL {
                self.data.final_label = i;
            }
        }
        self.label2state.clear();
        Ok(())
    }

    pub fn relabel(&self, label: Label) -> Label {
        if label == EPS_LABEL {
            return EPS_LABEL;
        }
        let label2index = self.data.label2index();
        let mut relabel = label2index[&label];
        if relabel == EPS_LABEL {
            relabel = label2index.len() + 1; // Adds a new label
        }
        relabel
    }

    pub fn relabel_fst<F: MutableFst>(&self, fst: &mut F, relabel_input: bool) -> Fallible<()> {
        for fst_data in fst.fst_iter_mut() {
            for arc in fst_data.arcs {
                if relabel_input {
                    arc.ilabel = self.relabel(arc.ilabel);
                } else {
                    arc.olabel = self.relabel(arc.olabel);
                }
            }
        }

        if relabel_input {
            arc_sort(fst, ilabel_compare);
            fst.unset_input_symbols();
        } else {
            arc_sort(fst, olabel_compare);
            fst.unset_output_symbols();
        }

        Ok(())
    }

    // Returns relabeling pairs (cf. relabel.h::Relabel()). If avoid_collisions is
    // true, extra pairs are added to ensure no collisions when relabeling
    // automata that have labels unseen here.
    pub fn relabel_pairs(&self, avoid_collisions: bool) -> Vec<(Label, Label)> {
        let mut pairs = vec![];
        let label2index = self.data.label2index();
        for (key, val) in label2index.iter() {
            if *val != self.data.final_label() {
                pairs.push((*key, *val));
            }
        }

        if avoid_collisions {
            for i in 1..=label2index.len() {
                let it = label2index.get(&i);
                if it.is_none() || it.unwrap() == &self.data.final_label() {
                    pairs.push((i, label2index.len() + 1));
                }
            }
        }

        pairs
    }

    pub fn reach_init<F: ExpandedFst>(&mut self, fst: &F, reach_input: bool) -> Fallible<()> {
        self.reach_fst_input = reach_input;
        let props = fst.properties()?;

        let true_prop = if self.reach_fst_input {
            FstProperties::I_LABEL_SORTED
        } else {
            FstProperties::O_LABEL_SORTED
        };

        if !props.contains(true_prop) {
            bail!("LabelReachable::ReachInit: Fst is not sorted")
        }
        Ok(())
    }

    // Can reach this label from current state?
    // Original labels must be transformed by the Relabel methods above.
    pub fn reach_label(&self, current_state: StateId, label: Label) -> Fallible<bool> {
        if label == EPS_LABEL {
            return Ok(false);
        }
        Ok(self.data.interval_set(current_state)?.member(label))
    }

    // Can reach final state (via epsilon transitions) from this state?
    pub fn reach_final(&self, current_state: StateId) -> Fallible<bool> {
        Ok(self
            .data
            .interval_set(current_state)?
            .member(self.data.final_label()))
    }

    pub fn reach<'a, W: Semiring + 'a>(
        &self,
        current_state: StateId,
        aiter: impl Iterator<Item = &'a Arc<W>>,
        aiter_begin: usize,
        aiter_end: usize,
        compute_weight: bool,
    ) -> Fallible<Option<(usize, usize, W)>> {
        let mut reach_begin = UNASSIGNED;
        let mut reach_end = UNASSIGNED;
        let mut reach_weight = W::zero();
        let interval_set = self.data.interval_set(current_state)?;
        if 2 * (aiter_end - aiter_begin) < interval_set.len() {
            let aiter = aiter.skip(aiter_begin);
            let mut reach_label = NO_LABEL;
            for (pos, arc) in aiter.take(aiter_end).enumerate() {
                let aiter_pos = aiter_begin + pos;
                let label = if self.reach_fst_input {
                    arc.ilabel
                } else {
                    arc.olabel
                };
                if label == reach_label || self.reach_label(current_state, label)? {
                    reach_label = label;
                    if reach_begin == UNASSIGNED {
                        reach_begin = aiter_pos;
                    }
                    reach_end = aiter_pos + 1;
                    if compute_weight {
                        reach_weight.plus_assign(&arc.weight)?;
                    }
                }
            }
        } else {
            let mut begin_low;
            let mut end_low = aiter_begin;

            let arcs = aiter.collect_vec();
            for interval in interval_set.iter() {
                begin_low = self.lower_bound(arcs.as_slice(), end_low, aiter_end, interval.begin);
                end_low = self.lower_bound(arcs.as_slice(), begin_low, aiter_end, interval.end);
                if end_low - begin_low > 0 {
                    if reach_begin == UNASSIGNED {
                        reach_begin = begin_low;
                    }
                    reach_end = end_low;
                    if compute_weight {
                        for i in begin_low..end_low {
                            reach_weight.plus_assign(&arcs[i].weight)?;
                        }
                    }
                }
            }
        }

        if reach_begin != UNASSIGNED {
            Ok(Some((reach_begin, reach_end, reach_weight)))
        } else {
            Ok(None)
        }
    }

    fn lower_bound<W: Semiring>(
        &self,
        arcs: &[&Arc<W>],
        aiter_begin: usize,
        aiter_end: usize,
        match_label: Label,
    ) -> usize {
        let mut low = aiter_begin;
        let mut high = aiter_end;
        while low < high {
            let mid = low + (high - low) / 2;
            let arc = arcs[mid];
            let label = if self.reach_fst_input {
                arc.ilabel
            } else {
                arc.olabel
            };
            if label < match_label {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        low
    }
}
