use crate::runtime::model::network::OperationalModel;

pub struct ConstraintEvaluation {
    pub mandatory: bool,
    pub advisory_score: f64,
}

pub trait ConstraintModel<M: OperationalModel> {
}
