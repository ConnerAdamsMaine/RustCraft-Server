/// Represents a Minecraft world/dimension
pub struct World {
    name: String,
}

impl World {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn default() -> Self {
        Self {
            name: "world".to_string(),
        }
    }
}
