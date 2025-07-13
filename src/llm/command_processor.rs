use super::phi_model::PhiModel;
use anyhow::{Result, Context};
use std::collections::HashMap;

pub struct CommandProcessor {
    phi_model: PhiModel,
    command_cache: HashMap<String, String>,
}

impl CommandProcessor {
    pub async fn new(base_url: impl Into<String>, model_name: impl Into<String>) -> Result<Self> {
        log::debug!("🔄 Connecting to Ollama...");
        let phi_model = PhiModel::new(base_url, model_name).await
            .context("Failed to initialize Ollama client")?;
        log::debug!("✅ Connected to Ollama!");
        
        Ok(Self {
            phi_model,
            command_cache: HashMap::new(),
        })
    }
    
    pub async fn process_query(&mut self, query: &str) -> Result<String> {
        // Check cache first
        if let Some(cached_command) = self.command_cache.get(query) {
            return Ok(cached_command.clone());
        }
        
        // Generate command using Ollama
        let command = self.phi_model.generate_command(query).await
            .context("Failed to generate command with Ollama")?;
        
        // Post-process the command to ensure it's clean
        let cleaned_command = self.clean_command(&command);
        
        // Cache the result
        self.command_cache.insert(query.to_string(), cleaned_command.clone());
        
        Ok(cleaned_command)
    }
    
    fn clean_command(&self, command: &str) -> String {
        let mut cleaned = command.trim();
        
        // Remove common prefixes that might be generated
        let prefixes_to_remove = [
            "$ ",
            "# ",
            "> ",
            "bash: ",
            "shell: ",
            "command: ",
        ];
        
        for prefix in &prefixes_to_remove {
            if cleaned.starts_with(prefix) {
                cleaned = &cleaned[prefix.len()..];
            }
        }
        
        // Remove quotes if they wrap the entire command
        if cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() > 2 {
            cleaned = &cleaned[1..cleaned.len()-1];
        }
        if cleaned.starts_with('\'') && cleaned.ends_with('\'') && cleaned.len() > 2 {
            cleaned = &cleaned[1..cleaned.len()-1];
        }
        
        cleaned.to_string()
    }
    
    pub fn get_cache_stats(&self) -> (usize, Vec<String>) {
        let count = self.command_cache.len();
        let queries: Vec<String> = self.command_cache.keys().cloned().collect();
        (count, queries)
    }
    
    pub fn clear_cache(&mut self) {
        self.command_cache.clear();
    }
}