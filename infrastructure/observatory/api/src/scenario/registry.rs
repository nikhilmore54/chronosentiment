use super::domain::ScenarioDomain;

/// Declarative scenario authority — registration and lookup only.
/// MUST NOT embed policy (no domain selection heuristics).
#[derive(Debug, Clone, Default)]
pub struct ScenarioRegistry {
    domains: Vec<ScenarioDomain>,
}

impl ScenarioRegistry {
    pub fn new() -> Self {
        Self {
            domains: Vec::new(),
        }
    }

    /// Phase C v1 default registry (see fixtures/contracts/scenario_registry.json).
    pub fn v1_default() -> Self {
        let mut registry = Self::new();
        registry.register(ScenarioDomain::certified_fixture(
            "deterministic_demo",
            "deterministic_demo_v1",
        ));
        registry.register(ScenarioDomain::certified_fixture(
            "deterministic_demo_execution",
            "deterministic_demo_v1_execution_path",
        ));
        registry
    }

    pub fn register(&mut self, domain: ScenarioDomain) {
        if self.domains.iter().any(|d| d.id == domain.id) {
            return;
        }
        self.domains.push(domain);
    }

    pub fn get(&self, id: &str) -> Option<&ScenarioDomain> {
        self.domains.iter().find(|d| d.id == id)
    }

    pub fn list_eligible(&self) -> Vec<&ScenarioDomain> {
        self.domains
            .iter()
            .filter(|d| d.evaluation_eligible)
            .collect()
    }

    pub fn all(&self) -> &[ScenarioDomain] {
        &self.domains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_declarative_only() {
        let registry = ScenarioRegistry::v1_default();
        assert!(registry.get("deterministic_demo").is_some());
        assert_eq!(registry.list_eligible().len(), 2);
    }
}
