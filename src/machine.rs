pub(crate) trait Machine {
    type I;
    type O;
    fn build(&mut self, i: Self::I);
    fn step(&mut self, i: Self::I);
    fn halt(self);
}
