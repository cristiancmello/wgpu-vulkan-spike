use crate::renderer::Vertex;

#[derive(Clone, Debug)]
pub struct Primitive {
    pub id: u32,
    pub vertices: Vec<Vertex>,
}

impl Primitive {
    pub fn new(id: u32, vertices: Vec<Vertex>) -> Self {
        Primitive { id, vertices }
    }
}

pub struct PrimitiveBuffer {
    primitives: Vec<Primitive>,
}

impl PrimitiveBuffer {
    pub fn new() -> Self {
        PrimitiveBuffer {
            primitives: Vec::new(),
        }
    }

    pub fn add(&mut self, primitive: Primitive) {
        if let Some(pos) = self.primitives.iter().position(|p| p.id == primitive.id) {
            self.primitives[pos] = primitive;
        } else {
            self.primitives.push(primitive);
        }
    }

    pub fn get_all(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }
}

impl Default for PrimitiveBuffer {
    fn default() -> Self {
        Self::new()
    }
}
