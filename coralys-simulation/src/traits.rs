pub trait Action {}

pub trait Environment {
    type State;
}

pub trait Policy<E, A>
where
    E: Environment,
    A: Action,
{
    fn choose_action(&self, state: &E::State) -> A;
}

pub trait Simulator<E, A>
where
    E: Environment,
    A: Action,
{
    type Outcome;

    fn step(&mut self, action: A) -> Self::Outcome;
}
