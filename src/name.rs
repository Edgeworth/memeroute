use std::collections::HashMap;

// Integer IDs representing names. Readable names used in PCB.
pub type Id = usize;
pub const NO_ID: Id = Id::MAX;

#[derive(Debug, Default, Clone)]
pub struct NameMap {
    name_to_id: HashMap<String, Id>, // Name to ID.
    id_to_name: HashMap<Id, String>, // ID to name.
    next_id: Id,
}

impl NameMap {
    pub fn name(&self, id: Id) -> &str {
        self.id_to_name.get(&id).unwrap()
    }

    pub fn name_to_id(&self, name: &str) -> Id {
        *self.name_to_id.get(name).unwrap()
    }

    pub fn ensure_name(&mut self, name: &str) -> Id {
        if let Some(id) = self.name_to_id.get(name) { *id } else { self.add_name(name) }
    }

    pub fn new_id(&mut self) -> Id {
        // TODO: may overwrite regular name.
        self.add_name("anon")
    }

    fn add_name(&mut self, name: &str) -> Id {
        let id = self.next_id;
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(id, name.to_string());
        self.next_id += 1;
        id
    }
}
