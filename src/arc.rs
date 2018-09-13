use Label;
use StateId;
use semiring::Semiring;

pub trait Arc<W: Semiring>  {
    fn ilabel(&self) -> Label;
    fn olabel(&self) -> Label;
    fn weight(&self) -> W;
    fn nextstate(&self) -> StateId;
}

pub struct StdArc<W: Semiring> {
	ilabel: Label,
	olabel: Label,
	weight: W,
	nextstate: StateId,
}

impl<W: Semiring> Arc<W> for StdArc<W> {
	fn ilabel(&self) -> Label {
		self.ilabel
	}
	fn olabel(&self) -> Label {
		self.olabel
	}
	fn weight(&self) -> W {
		self.weight.clone()
	}
	fn nextstate(&self) -> StateId {
		self.nextstate
	}
}