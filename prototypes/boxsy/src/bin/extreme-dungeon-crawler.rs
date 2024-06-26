use boxsy::*;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::rand::srand(get_time().to_bits());
    let mut game = Game::new();
    init(&mut game);
    run(game).await;
}

fn init(game: &mut Game) {
    game.set_background(BLACK);

    let mut player = Player::new();
    player.pos = IVec2::new(3, 3);
    player.color = PURPLE;

    let rocks = game.log_rsrc("rock".to_string());

    player.add_inventory_item(rocks.clone(), 0);

    game.set_player(player);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    game.log_tile_info(Tile::new());

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    #[rustfmt::skip]
    let maps = [
r#"00000000
0      0
0   2  0
0      0
0      0
0      0
0    1 0
00000000"#,

r#"00000000
0      0
0      0
0   1  0
0      0
0 2    0
0      0
00000000"#,

r#"00000000
0      0
0      0
0      0
0      0
0  1   0
0      0
00000000"#
    ];

    for map in maps.iter() {
        game.add_room_from_str(map).unwrap();
    }

    let mut character = Character::new();
    character.add_inventory_item(rocks.clone(), 2);
    character.pos = IVec2::new(1, 2);
    character.color = BROWN;
    let char_id = game.add_character(character, 0);

    game.add_collision_predicate(
        (0, CollisionEnt::Player, CollisionEnt::Character(char_id)),
        EngineAction::ChangeResource(
            PoolID::new(EntID::Character(char_id), rocks.clone()),
            Transaction::Trade(1, PoolID::new(EntID::Player, rocks.clone())),
        ),
    );
    game.add_link((0, CollisionEnt::Character(char_id)), (2, IVec2::new(3, 2)));

    let mut character = Character::new();
    character.pos = IVec2::new(3, 3);
    character.color = BROWN;
    let char_id = game.add_character(character, 1);
    game.add_link((1, CollisionEnt::Character(char_id)), (2, IVec2::new(3, 2)));

    game.add_link(
        (0, CollisionEnt::Tile(IVec2::new(5, 6))),
        (1, IVec2::new(3, 1)),
    );
    game.add_link(
        (1, CollisionEnt::Tile(IVec2::new(4, 3))),
        (0, IVec2::new(3, 1)),
    );
    game.add_link(
        (2, CollisionEnt::Tile(IVec2::new(3, 5))),
        (1, IVec2::new(3, 1)),
    );
}
