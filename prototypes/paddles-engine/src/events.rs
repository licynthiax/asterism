use asterism::Logic;
use macroquad::math::Vec2;

#[derive(Clone, Copy)]
pub enum EngineCtrlEvents {
    MovePaddle(usize, crate::types::ActionID),
    ServePressed(usize, crate::types::ActionID),
}

#[derive(Clone, Copy)]
pub enum EngineCollisionEvents {
    BallPaddleCollide { ball: usize, paddle: usize },
    BallWallCollide { ball: usize, wall: usize },
    BallScoreWallCollide { ball: usize, score: usize },
}

#[derive(Clone)]
pub enum EngineActions {
    BounceBall(usize, usize), // idx of paddle, then ball
    ServeBall(usize, usize),  // idx of paddle, then ball
    MovePaddle(usize, Vec2),
    MoveBall(usize, Vec2), // ball idx
    RemoveEntity(usize),
    AddEntity(crate::Ent),
    ChangeScore(usize, u16), // score idx, val
}

impl EngineActions {
    pub(crate) fn perform_action(&self, state: &mut crate::State, logics: &mut crate::Logics) {
        match self {
            Self::BounceBall(i, j) => {
                let id = state.get_id(*i);
                if let crate::EntID::Ball(ball_id) = id {
                    let sides_touched = logics.collision.sides_touched(*i, *j);
                    let mut vals = logics.physics.get_ident_data(ball_id.idx());
                    if sides_touched.y != 0.0 {
                        vals.vel.y *= -1.0;
                    }
                    if sides_touched.x != 0.0 {
                        vals.vel.x *= -1.0;
                    }
                    logics.physics.update_ident_data(ball_id.idx(), vals);
                }
            }
            Self::ChangeScore(score, val) => {
                logics.resources.handle_predicate(&(
                    // i think this logic is wrong, but that's the fault of past me when they
                    // decided to make this stuff so *convoluted*
                    crate::RsrcPool::Score(state.scores[*score]),
                    asterism::resources::Transaction::Change(*val),
                ));
            }
            _ => {}
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct Events {
    pub(crate) control: Vec<(EngineCtrlEvents, EngineActions)>,
    pub(crate) collision: Vec<(EngineCollisionEvents, EngineActions)>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
        }
    }
}

/* impl Events {
    pub fn new() -> Self {
        Self {
            control: None,
            physics: None,
            collision: None,
            resources: None,
        }
    }
}

#[macro_export]
/// paddles_engine rules!
///
/// These rules are defined in a way that requires knowledge of how existence-based processing works, which is... not super ideal maybe??, since the concept is sort of difficult to get your head around. But it works?????? which is cool????
///
/// You define each rule in a stage: control, physics, collision, and resources, in the order they're executed. Each rule declares whether it zips two output tables or filters them, corresponding to the actions currently available in [ConditionTables][asterism::tables::ConditionTables] and [Compose][asterism::tables::Compose].
///
/// Finally, optionally, the user can define if they want a predicate to run on each value of the resulting output table ("foreach"), if at least one is true ("ifany"), or only on the first value of the output table ("forfirst"). (MORE OPTIONS HERE?)
///
/// Note that it's impossible to add events while the game is running. This is a restriction of how the macro works (with closures rather than from a data structure or JSON file or something).
///
/// @setup rules are used during initialization to add a row of query outputs to the table. @run rules are used in the game loop to update those table rows. @then rules execute a piece of code according to the table output (as described above).
macro_rules! rules {
    (@setup filter $id:expr, $filter:expr => $filter_type:ty, |$_filter_pat:pat, $logic:pat, $state:pat| $_predicate:block $(, $($_then:tt)*)?) => {
        |game: &mut $crate::Game| {
            game.tables.add_query::<$filter_type>(
                $id,
                Some($crate::Compose::Filter($filter)),
            );
        }
    };
    (@setup zip $id:expr, ($zip1:expr => $zip_ty1:ty, $zip2:expr => $zip_ty2:ty) $(, $($_then:tt)*)?) => {
        |game: &mut $crate::Game| {
            game.tables.add_query::<($zip_ty1, $zip_ty2)>(
                $id,
                Some($crate::Compose::Zip(
                    $zip1,
                    $zip2,
                )),
            );
        }
    };

    (@run filter $id:expr, $filter:expr => $filter_type:ty, |$filter_pat:pat, $state:pat, $logics:pat| $predicate:block) => {
        |game: &mut Game| {
            let $logics = &game.logics;
            let $state = &game.state;
            game
                .tables
                .update_filter($id, |$filter_pat: &$filter_type| $predicate)
                .unwrap();
            }
    };
    (@run filter $id:expr, $filter:expr => $filter_type:ty, |$filter_pat:pat, $state:pat, $logics:pat| $predicate:block, $($then:tt)*) => {
        |game: &mut Game| {
            let ans = {
                let $state = &game.state;
                let $logics = &game.logics;
                game
                .tables
                .update_filter($id, |$filter_pat: &$filter_type| $predicate)
                .unwrap()
            };
            $crate::rules!(@then [ans] [game] [&$filter_type], $($then)*);
        }
    };

    (@run zip $id:expr, ($zip1:expr => $zip_ty1:ty, $zip2:expr => $zip_ty2:ty)) => {
        |game: &mut Game| {
        game.tables
            .update_zip::<$zip_ty1, $zip_ty2>($id)
            .unwrap();
        }
    };

    (@run zip $id:expr, ($_zip1:expr => $zip_ty1:ty, $_zip2:expr => $zip_ty2:ty), $($then:tt)*) => {
        |game: &mut Game| {
            let ans = game.tables
                .update_zip::<$zip_ty1, $zip_ty2>($id)
                .unwrap();
            $crate::rules!(@then [ans] [game] [&($zip_ty1, $zip_ty2)], $($then)*);
        }
    };

    (@then [$answers:ident] [$game:ident] [$ev_type:ty], foreach |$event:pat, $state:pat, $logics:pat| $predicate:block) => {
        let predicate = |$event: $ev_type, $state: &mut $crate::State, $logics: &mut $crate::Logics| $predicate;
        for ans in $answers.iter() {
            predicate(ans, &mut $game.state, &mut $game.logics);
        }
    };

    (@then [$answers:ident] [$game:ident] [$_ev_type:ty], ifany |$state:pat, $logics:pat| $predicate:block) => {
        let $state = &game.state;
        let $logics = &game.logics;
        if !$answers.is_empty() {
            $predicate
        }
    };

    (@then [$answers:ident] [$game:ident] [$ev_type:ty], forfirst |$event:pat, $state:pat, $logics:pat| $predicate:block) => {
        let predicate = |$event: $ev_type, $state: &mut $crate::State, $logics: &mut $crate::Logics| $predicate;
        if let Some(ans) = $answers.iter().next() {
            predicate(ans, &mut $game.state, &mut $game.logics);
        }
    };

    ($game:ident =>
        control: [ $({$($ctrl_rule:tt)+}),* ]
        physics: [ $({$($phys_rule:tt)+}),* ]
        collision: [ $({$($col_rule:tt)+}),* ]
        resources: [ $({$($rsrc_rule:tt)+}),* ]
    ) => {
        {
            use std::rc::Rc;
            trait PaddlesUserEvents {
                fn setup(&mut self, setup: Rc<dyn Fn(&mut Self)>);
                fn control(&mut self, control: Rc<dyn Fn(&mut Self)>);
                fn collision(&mut self, collision: Rc<dyn Fn(&mut Self)>);
                fn resources(&mut self, resources: Rc<dyn Fn(&mut Self)>);
                fn physics(&mut self, physics: Rc<dyn Fn(&mut Self)>);
            }

            impl PaddlesUserEvents for $crate::Game {
                fn setup(&mut self, setup: Rc<dyn Fn(&mut $crate::Game)>) {
                    setup(self);
                }
                fn control(&mut self, control: Rc<dyn Fn(&mut Self)>) {
                    self.events.control = Some(control);
                }
                fn collision(&mut self, collision: Rc<dyn Fn(&mut Self)>) {
                    self.events.collision = Some(collision);
                }
                fn resources(&mut self, resources: Rc<dyn Fn(&mut Self)>) {
                    self.events.resources = Some(resources);
            }
                fn physics(&mut self, physics: Rc<dyn Fn(&mut Self)>) {
                    self.events.physics = Some(physics);
                }
            }

            let setup = Rc::new(move |game: &mut $crate::Game| {
                $($crate::rules!(@setup $($ctrl_rule)+)(game);)*
                $($crate::rules!(@setup $($phys_rule)+)(game);)*
                $($crate::rules!(@setup $($col_rule)+)(game);)*
                $($crate::rules!(@setup $($rsrc_rule)+)(game);)*
            });
            $game.setup(setup);

            let control = Rc::new(move |game: &mut $crate::Game| {
                $($crate::rules!(@run $($ctrl_rule)+)(game);)*
            });
            let physics = Rc::new(move |game: &mut $crate::Game| {
                $($crate::rules!(@run $($phys_rule)+)(game);)*
            });
            let collision = Rc::new(move |game: &mut $crate::Game| {
                $($crate::rules!(@run $($col_rule)+)(game);)*
            });
            let resources = Rc::new(move |game: &mut $crate::Game| {
                $($crate::rules!(@run $($rsrc_rule)+)(game);)*
            });

            $game.control(control);
            $game.physics(physics);
            $game.collision(collision);
            $game.resources(resources);
        }
    };
} */
