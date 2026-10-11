//! Dynamic parameter schema and runtime value management for video plugins.

use std::collections::HashMap;

/// Parameter data types supported by video editor plugins.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Choice { index: usize, options: Vec<String> },
}

/// Metadata describing a plugin parameter to the host UI.
#[derive(Debug, Clone)]
pub struct ParameterDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_value: ParameterValue,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

/// Collection of parameter values configured for a plugin instance.
#[derive(Debug, Clone)]
pub struct PluginParameters {
    values: HashMap<&'static str, ParameterValue>,
}

impl PluginParameters {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: &'static str, value: ParameterValue) {
        self.values.insert(id, value);
    }

    pub fn get_float(&self, id: &'static str, default: f32) -> f32 {
        match self.values.get(id) {
            Some(ParameterValue::Float(v)) => *v,
            _ => default,
        }
    }

    pub fn get_int(&self, id: &'static str, default: i32) -> i32 {
        match self.values.get(id) {
            Some(ParameterValue::Int(v)) => *v,
            _ => default,
        }
    }

    pub fn get_bool(&self, id: &'static str, default: bool) -> bool {
        match self.values.get(id) {
            Some(ParameterValue::Bool(v)) => *v,
            _ => default,
        }
    }

    pub fn get_choice_index(&self, id: &'static str, default: usize) -> usize {
        match self.values.get(id) {
            Some(ParameterValue::Choice { index, .. }) => *index,
            _ => default,
        }
    }
}
