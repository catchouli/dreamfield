use std::collections::HashMap;

// Model manager
pub struct ModelManager {
    entries: HashMap<String, &'static [u8]>
}

impl ModelManager {
    pub fn new_with_models(sources: Vec<(&str, &'static [u8])>) -> Self {
        let entries = sources.into_iter().map(|(name, model)| {
            (name.to_string(), model)
        }).collect();

        Self {
            entries
        }
    }

    pub fn get(&self, name: &str) -> Result<&&[u8], String> {
        self.entries
            .get(name)
            .ok_or(format!("No such model {}", name))
    }
}
