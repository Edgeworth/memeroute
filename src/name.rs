use std::collections::HashMap;

// Integer IDs representing names. Readable names used in PCB.
pub type Id = usize;
pub const NO_ID: Id = Id::MAX;

#[derive(Debug, Default, Clone)]
pub struct NameMap {
    names: Vec<String>,              // ID to name.
    name_to_id: HashMap<String, Id>, // Name to  ID.
}

impl NameMap {
    pub fn name(&self, id: Id) -> &str {
        &self.names[id]
    }

    pub fn name_to_id(&self, name: &str) -> Id {
        *self.name_to_id.get(name).unwrap()
    }

    pub fn ensure_name(&mut self, name: &str) -> Id {
        if let Some(id) = self.name_to_id.get(name) {
            *id
        } else {
            let id = self.names.len();
            self.name_to_id.insert(name.to_string(), id);
            self.names.push(name.to_string());
            id
        }
    }
}
