use crate::text::{get_token, is_equal, value};

mod check;
mod parser;
pub use check::{check_orders, OrderCheckResult};
pub use parser::{
    parse_order_file, OrderParseError, OrderParseMode, OrderParseResult, ParsedOrderLine,
};

/// C++: `orders.h` order enum and `orders.cpp` `od` keyword table.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum OrderVerb {
    Atlantis = 0,
    EndDirective,
    Unit,
    Address,
    Advance,
    Armor,
    Assassinate,
    Attack,
    Autotax,
    Avoid,
    Behind,
    Build,
    Buy,
    Cast,
    Claim,
    Combat,
    Consume,
    Declare,
    Describe,
    Destroy,
    Distribute,
    EndForm,
    EndTurn,
    Enter,
    Entertain,
    Evict,
    Exchange,
    Faction,
    Find,
    Forget,
    Form,
    Give,
    Guard,
    Hold,
    Idle,
    Join,
    Leave,
    Move,
    Name,
    NoAid,
    NoCross,
    NoSpoils,
    Option,
    Password,
    Pillage,
    Prepare,
    Produce,
    Promote,
    Quit,
    Restart,
    Reveal,
    Sail,
    Sell,
    Share,
    Show,
    Spoils,
    Steal,
    Study,
    Take,
    Tax,
    Teach,
    Transport,
    Turn,
    Weapon,
    Withdraw,
    Work,
    #[default]
    Count,
}

/// C++: `orders.cpp` `od`, in the index order of `orders.h`.
pub const ORDER_VERB_STRINGS: [&str; OrderVerb::Count as usize] = [
    "#atlantis",
    "#end",
    "unit",
    "address",
    "advance",
    "armor",
    "assassinate",
    "attack",
    "autotax",
    "avoid",
    "behind",
    "build",
    "buy",
    "cast",
    "claim",
    "combat",
    "consume",
    "declare",
    "describe",
    "destroy",
    "distribute",
    "end",
    "endturn",
    "enter",
    "entertain",
    "evict",
    "exchange",
    "faction",
    "find",
    "forget",
    "form",
    "give",
    "guard",
    "hold",
    "idle",
    "join",
    "leave",
    "move",
    "name",
    "noaid",
    "nocross",
    "nospoils",
    "option",
    "password",
    "pillage",
    "prepare",
    "produce",
    "promote",
    "quit",
    "restart",
    "reveal",
    "sail",
    "sell",
    "share",
    "show",
    "spoils",
    "steal",
    "study",
    "take",
    "tax",
    "teach",
    "transport",
    "turn",
    "weapon",
    "withdraw",
    "work",
];

impl OrderVerb {
    fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::Atlantis,
            1 => Self::EndDirective,
            2 => Self::Unit,
            3 => Self::Address,
            4 => Self::Advance,
            5 => Self::Armor,
            6 => Self::Assassinate,
            7 => Self::Attack,
            8 => Self::Autotax,
            9 => Self::Avoid,
            10 => Self::Behind,
            11 => Self::Build,
            12 => Self::Buy,
            13 => Self::Cast,
            14 => Self::Claim,
            15 => Self::Combat,
            16 => Self::Consume,
            17 => Self::Declare,
            18 => Self::Describe,
            19 => Self::Destroy,
            20 => Self::Distribute,
            21 => Self::EndForm,
            22 => Self::EndTurn,
            23 => Self::Enter,
            24 => Self::Entertain,
            25 => Self::Evict,
            26 => Self::Exchange,
            27 => Self::Faction,
            28 => Self::Find,
            29 => Self::Forget,
            30 => Self::Form,
            31 => Self::Give,
            32 => Self::Guard,
            33 => Self::Hold,
            34 => Self::Idle,
            35 => Self::Join,
            36 => Self::Leave,
            37 => Self::Move,
            38 => Self::Name,
            39 => Self::NoAid,
            40 => Self::NoCross,
            41 => Self::NoSpoils,
            42 => Self::Option,
            43 => Self::Password,
            44 => Self::Pillage,
            45 => Self::Prepare,
            46 => Self::Produce,
            47 => Self::Promote,
            48 => Self::Quit,
            49 => Self::Restart,
            50 => Self::Reveal,
            51 => Self::Sail,
            52 => Self::Sell,
            53 => Self::Share,
            54 => Self::Show,
            55 => Self::Spoils,
            56 => Self::Steal,
            57 => Self::Study,
            58 => Self::Take,
            59 => Self::Tax,
            60 => Self::Teach,
            61 => Self::Transport,
            62 => Self::Turn,
            63 => Self::Weapon,
            64 => Self::Withdraw,
            65 => Self::Work,
            _ => return None,
        })
    }
}

/// C++: `orders.cpp` `Parse1Order`; returns no verb when the token is unknown.
pub fn parse_order_verb(token: &str) -> Option<OrderVerb> {
    ORDER_VERB_STRINGS
        .iter()
        .position(|keyword| is_equal(token, keyword))
        .and_then(OrderVerb::from_index)
}

/// C++: `unit.h` `UnitId`; parsed by `parseorders.cpp` `Game::ParseUnit`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OrderUnitId {
    pub unitnum: i32,
    pub alias: i32,
    pub faction: i32,
}

/// C++: `parseorders.cpp` `Game::ParseUnit`.
/// Parse one unit reference, consuming only the tokens that form it.
pub fn parse_unit_id(input: &mut String) -> Option<OrderUnitId> {
    let token = get_token(input)?;
    if is_equal(&token, "0") {
        return Some(OrderUnitId {
            unitnum: -1,
            alias: 0,
            faction: 0,
        });
    }

    if is_equal(&token, "faction") {
        let faction = value(&get_token(input)?);
        if faction == 0 {
            return None;
        }
        if !is_equal(&get_token(input)?, "new") {
            return None;
        }
        let alias = value(&get_token(input)?);
        if alias == 0 {
            return None;
        }
        return Some(OrderUnitId {
            unitnum: 0,
            alias,
            faction,
        });
    }

    if is_equal(&token, "new") {
        let alias = value(&get_token(input)?);
        if alias == 0 {
            return None;
        }
        return Some(OrderUnitId {
            unitnum: 0,
            alias,
            faction: 0,
        });
    }

    let unitnum = value(&token);
    (unitnum != 0).then_some(OrderUnitId {
        unitnum,
        alias: 0,
        faction: 0,
    })
}

/// C++: `orders.h` movement modes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum MoveMode {
    #[default]
    None = 0,
    Walk = 1,
    Ride = 2,
    Fly = 3,
    Swim = 4,
    Sail = 5,
}

pub const MOVE_PAUSE: i32 = 97;
pub const MOVE_IN: i32 = 98;
pub const MOVE_OUT: i32 = 99;
pub const MOVE_ENTER: i32 = 100;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MoveDir {
    pub dir: i32,
}

/// C++: `orders.h` `Order` base fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Order {
    pub kind: OrderVerb,
    pub quiet: i32,
}

impl Order {
    pub const fn new(kind: OrderVerb) -> Self {
        Self { kind, quiet: 0 }
    }
}

macro_rules! define_order {
    ($name:ident, $kind:ident { $($field:ident: $field_type:ty = $default:expr),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            pub order: Order,
            $(pub $field: $field_type,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    order: Order::new(OrderVerb::$kind),
                    $($field: $default,)*
                }
            }
        }
    };
}

define_order!(MoveOrder, Move {
    advancing: i32 = 0,
    dirs: Vec<MoveDir> = Vec::new(),
});
define_order!(
    WithdrawOrder,
    Withdraw {
        item: i32 = 0,
        amount: i32 = 0,
    }
);
define_order!(GiveOrder, Give {
    item: i32 = 0,
    amount: i32 = 0,
    except: i32 = 0,
    unfinished: i32 = 0,
    merge: i32 = 0,
    target: Option<OrderUnitId> = None,
});
impl GiveOrder {
    pub fn take() -> Self {
        Self {
            order: Order::new(OrderVerb::Take),
            ..Self::default()
        }
    }
}
define_order!(
    StudyOrder,
    Study {
        skill: i32 = 0,
        days: i32 = 0,
        level: i32 = 0,
    }
);
define_order!(TeachOrder, Teach {
    targets: Vec<OrderUnitId> = Vec::new(),
});
define_order!(
    ProduceOrder,
    Produce {
        item: i32 = 0,
        skill: i32 = -1,
        productivity: i32 = 0,
        target: i32 = 0,
    }
);
define_order!(
    BuyOrder,
    Buy {
        item: i32 = 0,
        num: i32 = 0,
    }
);
define_order!(
    SellOrder,
    Sell {
        item: i32 = 0,
        num: i32 = 0,
    }
);
define_order!(AttackOrder, Attack {
    targets: Vec<OrderUnitId> = Vec::new(),
});
define_order!(BuildOrder, Build {
    target: Option<OrderUnitId> = None,
    new_building: i32 = -1,
    needtocomplete: i32 = 0,
});
define_order!(SailOrder, Sail {
    dirs: Vec<MoveDir> = Vec::new(),
});
define_order!(FindOrder, Find { find: i32 = 0 });
define_order!(StealOrder, Steal {
    target: Option<OrderUnitId> = None,
    item: i32 = 0,
});
define_order!(AssassinateOrder, Assassinate {
    target: Option<OrderUnitId> = None,
});
define_order!(ForgetOrder, Forget { skill: i32 = 0 });
define_order!(ExchangeOrder, Exchange {
    give_item: i32 = 0,
    give_amount: i32 = 0,
    expect_item: i32 = 0,
    expect_amount: i32 = 0,
    exchange_status: i32 = -1,
    target: Option<OrderUnitId> = None,
});
define_order!(TurnOrder, Turn {
    repeating: i32 = 0,
    turn_orders: Vec<String> = Vec::new(),
});
define_order!(
    CastOrder,
    Cast {
        spell: i32 = 0,
        level: i32 = 0,
    }
);
define_order!(CastMindOrder, Cast {
    spell: i32 = 0,
    level: i32 = 0,
    id: Option<OrderUnitId> = None,
});
define_order!(
    CastRegionOrder,
    Cast {
        spell: i32 = 0,
        level: i32 = 0,
        xloc: i32 = 0,
        yloc: i32 = 0,
        zloc: i32 = 0,
    }
);
define_order!(TeleportOrder, Cast {
    spell: i32 = 0,
    level: i32 = 0,
    xloc: i32 = 0,
    yloc: i32 = 0,
    zloc: i32 = 0,
    gate: i32 = 0,
    units: Vec<OrderUnitId> = Vec::new(),
});
define_order!(
    CastIntOrder,
    Cast {
        spell: i32 = 0,
        level: i32 = 0,
        target: i32 = 0,
    }
);
define_order!(CastUnitsOrder, Cast {
    spell: i32 = 0,
    level: i32 = 0,
    units: Vec<OrderUnitId> = Vec::new(),
});
define_order!(
    CastTransmuteOrder,
    Cast {
        spell: i32 = 0,
        level: i32 = 0,
        item: i32 = 0,
        number: i32 = 0,
    }
);
define_order!(EvictOrder, Evict {
    targets: Vec<OrderUnitId> = Vec::new(),
});
define_order!(IdleOrder, Idle {});
define_order!(TransportOrder, Transport {
    item: i32 = -1,
    amount: i32 = 0,
    except: i32 = 0,
    target: Option<OrderUnitId> = None,
});
impl TransportOrder {
    pub fn distribute() -> Self {
        Self {
            order: Order::new(OrderVerb::Distribute),
            ..Self::default()
        }
    }
}
define_order!(JoinOrder, Join {
    target: Option<OrderUnitId> = None,
    overload: i32 = 0,
    merge: i32 = 0,
});

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum OrderData {
    #[default]
    None,
    Move(MoveOrder),
    Withdraw(WithdrawOrder),
    Give(GiveOrder),
    Study(StudyOrder),
    Teach(TeachOrder),
    Produce(ProduceOrder),
    Buy(BuyOrder),
    Sell(SellOrder),
    Attack(AttackOrder),
    Build(BuildOrder),
    Sail(SailOrder),
    Find(FindOrder),
    Steal(StealOrder),
    Assassinate(AssassinateOrder),
    Forget(ForgetOrder),
    Exchange(ExchangeOrder),
    Turn(TurnOrder),
    Cast(CastOrder),
    CastMind(CastMindOrder),
    CastRegion(CastRegionOrder),
    Teleport(TeleportOrder),
    CastInt(CastIntOrder),
    CastUnits(CastUnitsOrder),
    CastTransmute(CastTransmuteOrder),
    Evict(EvictOrder),
    Idle(IdleOrder),
    Transport(TransportOrder),
    Join(JoinOrder),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CastOrderData {
    Cast(CastOrder),
    Mind(CastMindOrder),
    Region(CastRegionOrder),
    Int(CastIntOrder),
    Units(CastUnitsOrder),
    Transmute(CastTransmuteOrder),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StealOrderData {
    Steal(StealOrder),
    Assassinate(AssassinateOrder),
}

/// C++: `unit.h` per-unit slots; vectors retain `AList` insertion order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnitOrders {
    pub destroy: i32,
    pub enter: i32,
    pub build: i32,
    pub promote: Option<OrderUnitId>,
    pub find_orders: Vec<FindOrder>,
    pub give_orders: Vec<GiveOrder>,
    pub withdraw_orders: Vec<WithdrawOrder>,
    pub bank_orders: Vec<Order>,
    pub buy_orders: Vec<BuyOrder>,
    pub sell_orders: Vec<SellOrder>,
    pub forget_orders: Vec<ForgetOrder>,
    pub cast_orders: Option<CastOrderData>,
    pub teleport_orders: Option<TeleportOrder>,
    pub steal_orders: Option<StealOrderData>,
    pub month_orders: Option<OrderData>,
    pub attack_orders: Option<AttackOrder>,
    pub evict_orders: Option<EvictOrder>,
    pub exchange_orders: Vec<ExchangeOrder>,
    pub turn_orders: Vec<TurnOrder>,
    pub in_turn_block: i32,
    pub present_month_orders: Option<OrderData>,
    pub transport_orders: Vec<TransportOrder>,
    pub join_orders: Option<JoinOrder>,
}

#[cfg(test)]
mod tests {
    use super::{
        parse_order_verb, parse_unit_id, AssassinateOrder, AttackOrder, BuildOrder, BuyOrder,
        CastIntOrder, CastMindOrder, CastOrder, CastRegionOrder, CastTransmuteOrder,
        CastUnitsOrder, EvictOrder, ExchangeOrder, FindOrder, ForgetOrder, GiveOrder, IdleOrder,
        JoinOrder, MoveDir, MoveMode, MoveOrder, Order, OrderData, OrderUnitId, OrderVerb,
        ProduceOrder, SailOrder, SellOrder, StealOrder, StudyOrder, TeachOrder, TeleportOrder,
        TransportOrder, TurnOrder, UnitOrders, WithdrawOrder, MOVE_ENTER, MOVE_IN, MOVE_OUT,
        MOVE_PAUSE,
    };

    const ORDER_KEYWORDS: [&str; 66] = [
        "#atlantis",
        "#end",
        "unit",
        "address",
        "advance",
        "armor",
        "assassinate",
        "attack",
        "autotax",
        "avoid",
        "behind",
        "build",
        "buy",
        "cast",
        "claim",
        "combat",
        "consume",
        "declare",
        "describe",
        "destroy",
        "distribute",
        "end",
        "endturn",
        "enter",
        "entertain",
        "evict",
        "exchange",
        "faction",
        "find",
        "forget",
        "form",
        "give",
        "guard",
        "hold",
        "idle",
        "join",
        "leave",
        "move",
        "name",
        "noaid",
        "nocross",
        "nospoils",
        "option",
        "password",
        "pillage",
        "prepare",
        "produce",
        "promote",
        "quit",
        "restart",
        "reveal",
        "sail",
        "sell",
        "share",
        "show",
        "spoils",
        "steal",
        "study",
        "take",
        "tax",
        "teach",
        "transport",
        "turn",
        "weapon",
        "withdraw",
        "work",
    ];

    #[test]
    fn order_keyword_table_maps_all_66_verbs_in_cpp_index_order() {
        for (index, keyword) in ORDER_KEYWORDS.iter().enumerate() {
            assert_eq!(
                parse_order_verb(keyword).map(|verb| verb as i32),
                Some(index as i32),
                "keyword {keyword:?}"
            );
        }
        assert_eq!(OrderVerb::Count as i32, 66);
        assert_eq!(parse_order_verb("#ATLANTIS"), Some(OrderVerb::Atlantis));
        assert_eq!(parse_order_verb("not-an-order"), None);
    }

    #[test]
    fn parse_order_unit_id_matches_cpp_number_alias_and_foreign_alias_forms() {
        let mut number = "123 trailing".to_owned();
        assert_eq!(
            parse_unit_id(&mut number),
            Some(OrderUnitId {
                unitnum: 123,
                alias: 0,
                faction: 0
            })
        );
        assert_eq!(number, " trailing");

        let mut zero = "0 next".to_owned();
        assert_eq!(
            parse_unit_id(&mut zero),
            Some(OrderUnitId {
                unitnum: -1,
                alias: 0,
                faction: 0
            })
        );
        assert_eq!(zero, " next");

        let mut alias = "NeW 5 MOVE".to_owned();
        assert_eq!(
            parse_unit_id(&mut alias),
            Some(OrderUnitId {
                unitnum: 0,
                alias: 5,
                faction: 0
            })
        );
        assert_eq!(alias, " MOVE");

        let mut foreign_alias = "FaCtIoN 7 NeW 5 END".to_owned();
        assert_eq!(
            parse_unit_id(&mut foreign_alias),
            Some(OrderUnitId {
                unitnum: 0,
                alias: 5,
                faction: 7
            })
        );
        assert_eq!(foreign_alias, " END");

        let mut missing = "faction 7 new".to_owned();
        assert_eq!(parse_unit_id(&mut missing), None);
        assert_eq!(missing, "");

        let mut zero_alias = "new 0 trailing".to_owned();
        assert_eq!(parse_unit_id(&mut zero_alias), None);
        assert_eq!(zero_alias, " trailing");

        let mut invalid_foreign_prefix = "faction 7 old 5".to_owned();
        assert_eq!(parse_unit_id(&mut invalid_foreign_prefix), None);
        assert_eq!(invalid_foreign_prefix, " 5");

        let mut invalid_number = "word trailing".to_owned();
        assert_eq!(parse_unit_id(&mut invalid_number), None);
        assert_eq!(invalid_number, " trailing");

        let mut empty = String::new();
        assert_eq!(parse_unit_id(&mut empty), None);
    }

    #[test]
    fn order_payload_defaults_match_cpp_order_constructors() {
        let order = Order::default();
        assert_eq!(order.kind, OrderVerb::Count);
        assert_eq!(order.quiet, 0);
        assert_eq!(OrderData::default(), OrderData::None);

        let constructor_verbs = [
            MoveOrder::default().order.kind,
            WithdrawOrder::default().order.kind,
            GiveOrder::default().order.kind,
            StudyOrder::default().order.kind,
            TeachOrder::default().order.kind,
            ProduceOrder::default().order.kind,
            BuyOrder::default().order.kind,
            SellOrder::default().order.kind,
            AttackOrder::default().order.kind,
            BuildOrder::default().order.kind,
            SailOrder::default().order.kind,
            FindOrder::default().order.kind,
            StealOrder::default().order.kind,
            AssassinateOrder::default().order.kind,
            ForgetOrder::default().order.kind,
            ExchangeOrder::default().order.kind,
            TurnOrder::default().order.kind,
            CastOrder::default().order.kind,
            CastMindOrder::default().order.kind,
            CastRegionOrder::default().order.kind,
            TeleportOrder::default().order.kind,
            CastIntOrder::default().order.kind,
            CastUnitsOrder::default().order.kind,
            CastTransmuteOrder::default().order.kind,
            EvictOrder::default().order.kind,
            IdleOrder::default().order.kind,
            TransportOrder::default().order.kind,
            JoinOrder::default().order.kind,
        ];
        assert_eq!(
            constructor_verbs,
            [
                OrderVerb::Move,
                OrderVerb::Withdraw,
                OrderVerb::Give,
                OrderVerb::Study,
                OrderVerb::Teach,
                OrderVerb::Produce,
                OrderVerb::Buy,
                OrderVerb::Sell,
                OrderVerb::Attack,
                OrderVerb::Build,
                OrderVerb::Sail,
                OrderVerb::Find,
                OrderVerb::Steal,
                OrderVerb::Assassinate,
                OrderVerb::Forget,
                OrderVerb::Exchange,
                OrderVerb::Turn,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Cast,
                OrderVerb::Evict,
                OrderVerb::Idle,
                OrderVerb::Transport,
                OrderVerb::Join,
            ]
        );

        assert_eq!(MoveMode::None as i32, 0);
        assert_eq!(MoveMode::Sail as i32, 5);
        assert_eq!(
            (MOVE_PAUSE, MOVE_IN, MOVE_OUT, MOVE_ENTER),
            (97, 98, 99, 100)
        );
        assert_eq!(MoveDir::default().dir, 0);
        assert_eq!(MoveOrder::default().advancing, 0);
        assert!(MoveOrder::default().dirs.is_empty());
        assert_eq!(
            OrderUnitId::default(),
            OrderUnitId {
                unitnum: 0,
                alias: 0,
                faction: 0,
            }
        );
        assert_eq!(WithdrawOrder::default().amount, 0);
        let withdraw = WithdrawOrder::default();
        assert_eq!((withdraw.item, withdraw.amount), (0, 0));
        let give = GiveOrder::default();
        assert_eq!(
            (
                give.item,
                give.amount,
                give.except,
                give.unfinished,
                give.merge,
                give.target,
            ),
            (0, 0, 0, 0, 0, None)
        );
        assert_eq!(GiveOrder::take().order.kind, OrderVerb::Take);
        let study = StudyOrder::default();
        assert_eq!((study.skill, study.days, study.level), (0, 0, 0));
        assert!(TeachOrder::default().targets.is_empty());
        let produce = ProduceOrder::default();
        assert_eq!(
            (
                produce.item,
                produce.skill,
                produce.productivity,
                produce.target,
            ),
            (0, -1, 0, 0)
        );
        assert_eq!((BuyOrder::default().item, BuyOrder::default().num), (0, 0));
        assert_eq!(
            (SellOrder::default().item, SellOrder::default().num),
            (0, 0)
        );
        assert!(AttackOrder::default().targets.is_empty());
        let build = BuildOrder::default();
        assert_eq!(
            (build.target, build.new_building, build.needtocomplete),
            (None, -1, 0)
        );
        assert!(SailOrder::default().dirs.is_empty());
        assert_eq!(FindOrder::default().find, 0);
        let steal = StealOrder::default();
        assert_eq!((steal.target, steal.item), (None, 0));
        assert_eq!(AssassinateOrder::default().target, None);
        assert_eq!(ForgetOrder::default().skill, 0);
        let exchange = ExchangeOrder::default();
        assert_eq!(
            (
                exchange.give_item,
                exchange.give_amount,
                exchange.expect_item,
                exchange.expect_amount,
                exchange.exchange_status,
                exchange.target,
            ),
            (0, 0, 0, 0, -1, None)
        );
        let turn = TurnOrder::default();
        assert_eq!(turn.repeating, 0);
        assert!(turn.turn_orders.is_empty());
        assert_eq!(
            (CastOrder::default().spell, CastOrder::default().level),
            (0, 0)
        );
        let cast_mind = CastMindOrder::default();
        assert_eq!(
            (cast_mind.spell, cast_mind.level, cast_mind.id),
            (0, 0, None)
        );
        let cast_region = CastRegionOrder::default();
        assert_eq!(
            (
                cast_region.spell,
                cast_region.level,
                cast_region.xloc,
                cast_region.yloc,
                cast_region.zloc,
            ),
            (0, 0, 0, 0, 0)
        );
        let teleport = TeleportOrder::default();
        assert_eq!(
            (
                teleport.spell,
                teleport.level,
                teleport.xloc,
                teleport.yloc,
                teleport.zloc,
                teleport.gate,
            ),
            (0, 0, 0, 0, 0, 0)
        );
        assert!(teleport.units.is_empty());
        let cast_int = CastIntOrder::default();
        assert_eq!((cast_int.spell, cast_int.level, cast_int.target), (0, 0, 0));
        let cast_units = CastUnitsOrder::default();
        assert_eq!((cast_units.spell, cast_units.level), (0, 0));
        assert!(cast_units.units.is_empty());
        let cast_transmute = CastTransmuteOrder::default();
        assert_eq!(
            (
                cast_transmute.spell,
                cast_transmute.level,
                cast_transmute.item,
                cast_transmute.number,
            ),
            (0, 0, 0, 0)
        );
        assert!(EvictOrder::default().targets.is_empty());
        assert_eq!(IdleOrder::default().order.kind, OrderVerb::Idle);
        let transport = TransportOrder::default();
        assert_eq!(
            (
                transport.item,
                transport.amount,
                transport.except,
                transport.target
            ),
            (-1, 0, 0, None)
        );
        assert_eq!(
            TransportOrder::distribute().order.kind,
            OrderVerb::Distribute
        );
        let join = JoinOrder::default();
        assert_eq!((join.target, join.overload, join.merge), (None, 0, 0));

        let unit_orders = UnitOrders::default();
        assert_eq!(
            (
                unit_orders.destroy,
                unit_orders.enter,
                unit_orders.build,
                unit_orders.in_turn_block,
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(unit_orders.promote, None);
        assert!(unit_orders.find_orders.is_empty());
        assert!(unit_orders.give_orders.is_empty());
        assert!(unit_orders.withdraw_orders.is_empty());
        assert!(unit_orders.bank_orders.is_empty());
        assert!(unit_orders.buy_orders.is_empty());
        assert!(unit_orders.sell_orders.is_empty());
        assert!(unit_orders.forget_orders.is_empty());
        assert!(unit_orders.cast_orders.is_none());
        assert!(unit_orders.teleport_orders.is_none());
        assert!(unit_orders.steal_orders.is_none());
        assert!(unit_orders.month_orders.is_none());
        assert!(unit_orders.attack_orders.is_none());
        assert!(unit_orders.evict_orders.is_none());
        assert!(unit_orders.exchange_orders.is_empty());
        assert!(unit_orders.turn_orders.is_empty());
        assert!(unit_orders.transport_orders.is_empty());
        assert!(unit_orders.present_month_orders.is_none());
        assert!(unit_orders.join_orders.is_none());
    }
}
