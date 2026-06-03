use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcScenario {
    pub id: String,
    #[serde(rename = "numberOfWeeks")]
    pub number_of_weeks: usize,
    pub skills: Vec<String>,
    #[serde(rename = "shiftTypes")]
    pub shift_types: Vec<InrcShiftType>,
    #[serde(rename = "forbiddenShiftTypeSuccessions")]
    pub forbidden_shift_type_successions: Vec<InrcForbiddenSuccession>,
    pub contracts: Vec<InrcContract>,
    pub nurses: Vec<InrcNurse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcShiftType {
    pub id: String,
    #[serde(rename = "minimumNumberOfConsecutiveAssignments")]
    pub min_consecutive: usize,
    #[serde(rename = "maximumNumberOfConsecutiveAssignments")]
    pub max_consecutive: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcForbiddenSuccession {
    #[serde(rename = "precedingShiftType")]
    pub preceding: String,
    #[serde(rename = "succeedingShiftTypes")]
    pub succeeding: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcContract {
    pub id: String,
    #[serde(rename = "minimumNumberOfAssignments")]
    pub min_assignments: usize,
    #[serde(rename = "maximumNumberOfAssignments")]
    pub max_assignments: usize,
    #[serde(rename = "minimumNumberOfConsecutiveWorkingDays")]
    pub min_consecutive_working_days: usize,
    #[serde(rename = "maximumNumberOfConsecutiveWorkingDays")]
    pub max_consecutive_working_days: usize,
    #[serde(rename = "minimumNumberOfConsecutiveDaysOff")]
    pub min_consecutive_days_off: usize,
    #[serde(rename = "maximumNumberOfConsecutiveDaysOff")]
    pub max_consecutive_days_off: usize,
    #[serde(rename = "maximumNumberOfWorkingWeekends")]
    pub max_working_weekends: usize,
    #[serde(rename = "completeWeekends")]
    pub complete_weekends: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcNurse {
    pub id: String,
    pub contract: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcHistory {
    pub week: usize,
    pub scenario: String,
    #[serde(rename = "nurseHistory")]
    pub nurse_history: Vec<InrcNurseHistory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcNurseHistory {
    pub nurse: String,
    #[serde(rename = "numberOfAssignments")]
    pub number_of_assignments: usize,
    #[serde(rename = "numberOfWorkingWeekends")]
    pub number_of_working_weekends: usize,
    #[serde(rename = "lastAssignedShiftType")]
    pub last_assigned_shift_type: String,
    #[serde(rename = "numberOfConsecutiveAssignments")]
    pub number_of_consecutive_assignments: usize,
    #[serde(rename = "numberOfConsecutiveWorkingDays")]
    pub number_of_consecutive_working_days: usize,
    #[serde(rename = "numberOfConsecutiveDaysOff")]
    pub number_of_consecutive_days_off: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcWeekData {
    pub scenario: String,
    pub requirements: Vec<InrcRequirement>,
    #[serde(rename = "shiftOffRequests")]
    pub shift_off_requests: Vec<InrcShiftOffRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcRequirement {
    #[serde(rename = "shiftType")]
    pub shift_type: String,
    pub skill: String,
    #[serde(rename = "requirementOnMonday")]
    pub monday: InrcRequirementLevel,
    #[serde(rename = "requirementOnTuesday")]
    pub tuesday: InrcRequirementLevel,
    #[serde(rename = "requirementOnWednesday")]
    pub wednesday: InrcRequirementLevel,
    #[serde(rename = "requirementOnThursday")]
    pub thursday: InrcRequirementLevel,
    #[serde(rename = "requirementOnFriday")]
    pub friday: InrcRequirementLevel,
    #[serde(rename = "requirementOnSaturday")]
    pub saturday: InrcRequirementLevel,
    #[serde(rename = "requirementOnSunday")]
    pub sunday: InrcRequirementLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcRequirementLevel {
    pub minimum: usize,
    pub optimal: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InrcShiftOffRequest {
    pub nurse: String,
    #[serde(rename = "shiftType")]
    pub shift_type: String,
    pub day: String,
}
