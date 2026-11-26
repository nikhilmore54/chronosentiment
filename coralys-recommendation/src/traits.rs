pub trait Ranker<T> {
    fn rank(&self, candidates: Vec<T>) -> Vec<T>;
}

pub trait Explainer<T> {
    type Explanation;

    fn explain(&self, item: &T) -> Self::Explanation;
}
