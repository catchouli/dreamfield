use bevy_ecs::prelude::Component;

/// A component for entities that can take damage
#[derive(Component)]
pub struct Health {
    pub health: f32,
}

impl Health {
    /// Create a new Health with the given amount
    pub fn new(health: f32) -> Self {
        Health { health }
    }

    /// Apply damage, returning whether the entity died
    pub fn damage(&mut self, amount: f32) -> bool {
        self.health -= amount;
        self.health <= 0.0
    }
}
