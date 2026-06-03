pub trait CandidateEvaluator<C> {
    type Evaluation;

    fn evaluate(&self, candidate: &C) -> Self::Evaluation;
}

pub trait DecisionMaker<E> {
    type Decision;

    fn decide(&self, evaluation: &E) -> Self::Decision;
}

pub trait DecisionPolicy<D> {
    fn accept(&self, decision: &D) -> bool;
}
