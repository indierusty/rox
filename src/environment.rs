use std::collections::HashMap;

use crate::value::Value;

pub struct Environment {
    // environment for each block scope
    environments: Vec<HashMap<String, Option<Value>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            // predefined scope for Global Scope
            // TODO: make this empty vec by parsing src global scope to block itself.
            environments: vec![HashMap::new()],
        }
    }

    pub fn begin_scope(&mut self) {
        self.environments.push(HashMap::new())
    }

    pub fn end_scope(&mut self) {
        self.environments.pop();
    }

    pub fn define_var(&mut self, name: String, value: Option<Value>) {
        let env = self.environments.last_mut().expect("No scope yet.");
        env.insert(name, value);
    }

    pub fn get_var(&mut self, name: String) -> Result<Value, String> {
        for i in (0..self.environments.len()).rev() {
            if self.environments[i].contains_key(&name) {
                let value = &self.environments[i][&name];
                if let Some(value) = value {
                    return Ok(value.clone());
                } else {
                    return Err("Variable is not initialized.".to_string());
                }
            }
        }

        Err("Variable is undefined.".to_string())
    }

    pub fn assign_var(&mut self, name: String, value: Value) -> Result<Value, String> {
        for i in (0..self.environments.len()).rev() {
            if self.environments[i].contains_key(&name) {
                self.environments[i].insert(name.clone(), Some(value));
                break;
            }
        }

        Err(format!("Undefined variable '{}'", name))
    }
}
