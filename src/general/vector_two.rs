#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    pub x: f32, 
    pub y: f32,
}


impl Vector2 {
    pub fn new() -> Self {
        Vector2 {
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn subtract(&mut self, other: &Vector2){
        self.x -= other.x;
        self.y -= other.y;
    }

    pub fn subtract_to_new(&self, other: &Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn add(&mut self, other: &Vector2){
        self.x += other.x;
        self.y += other.y;
    }

    pub fn add_to_new(&self, other: &Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn multiply(&mut self, multiplier: f32){
        self.x *= multiplier;
        self.y *= multiplier;
    }

    pub fn multiply_to_new(&self, multiplier: f32) -> Vector2 {
        Vector2 {
            x: self.x * multiplier,
            y: self.y * multiplier,
        }
    }

    pub fn get_sqr_magnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn get_magnitude(&self) -> f32 {
        self.get_sqr_magnitude().sqrt()
    }

    pub fn normalise(&mut self){
        let length = self.get_magnitude();

        self.x /= length;
        self.y /= length;
    }

    pub fn dot(vector_one: Vector2, vector_two: Vector2) -> f32{
        vector_one.x * vector_two.x + vector_one.y * vector_two.y
    }
}