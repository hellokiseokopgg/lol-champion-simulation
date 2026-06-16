use itertools::Itertools;

/// A simple optimizer that generates permutations of APL action priorities.
pub struct APLOptimizer {
    pub base_actions: Vec<String>,
    pub permutable_actions: Vec<String>,
}

impl APLOptimizer {
    pub fn new(base_actions: Vec<&str>, permutable_actions: Vec<&str>) -> Self {
        Self {
            base_actions: base_actions.into_iter().map(|s| s.to_string()).collect(),
            permutable_actions: permutable_actions.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Generates all permutations of the permutable actions, prepended by the base actions.
    pub fn generate_permutations(&self) -> Vec<String> {
        let mut results = Vec::new();
        
        let perms = self.permutable_actions.iter().permutations(self.permutable_actions.len());
        
        for perm in perms {
            let mut lines = Vec::new();
            lines.extend(self.base_actions.clone());
            lines.extend(perm.into_iter().cloned());
            results.push(lines.join("\n"));
        }
        
        results
    }
}
