pub trait AssignmentSolver {
    type Worker;
    type Demand;
    type Matching;

    fn assign(&self, workers: &[Self::Worker], demands: &[Self::Demand]) -> Self::Matching;
}
