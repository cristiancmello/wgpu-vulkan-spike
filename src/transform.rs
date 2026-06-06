use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub tx: f32,
    pub ty: f32,
    pub sx: f32,
    pub sy: f32,
    pub angle: f32,
}

impl Transform {
    pub fn new(tx: f32, ty: f32, sx: f32, sy: f32, angle: f32) -> Self {
        Transform { tx, ty, sx, sy, angle }
    }

    pub fn identity() -> Self {
        Transform {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            angle: 0.0,
        }
    }

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();

        [
            [self.sx * cos_a, self.sx * sin_a, 0.0, 0.0],
            [-self.sy * sin_a, self.sy * cos_a, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [self.tx, self.ty, 0.0, 1.0],
        ]
    }
}

pub struct TransformBuffer {
    transforms: HashMap<u32, Transform>,
}

impl TransformBuffer {
    pub fn new() -> Self {
        TransformBuffer {
            transforms: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: u32, transform: Transform) {
        self.transforms.insert(id, transform);
    }

    pub fn get(&self, id: u32) -> Option<Transform> {
        self.transforms.get(&id).copied()
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

impl Default for TransformBuffer {
    fn default() -> Self {
        Self::new()
    }
}
