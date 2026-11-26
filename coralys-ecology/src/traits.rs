pub trait TopologyModel<T> {
    type Output;

    fn transform(&self, input: T) -> Self::Output;
}

pub trait MemoryModel<T> {
    type State;

    fn observe(&mut self, value: T);

    fn state(&self) -> &Self::State;
}
