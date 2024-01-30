use bitsy::*;
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

    let player_rocks = game.log_rsrc("rocks");
    player.add_inventory_item(player_rocks, 0);

    game.set_player(player);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    let mut tile = Tile::new();
    tile.solid = true;
    game.log_tile_info(tile);

    game.log_tile_info(Tile::new());

    #[rustfmt::skip]
    let maps = [r#"
00000000
0      0
0   2  0
0      0
0      0
0  3   0
0      0
00000000
    "#,

r#"
00000000
0      0
0      0
0   1  0
0      0
0      0
0      0
00000000
    "#
    ];

    for map in maps.iter() {
        game.add_room_from_str(map).unwrap();
    }

    let mut character = Character::new();

    let char_rocks = game.log_rsrc("rocks");
    character.add_inventory_item(char_rocks, 2);

    character.pos = IVec2::new(1, 2);
    character.color = BROWN;
    let char_id = game.add_character(character, 0);

    // oh wait this only does the transaction change for the first thing =_=
    game.add_collision_predicate(
        (0, Contact::Ent(0, char_id.idx() + 1)),
        EngineAction::ChangeResource(
            PoolID::new(EntID::Player, player_rocks),
            Transaction::Change(1),
        ),
    );
    game.add_collision_predicate(
        (0, Contact::Ent(0, char_id.idx() + 1)),
        EngineAction::ChangeResource(
            PoolID::new(EntID::Character(char_id), char_rocks),
            Transaction::Change(1),
        ),
    );

    game.add_link((0, IVec2::new(3, 5)), (1, IVec2::new(1, 1)));
}
