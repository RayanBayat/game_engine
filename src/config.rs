//Player
pub const PLAYER_TOP_SPEED: f32 = 600.0;
pub const PLAYER_SPEED: f32 = 800.0;
pub const PLAYER_ACCELERATION: f32 = 1.0;
pub const PLAYER_JUMP_STRENGTH: f32 = 500.0;
pub const PLAYER_FRICTION: f32 = 0.85;
pub const PLAYER_STARTING_POSITION: [f32; 2] = [5.0, 5.0];
pub const PLAYER_SIZE: [f32; 2] = [50.0, 50.0];
pub const PLAYER_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
pub const PLAYER_ROTATION: f32 = 0.00;
pub const PLAYER_ANIMATION_SPEED: f32 = 0.2;

//camera
pub const CAMERA_STARTING_POSITION: [f32; 2] = [0.0, 0.0];

//wall
pub const WALL_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const WALL_ROTATION: f32 = 0.0;

//world
pub const WORLD_DIMENSIONS: [i32; 2] = [10, 10];
pub const GRAVITY: f32 = 1200.0;

pub type UniformPadding = [f32; 8];
pub const UNIFORM_PADDING: [f32; 8] = [0.0; 8];
