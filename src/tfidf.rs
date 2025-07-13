use std::collections::HashMap;
use anyhow::Result;

// TF-IDF constants
pub const FUZZY_MATCH_THRESHOLD: f64 = 0.3; // Minimum similarity score to suggest

// TF-IDF vector index structure
#[derive(Debug, Clone)]
pub struct TfIdfIndex {
    pub vocabulary: HashMap<String, usize>,
    pub document_frequencies: Vec<f64>,
    pub tfidf_vectors: Vec<HashMap<usize, f64>>,
    pub entity_names: Vec<String>, // Task names or aide names
    pub total_docs: usize,
}

// Fuzzy match result structure
#[derive(Debug)]
pub struct FuzzyMatchResult {
    pub exact_match: bool,
    pub suggested_name: Option<String>,
    pub score: Option<f64>,
}

// TF-IDF helper functions
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|s| s.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect())
        .filter(|s: &String| !s.is_empty())
        .collect()
}

pub fn calculate_tf(tokens: &[String], vocab: &HashMap<String, usize>) -> HashMap<usize, f64> {
    let mut tf = HashMap::new();
    let doc_length = tokens.len() as f64;
    
    for token in tokens {
        if let Some(&word_id) = vocab.get(token) {
            *tf.entry(word_id).or_insert(0.0) += 1.0;
        }
    }
    
    // Normalize by document length
    for (_, count) in tf.iter_mut() {
        *count /= doc_length;
    }
    
    tf
}

pub fn cosine_similarity(vec1: &HashMap<usize, f64>, vec2: &HashMap<usize, f64>) -> f64 {
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;
    
    for (&key, &val1) in vec1 {
        dot_product += val1 * vec2.get(&key).unwrap_or(&0.0);
        norm1 += val1 * val1;
    }
    
    for &val2 in vec2.values() {
        norm2 += val2 * val2;
    }
    
    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot_product / (norm1.sqrt() * norm2.sqrt())
    }
}

// Build TF-IDF index from a list of names
pub fn build_tfidf_index(names: Vec<String>) -> Result<TfIdfIndex> {
    if names.is_empty() {
        return Ok(TfIdfIndex {
            vocabulary: HashMap::new(),
            document_frequencies: Vec::new(),
            tfidf_vectors: Vec::new(),
            entity_names: Vec::new(),
            total_docs: 0,
        });
    }
    
    let documents: Vec<String> = names.iter().map(|name| name.clone()).collect();
    let total_docs = documents.len();
    
    // Build vocabulary
    let mut vocabulary = HashMap::new();
    let mut word_doc_count = HashMap::new();
    
    for (_, doc) in documents.iter().enumerate() {
        let tokens = tokenize(doc);
        let mut unique_tokens = std::collections::HashSet::new();
        
        for token in tokens {
            if !vocabulary.contains_key(&token) {
                vocabulary.insert(token.clone(), vocabulary.len());
            }
            unique_tokens.insert(token);
        }
        
        // Count document frequency for each unique token in this document
        for token in unique_tokens {
            *word_doc_count.entry(token).or_insert(0) += 1;
        }
    }
    
    // Calculate document frequencies
    let mut document_frequencies = vec![0.0; vocabulary.len()];
    for (word, &word_id) in &vocabulary {
        document_frequencies[word_id] = *word_doc_count.get(word).unwrap_or(&0) as f64;
    }
    
    // Build TF-IDF vectors
    let mut tfidf_vectors = Vec::new();
    
    for doc in &documents {
        let tokens = tokenize(doc);
        let tf = calculate_tf(&tokens, &vocabulary);
        
        let mut tfidf_vector = HashMap::new();
        for (&word_id, &tf_val) in &tf {
            let df = document_frequencies[word_id];
            let idf = (total_docs as f64 / (df + 1.0)).ln(); // +1 for smoothing
            tfidf_vector.insert(word_id, tf_val * idf);
        }
        
        tfidf_vectors.push(tfidf_vector);
    }
    
    Ok(TfIdfIndex {
        vocabulary,
        document_frequencies,
        tfidf_vectors,
        entity_names: names,
        total_docs,
    })
}

// Core fuzzy matching logic using TF-IDF + string similarity
pub fn find_fuzzy_match_in_index(input_name: &str, index: &TfIdfIndex) -> Result<FuzzyMatchResult> {
    // Check for exact match first
    if index.entity_names.contains(&input_name.to_string()) {
        return Ok(FuzzyMatchResult {
            exact_match: true,
            suggested_name: Some(input_name.to_string()),
            score: Some(1.0),
        });
    }
    
    // Use both string similarity and TF-IDF for better matching
    let mut matches = Vec::new();
    
    for name in &index.entity_names {
        // Calculate string similarity (for substring matching)
        let string_score = calculate_string_similarity(input_name, name);
        
        // Calculate TF-IDF similarity
        let tfidf_score = if index.vocabulary.is_empty() {
            0.0
        } else {
            let input_tokens = tokenize(input_name);
            let input_tf = calculate_tf(&input_tokens, &index.vocabulary);
            
            let mut input_tfidf = HashMap::new();
            for (&word_id, &tf_val) in &input_tf {
                let df = index.document_frequencies[word_id];
                let idf = (index.total_docs as f64 / (df + 1.0)).ln();
                input_tfidf.insert(word_id, tf_val * idf);
            }
            
            // Find the corresponding TF-IDF vector for this name
            let name_index = index.entity_names.iter().position(|n| n == name).unwrap();
            let doc_vector = &index.tfidf_vectors[name_index];
            cosine_similarity(&input_tfidf, doc_vector)
        };
        
        // Combine both scores (weighted average)
        let combined_score = (string_score * 0.7) + (tfidf_score * 0.3);
        
        if combined_score >= FUZZY_MATCH_THRESHOLD {
            matches.push((name.clone(), combined_score));
        }
    }
    
    // Sort by similarity score (descending)
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let suggested_name = matches.first().map(|(name, _)| name.clone());
    let score = matches.first().map(|(_, score)| *score);
    
    Ok(FuzzyMatchResult {
        exact_match: false,
        suggested_name,
        score,
    })
}

// Simple string similarity function for substring matching
fn calculate_string_similarity(input: &str, target: &str) -> f64 {
    let input_lower = input.to_lowercase();
    let target_lower = target.to_lowercase();
    
    // Check if input is a substring of target
    if target_lower.contains(&input_lower) {
        return 0.8; // High score for substring matches
    }
    
    // Check if target is a substring of input
    if input_lower.contains(&target_lower) {
        return 0.6; // Lower score for reverse substring
    }
    
    // Calculate simple character-based similarity
    let input_chars: Vec<char> = input_lower.chars().collect();
    let target_chars: Vec<char> = target_lower.chars().collect();
    
    let mut common_chars = 0;
    let mut i = 0;
    let mut j = 0;
    
    while i < input_chars.len() && j < target_chars.len() {
        if input_chars[i] == target_chars[j] {
            common_chars += 1;
            i += 1;
            j += 1;
        } else if i < input_chars.len() - 1 && input_chars[i + 1] == target_chars[j] {
            i += 1;
        } else if j < target_chars.len() - 1 && input_chars[i] == target_chars[j + 1] {
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
    
    let max_len = input_chars.len().max(target_chars.len());
    if max_len == 0 {
        return 0.0;
    }
    
    (common_chars as f64) / (max_len as f64)
}

impl TfIdfIndex {
    /// Add a single new entity to the existing index
    pub fn add_entity(&mut self, entity_name: String) -> Result<()> {
        // Check if entity already exists
        if self.entity_names.contains(&entity_name) {
            return Ok(());
        }
        
        let tokens = tokenize(&entity_name);
        let mut new_words = Vec::new();
        
        // Add new words to vocabulary
        for token in &tokens {
            if !self.vocabulary.contains_key(token) {
                let word_id = self.vocabulary.len();
                self.vocabulary.insert(token.clone(), word_id);
                self.document_frequencies.push(0.0);
                new_words.push(word_id);
            }
        }
        
        // Update document frequencies for words in this document
        let mut unique_tokens = std::collections::HashSet::new();
        for token in &tokens {
            if let Some(&word_id) = self.vocabulary.get(token) {
                unique_tokens.insert(word_id);
            }
        }
        
        for &word_id in &unique_tokens {
            self.document_frequencies[word_id] += 1.0;
        }
        
        // Calculate TF-IDF vector for new document
        let tf = calculate_tf(&tokens, &self.vocabulary);
        let mut tfidf_vector = HashMap::new();
        
        self.total_docs += 1;
        
        for (&word_id, &tf_val) in &tf {
            let df = self.document_frequencies[word_id];
            let idf = (self.total_docs as f64 / (df + 1.0)).ln();
            tfidf_vector.insert(word_id, tf_val * idf);
        }
        
        // Add to index
        self.tfidf_vectors.push(tfidf_vector);
        self.entity_names.push(entity_name);
        
        // Only recalculate IDF for affected documents (containing new words)
        if !new_words.is_empty() {
            self.recalculate_idf_for_new_words(&new_words)?;
        }
        
        Ok(())
    }
    
    /// Remove an entity from the index
    pub fn remove_entity(&mut self, entity_name: &str) -> Result<bool> {
        if let Some(index) = self.entity_names.iter().position(|name| name == entity_name) {
            // Update document frequencies
            let tokens = tokenize(entity_name);
            let mut unique_tokens = std::collections::HashSet::new();
            for token in &tokens {
                if let Some(&word_id) = self.vocabulary.get(token) {
                    unique_tokens.insert(word_id);
                }
            }
            
            for &word_id in &unique_tokens {
                self.document_frequencies[word_id] -= 1.0;
            }
            
            // Remove from collections
            self.entity_names.remove(index);
            self.tfidf_vectors.remove(index);
            self.total_docs -= 1;
            
            // Recalculate IDF for all remaining documents (since total_docs changed)
            self.recalculate_all_idf()?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn recalculate_idf_for_new_words(&mut self, new_word_ids: &[usize]) -> Result<()> {
        for tfidf_vector in &mut self.tfidf_vectors {
            for &word_id in new_word_ids {
                if let Some(tf_val) = tfidf_vector.get(&word_id) {
                    let df = self.document_frequencies[word_id];
                    let new_idf = (self.total_docs as f64 / (df + 1.0)).ln();
                    tfidf_vector.insert(word_id, tf_val * new_idf);
                }
            }
        }
        Ok(())
    }
    
    fn recalculate_all_idf(&mut self) -> Result<()> {
        for (doc_idx, tfidf_vector) in self.tfidf_vectors.iter_mut().enumerate() {
            let entity_name = &self.entity_names[doc_idx];
            let tokens = tokenize(entity_name);
            let tf = calculate_tf(&tokens, &self.vocabulary);
            
            tfidf_vector.clear();
            for (&word_id, &tf_val) in &tf {
                let df = self.document_frequencies[word_id];
                let idf = (self.total_docs as f64 / (df + 1.0)).ln();
                tfidf_vector.insert(word_id, tf_val * idf);
            }
        }
        Ok(())
    }
}