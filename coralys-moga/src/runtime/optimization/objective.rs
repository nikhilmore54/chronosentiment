use crate::runtime::model::network::OperationalModel;

pub trait DecisionVector {}

pub trait ObjectiveModel<M: OperationalModel> {}
