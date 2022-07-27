mod console;

use std::mem::MaybeUninit;

use console::*;

const GRAVITY: f32 = 0.25;
const AMOUNT: usize = 5;

struct GameState {
    positions: Vec<(f32, f32)>,
    velocities: Vec<(f32, f32)>,
    graphics_params: Vec<i32>,
    item_width: i32,
    item_height: i32,
    palette_tracker: i32,
    window_height: i32,
    window_width: i32,
    palette_count: i32,
    rng: MaybeUninit<fastrand::Rng>,
}

static mut GAMESTATE: GameState = GameState {
    positions: Vec::new(),
    velocities: Vec::new(),
    graphics_params: Vec::new(),
    item_width: 0,
    item_height: 0,
    palette_tracker: 0,
    window_height: 0,
    window_width: 0,
    palette_count: 0,
    rng: MaybeUninit::uninit(),
};

#[no_mangle]
pub unsafe extern "C" fn init() {
    // Set up our intial values
    GAMESTATE.item_height = sprite_height(0);
    GAMESTATE.item_width = sprite_width(0);
    GAMESTATE.window_height = height();
    GAMESTATE.window_width = width();
    GAMESTATE.palette_count = palette_count();
    GAMESTATE.rng.write(fastrand::Rng::with_seed(0xBEEF));
    add_items(AMOUNT);
}

#[no_mangle]
pub unsafe extern "C" fn update() {
    // Spawn more items if user presses A
    if button_a_held(0) != 0 {
        add_items(AMOUNT);
    }

    // Prepare some local working variables.
    let positions = GAMESTATE.positions.iter_mut();
    let velocities = GAMESTATE.velocities.iter_mut();
    let width = GAMESTATE.item_width;
    let height = GAMESTATE.item_height;
    let rng = GAMESTATE.rng.assume_init_mut();

    // Iterate through each pair of positions & velocities
    positions.zip(velocities).for_each(|(position, velocity)| {
        // Handle left, right edge collisions
        let left_collision = (position.0 as i32) < 0 && velocity.0 < 0.0;
        let right_collision =
            (position.0 as i32 + width > GAMESTATE.window_width) && velocity.0 > 0.0;

        if left_collision || right_collision {
            velocity.0 = -velocity.0;
        }

        // Handle bottom floor collisions
        if position.1 as i32 + height > GAMESTATE.window_height {
            velocity.1 = -((rng.f32() * 5.0) + 5.0);
        }

        // Add gravity
        velocity.1 += GRAVITY;

        // Move positions based on velocity
        position.0 += velocity.0;
        position.1 += velocity.1;
    })
}

#[no_mangle]
pub unsafe extern "C" fn draw() {
    clear_screen(0);

    // Prepare iterators
    let positions = GAMESTATE.positions.iter();
    let params = GAMESTATE.graphics_params.iter();

    // Render each sprite
    positions
        .zip(params)
        .for_each(|(position, param)| sprite(*param, 0, position.0 as i32, position.1 as i32));
}

// Adds N items to the game state
unsafe fn add_items(count: usize) {
    let rng = GAMESTATE.rng.assume_init_mut();

    (0..count).for_each(|_| {
        // Prepare the components
        let velocity = (rng.f32() * 10.0 - (5.0), 0.0f32);
        let position = (GAMESTATE.window_width as f32 / 2.0, 0.0);
        let palette = GAMESTATE.palette_tracker % GAMESTATE.palette_count;
        let params = graphics_parameters(palette, 0, 0, 0, 0, 0);

        // Push them into the vec
        GAMESTATE.velocities.push(velocity);
        GAMESTATE.positions.push(position);
        GAMESTATE.graphics_params.push(params);

        // Increment the palette counter
        GAMESTATE.palette_tracker += 1;
    });

    // Prevent the palette counter from overflowing
    GAMESTATE.palette_tracker %= GAMESTATE.palette_count;
}
