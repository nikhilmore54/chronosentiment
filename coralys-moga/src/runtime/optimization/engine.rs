use crate::runtime::model::network::OperationalModel;

pub trait OptimizationEngine<M: OperationalModel> {
    fn optimize(&mut self);
}
