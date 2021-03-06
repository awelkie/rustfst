use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;

use crate::SymbolTable;
use anyhow::{format_err, Result};
use std::rc::Rc;

impl<W: Semiring + 'static> Fst for ConstFst<W> {
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.isymt.clone()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.isymt = Some(Rc::clone(&symt))
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.osymt = Some(Rc::clone(&symt));
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.isymt.take()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<W: Semiring> CoreFst for ConstFst<W> {
    type W = W;

    fn start(&self) -> Option<usize> {
        self.start
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
        let s = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.as_ref())
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.states.get_unchecked(state_id).final_weight.as_ref()
    }

    fn num_arcs(&self, s: usize) -> Result<usize> {
        let const_state = self
            .states
            .get(s)
            .ok_or_else(|| format_err!("State doesn't exist"))?;
        Ok(const_state.narcs)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.states.get_unchecked(s).narcs
    }
}
