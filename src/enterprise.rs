use std::collections::HashMap;

#[derive(Debug)]
pub struct Enterprises {
    enterprises: HashMap<String, Enterprise>,
}

impl Enterprises {
    pub fn new() -> Self {
        let mut enterprises = HashMap::new();
        let id = "enterprise_id";
        enterprises.insert(String::from(id), Enterprise::new(id));
        Enterprises { enterprises }
    }

    pub fn find(&self, id: &str) -> Option<&Enterprise> {
        let id = String::from(id);
        self.enterprises.get(&id)
    }
}

#[derive(Debug)]
pub struct Enterprise {
    pub id: String,
    pub origin: String,
}

impl Enterprise {
    pub fn new(id: &str) -> Self {
        Enterprise {
            id: String::from(id),
            origin: String::from("http://localhost:8081"),
        }
    }
}
