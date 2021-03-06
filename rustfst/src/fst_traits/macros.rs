macro_rules! display_single_state {
    ($fst:expr, $state_id:expr, $f: expr, $show_weight_one: expr) => {
        for arc in $fst.arcs_iter($state_id).unwrap() {
            if arc.weight.is_one() && !$show_weight_one {
                writeln!(
                    $f,
                    "{}\t{}\t{}\t{}",
                    $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel
                )?;
            } else {
                writeln!(
                    $f,
                    "{}\t{}\t{}\t{}\t{}",
                    $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel, &arc.weight
                )?;
            }
        }
    };
}

macro_rules! write_fst {
    ($fst:expr, $f:expr, $show_weight_one: expr) => {
        if let Some(start_state) = $fst.start() {
            // Firstly print the arcs leaving the start state
            display_single_state!($fst, start_state, $f, $show_weight_one);

            // Secondly, print the arcs leaving all the other states
            for state_id in $fst.states_iter() {
                if state_id != start_state {
                    display_single_state!($fst, state_id, $f, $show_weight_one);
                }
            }

            // Finally, print the final states with their weight
            for final_state in $fst.final_states_iter() {
                if final_state.final_weight.is_one() && !$show_weight_one {
                    writeln!($f, "{}", &final_state.state_id)?;
                } else {
                    writeln!(
                        $f,
                        "{}\t{}",
                        &final_state.state_id, &final_state.final_weight
                    )?;
                }
            }
        }
    };
}

macro_rules! display_fst_trait {
    ($semiring:tt, $fst_type:ty) => {
        impl<$semiring: 'static + SerializableSemiring> fmt::Display for $fst_type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write_fst!(self, f, true);
                Ok(())
            }
        }
    };
}
