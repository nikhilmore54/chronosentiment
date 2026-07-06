use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Skill(pub String);

impl Skill {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Skill(s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worker {
    pub id: u64,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shift {
    pub id: u64,
    pub start_hour: u64, // simplified time: hour of the week (0..168)
    pub duration_hours: u64,
    pub required_skill: Skill,
}

impl Shift {
    pub fn end_hour(&self) -> u64 {
        self.start_hour + self.duration_hours
    }

    pub fn overlaps_with(&self, other: &Shift) -> bool {
        self.start_hour < other.end_hour() && other.start_hour < self.end_hour()
    }
}
