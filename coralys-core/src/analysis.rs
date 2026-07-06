pub trait InstanceFeatures: Send + Sync + std::fmt::Debug {}

pub trait DifficultyAssessment: Send + Sync + std::fmt::Debug {
    fn difficulty_label(&self) -> &'static str;
}

pub trait ConfigurationPolicy: Send + Sync + std::fmt::Debug {}

pub trait InstanceAnalyzer<I>: Send + Sync {
    type Features: InstanceFeatures;
    type Difficulty: DifficultyAssessment;
    type Policy: ConfigurationPolicy;

    fn analyze(&self, instance: &I) -> Result<(Self::Features, Self::Difficulty, Self::Policy), String>;
}
