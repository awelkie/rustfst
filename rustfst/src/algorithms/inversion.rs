use std::mem::swap;

use crate::fst_traits::{ExpandedFst, MutableFst};

/// This operation inverts the transduction corresponding to an FST
/// by exchanging the FST's input and output labels.
///
/// # Example 1
/// ```
/// # use rustfst::fst;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::invert;
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// invert(&mut fst);
///
/// assert_eq!(fst, fst![3 => 2]);
/// ```
///
/// # Example 2
///
/// ## Input
///
/// ![invert_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/invert_in.svg?sanitize=true)
///
/// ## Invert
///
/// ![invert_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/invert_out.svg?sanitize=true)
///
pub fn invert<F: ExpandedFst + MutableFst>(fst: &mut F) {
    for state in 0..fst.num_states() {
        for arc in unsafe { fst.arcs_iter_unchecked_mut(state) } {
            swap(&mut arc.ilabel, &mut arc.olabel);
        }
    }
}
