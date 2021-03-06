use crate::algorithms::ReplaceFst;
use crate::arc::Arc;
use crate::fst_traits::{
    AllocableFst, ArcIterator, CoreFst, FinalStatesIterator, Fst, FstIterator, MutableFst,
    StateIterator,
};
use crate::semirings::Semiring;
use crate::{SymbolTable, EPS_LABEL};
use anyhow::Result;
use std::rc::Rc;

/// Defines the different types of closure : Star or Plus.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ClosureType {
    ClosureStar,
    ClosurePlus,
}

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
///  If closure_star then the empty string is transduced to itself with weight `1` as well.
///
/// # Example
///
/// ## Input
/// ![closure_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_in.svg?sanitize=true)
///
/// ## Closure Plus
/// ![closure_out_closure_plus](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_plus.svg?sanitize=true)
///
/// ## Closure Star
/// ![closure_out_closure_star](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_star.svg?sanitize=true)
pub fn closure<F>(fst: &mut F, closure_type: ClosureType)
where
    F: MutableFst,
{
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst
            .final_states_iter()
            .map(|u| (u.state_id, u.final_weight.clone()))
            .collect();
        for (final_state_id, final_weight) in final_states_id {
            unsafe {
                fst.add_arc_unchecked(
                    final_state_id,
                    Arc::new(EPS_LABEL, EPS_LABEL, final_weight, start_state),
                )
            };
        }
    }

    if closure_type == ClosureType::ClosureStar {
        let nstart = fst.add_state();

        // Add a new start state to allow empty path
        if let Some(start_state_id) = fst.start() {
            unsafe {
                fst.add_arc_unchecked(
                    nstart,
                    Arc::new(
                        EPS_LABEL,
                        EPS_LABEL,
                        <F as CoreFst>::W::one(),
                        start_state_id,
                    ),
                );
            }
        }

        unsafe {
            fst.set_start_unchecked(nstart);
            fst.set_final_unchecked(nstart, F::W::one());
        }
    }
}

/// Computes the concatenative closure. This version is a delayed FST. If an FST
/// transduces string x to y with weight a, then its closure transduces x to y
/// with weight a, xx to yy with weight Times(a, a), xxx to yyy with weight
/// Times(Times(a, a), a), etc. If closure_type == CLOSURE_STAR, then the empty
/// string is transduced to itself with weight Weight::One() as well.
#[derive(Debug, PartialEq)]
pub struct ClosureFst<F: Fst + 'static>(ReplaceFst<F, F>)
where
    F::W: 'static;

impl<F: Fst + MutableFst + AllocableFst> ClosureFst<F>
where
    F::W: 'static,
{
    //TODO: Use a borrow and not a move
    //TODO: Allow fsts of different types
    pub fn new(fst: F, closure_type: ClosureType) -> Result<Self> {
        let mut rfst = F::new();
        if let Some(isymt) = fst.input_symbols() {
            rfst.set_input_symbols(isymt);
        }
        if let Some(osymt) = fst.output_symbols() {
            rfst.set_output_symbols(osymt);
        }
        match closure_type {
            ClosureType::ClosureStar => {
                rfst.add_state();
                unsafe {
                    rfst.set_start_unchecked(0);
                    rfst.set_final_unchecked(0, F::W::one());
                    rfst.add_arc_unchecked(0, Arc::new(EPS_LABEL, std::usize::MAX, F::W::one(), 0));
                }
            }
            ClosureType::ClosurePlus => {
                rfst.add_states(2);
                unsafe {
                    rfst.set_start_unchecked(0);
                    rfst.set_final_unchecked(1, F::W::one());
                    rfst.add_arc_unchecked(0, Arc::new(EPS_LABEL, std::usize::MAX, F::W::one(), 1));
                    rfst.add_arc_unchecked(1, Arc::new(EPS_LABEL, EPS_LABEL, F::W::one(), 0));
                }
            }
        };
        let mut fst_tuples = Vec::with_capacity(3);
        fst_tuples.push((0, rfst));
        fst_tuples.push((std::usize::MAX, fst));

        Ok(ClosureFst(ReplaceFst::new(fst_tuples, 0, false)?))
    }
}

impl<F: Fst> CoreFst for ClosureFst<F>
where
    F::W: 'static,
{
    type W = F::W;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_arcs(&self, s: usize) -> Result<usize> {
        self.0.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.0.num_arcs_unchecked(s)
    }
}

impl<'a, F: Fst + 'static> StateIterator<'a> for ClosureFst<F>
where
    F::W: 'static,
{
    type Iter = <ReplaceFst<F, F> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, F: Fst + 'static> ArcIterator<'a> for ClosureFst<F>
where
    F::W: 'static,
{
    type Iter = <ReplaceFst<F, F> as ArcIterator<'a>>::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
        self.0.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.0.arcs_iter_unchecked(state_id)
    }
}

impl<F: Fst + 'static> Fst for ClosureFst<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.0.unset_input_symbols()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.0.unset_output_symbols()
    }
}

impl<'a, F: Fst + 'static> FstIterator<'a> for ClosureFst<F>
where
    F::W: 'static,
{
    type ArcsIter = <ReplaceFst<F, F> as FstIterator<'a>>::ArcsIter;
    type FstIter = <ReplaceFst<F, F> as FstIterator<'a>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}
