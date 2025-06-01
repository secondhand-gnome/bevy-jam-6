use avian2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Enemy,  // Layer 1
    Plant,  // Layer 2
}
