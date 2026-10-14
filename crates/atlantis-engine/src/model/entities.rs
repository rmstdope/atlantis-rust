use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::defs::{self, ItemId, TerrainId};
use crate::orders::UnitOrders;

use super::{FactionId, ObjectId, RegionId, UnitId};
use super::{ItemList, SkillList};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionItemShow {
    pub item_type: ItemId,
    pub full: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionSkillShow {
    pub skill_type: usize,
    pub level: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Faction {
    pub num: i32,
    pub faction_types: Vec<(String, i32)>,
    pub last_change: i32,
    pub last_orders: i32,
    pub unclaimed: i32,
    pub bank_account: i32,
    pub interest: i32,
    pub name: Option<String>,
    pub address: Option<String>,
    pub password: Option<String>,
    pub times: i32,
    pub show_unit_attitudes: i32,
    pub template_format: i32,
    pub battle_log_format: i32,
    pub exists: i8,
    pub quit: i32,
    pub num_shows: i32,
    pub num_mages: i32,
    pub num_apprentices: i32,
    pub num_quartermasters: i32,
    pub num_tacticians: i32,
    pub activity: Vec<FactionActivityRecord>,
    pub present_regions: Vec<RegionId>,
    pub default_attitude: i32,
    pub attitudes: Vec<Attitude>,
    pub skills: SkillList,
    pub items: ItemList,
    pub extra_players: Vec<String>,
    pub errors: Vec<String>,
    pub events: Vec<String>,
    pub battles: Vec<String>,
    pub shows: Vec<FactionSkillShow>,
    pub item_shows: Vec<FactionItemShow>,
    pub object_shows: Vec<String>,
    pub region: Option<RegionId>,
    pub start_location: Option<RegionId>,
    pub no_start_leader: i32,
    pub start_turn: i32,
}

impl Default for Faction {
    fn default() -> Self {
        Self {
            num: 0,
            faction_types: Vec::new(),
            last_change: -6,
            last_orders: 0,
            unclaimed: 0,
            bank_account: 0,
            interest: 0,
            name: None,
            address: None,
            password: None,
            times: 0,
            show_unit_attitudes: 0,
            template_format: 0,
            battle_log_format: 0,
            exists: 1,
            quit: 0,
            num_shows: 0,
            num_mages: 0,
            num_apprentices: 0,
            num_quartermasters: 0,
            num_tacticians: 0,
            activity: Vec::new(),
            present_regions: Vec::new(),
            default_attitude: 2,
            attitudes: Vec::new(),
            skills: SkillList::default(),
            items: ItemList::default(),
            extra_players: Vec::new(),
            errors: Vec::new(),
            events: Vec::new(),
            battles: Vec::new(),
            shows: Vec::new(),
            item_shows: Vec::new(),
            object_shows: Vec::new(),
            region: None,
            start_location: None,
            no_start_leader: 0,
            start_turn: 0,
        }
    }
}

impl Faction {
    pub fn with_faction_types(faction_types: &[&str]) -> Self {
        Self {
            faction_types: faction_types
                .iter()
                .map(|faction_type| ((*faction_type).to_owned(), 1))
                .collect(),
            ..Self::default()
        }
    }

    pub fn new(number: i32, faction_types: &[&str]) -> Self {
        let mut faction = Self::with_faction_types(faction_types);
        faction.num = number;
        faction.name = Some(format!("Faction ({number})"));
        faction.address = Some("NoAddress".to_owned());
        faction.password = Some("none".to_owned());
        faction.times = 1;
        faction.template_format = 2;
        faction
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum FactionActivity {
    Tax = 1,
    Trade = 2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionActivityRecord {
    pub region: RegionId,
    pub activities: Vec<FactionActivity>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Attitude {
    pub faction_num: i32,
    pub attitude: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Quest {
    pub quest_type: i32,
    pub target: i32,
    pub objective_type: i32,
    pub objective_num: i32,
    pub building: i32,
    pub region_num: i32,
    pub region_name: String,
    pub destinations: BTreeSet<String>,
    pub rewards: ItemList,
}

impl Default for Quest {
    fn default() -> Self {
        Self {
            quest_type: -1,
            target: -1,
            objective_type: -1,
            objective_num: 0,
            building: -1,
            region_num: -1,
            region_name: "-".to_owned(),
            destinations: BTreeSet::new(),
            rewards: ItemList::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TownInfo {
    pub name: Option<String>,
    pub population: i32,
    pub activity: i32,
    pub habitat: i32,
    pub development: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum MarketType {
    Buy = 0,
    Sell = 1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Market {
    pub market_type: MarketType,
    pub item: i32,
    pub price: i32,
    pub amount: i32,
    pub min_population: i32,
    pub max_population: i32,
    pub min_amount: i32,
    pub max_amount: i32,
    pub base_price: i32,
    pub activity: i32,
}

impl Default for Market {
    fn default() -> Self {
        Self {
            market_type: MarketType::Buy,
            item: 0,
            price: 0,
            amount: 0,
            min_population: 0,
            max_population: 0,
            min_amount: 0,
            max_amount: 0,
            base_price: 0,
            activity: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Production {
    pub item_type: i32,
    pub base_amount: i32,
    pub amount: i32,
    pub skill: i32,
    pub productivity: i32,
    pub activity: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Region {
    pub name: String,
    pub num: i32,
    pub terrain_type: Option<TerrainId>,
    pub xloc: i32,
    pub yloc: i32,
    pub zloc: i32,
    pub building_seq: i32,
    pub gate: i32,
    pub gate_month: i32,
    pub gate_open: i32,
    pub weather: i32,
    pub town: Option<TownInfo>,
    pub race: i32,
    pub population: i32,
    pub base_population: i32,
    pub wages: i32,
    pub max_wages: i32,
    pub wealth: i32,
    pub habitat: i32,
    pub development: i32,
    pub max_development: i32,
    pub elevation: i32,
    pub humidity: i32,
    pub temperature: i32,
    pub vegetation: i32,
    pub culture: i32,
    pub migration_origins: Vec<RegionId>,
    pub migration_development: i32,
    pub immigrants: i32,
    pub emigrants: i32,
    pub improvement: i32,
    pub clearskies: i32,
    pub earthlore: i32,
    pub phantasmal_entertainment: i32,
    pub neighbors: [Option<RegionId>; 6],
    pub(super) object_ids: Vec<ObjectId>,
    pub new_fleets: BTreeMap<i32, i32>,
    pub fleet_alias: i32,
    pub(super) hell_unit_ids: Vec<UnitId>,
    pub farsees: Vec<FactionId>,
    pub passers: Vec<UnitId>,
    pub products: Vec<Production>,
    pub markets: Vec<Market>,
    pub visited: i32,
    pub distance: i32,
    pub next: Option<RegionId>,
}

impl Default for Region {
    fn default() -> Self {
        Self {
            name: "Region".to_owned(),
            num: 0,
            terrain_type: None,
            xloc: 0,
            yloc: 0,
            zloc: 0,
            building_seq: 1,
            gate: 0,
            gate_month: 0,
            gate_open: 1,
            weather: 0,
            town: None,
            race: 0,
            population: 0,
            base_population: 0,
            wages: 0,
            max_wages: 0,
            wealth: 0,
            habitat: 0,
            development: 0,
            max_development: 0,
            elevation: 0,
            humidity: 0,
            temperature: 0,
            vegetation: 0,
            culture: 0,
            migration_origins: Vec::new(),
            migration_development: 0,
            immigrants: 0,
            emigrants: 0,
            improvement: 0,
            clearskies: 0,
            earthlore: 0,
            phantasmal_entertainment: 0,
            neighbors: [None; 6],
            object_ids: Vec::new(),
            new_fleets: BTreeMap::new(),
            fleet_alias: 0,
            hell_unit_ids: Vec::new(),
            farsees: Vec::new(),
            passers: Vec::new(),
            products: Vec::new(),
            markets: Vec::new(),
            visited: 0,
            distance: 0,
            next: None,
        }
    }
}

impl Region {
    pub fn object_ids(&self) -> &[ObjectId] {
        &self.object_ids
    }

    pub fn hell_unit_ids(&self) -> &[UnitId] {
        &self.hell_unit_ids
    }

    pub(super) fn add_object(&mut self, object_id: ObjectId) {
        self.object_ids.push(object_id);
    }

    pub(super) fn add_hell_unit(&mut self, unit_id: UnitId) {
        self.hell_unit_ids.push(unit_id);
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Object {
    pub name: Option<String>,
    pub describe: Option<String>,
    pub region: Option<RegionId>,
    pub inner: i32,
    pub num: i32,
    pub object_type: i32,
    pub incomplete: i32,
    pub capacity: i32,
    pub flying: i32,
    pub load: i32,
    pub runes: i32,
    pub prevdir: i32,
    pub mages: i32,
    pub shipno: i32,
    pub movepoints: i32,
    pub destroyed: i32,
    pub(super) unit_ids: Vec<UnitId>,
    pub ships: ItemList,
}

impl Default for Object {
    fn default() -> Self {
        Self {
            name: Some("Dummy".to_owned()),
            describe: None,
            region: None,
            inner: -1,
            num: 0,
            object_type: defs::objects::O_DUMMY as i32,
            incomplete: 0,
            capacity: 0,
            flying: 0,
            load: 0,
            runes: 0,
            prevdir: -1,
            mages: 0,
            shipno: 0,
            movepoints: 0,
            destroyed: 0,
            unit_ids: Vec::new(),
            ships: ItemList::default(),
        }
    }
}

impl Object {
    /// C++: `object.cpp` `Object::Object`; pass `Globals->PHASED_MOVE_OFFSET % Globals->MAX_SPEED`.
    pub fn new(default_move_points: i32) -> Self {
        Self {
            movepoints: default_move_points,
            ..Self::default()
        }
    }

    pub fn unit_ids(&self) -> &[UnitId] {
        &self.unit_ids
    }

    pub(super) fn add_unit(&mut self, unit_id: UnitId) {
        self.unit_ids.push(unit_id);
    }

    /// C++: `object.cpp` `Object::IsFleet`.
    pub fn is_fleet(&self, object_defs: &[defs::ObjectDef]) -> Option<bool> {
        let object_type = usize::try_from(self.object_type).ok()?;
        let definition = object_defs.get(object_type)?;
        Some(
            self.object_type == defs::objects::O_FLEET as i32
                || definition.sailors > 0
                || !self.ships.entries().is_empty(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum UnitType {
    #[default]
    Normal = 0,
    Mage = 1,
    Guard = 2,
    WanderingMonster = 3,
    GuardMage = 4,
    Apprentice = 5,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum GuardMode {
    #[default]
    None = 0,
    Guard = 1,
    Avoid = 2,
    Set = 3,
    Advance = 4,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Unit {
    pub name: Option<String>,
    pub describe: Option<String>,
    pub num: i32,
    pub unit_type: UnitType,
    pub faction: Option<FactionId>,
    pub form_faction: Option<FactionId>,
    pub object: Option<ObjectId>,
    pub alias: i32,
    pub gm_alias: i32,
    pub guard: GuardMode,
    pub reveal: i32,
    pub flags: i32,
    pub taxing: i32,
    pub movepoints: i32,
    pub canattack: i32,
    pub nomove: i32,
    pub routed: i32,
    pub skills: SkillList,
    pub items: ItemList,
    pub combat: i32,
    pub ready_item: i32,
    pub ready_weapon: [i32; 4],
    pub ready_armor: [i32; 4],
    pub old_orders: Vec<String>,
    #[serde(skip)]
    pub orders: UnitOrders,
    pub needed: i32,
    pub hunger: i32,
    pub stomach_space: i32,
    pub losses: i32,
    pub free: i32,
    pub practiced: i32,
    pub moved: i32,
    pub phase: i32,
    pub saved_movement: i32,
    pub saved_move_dir: i32,
    pub advance_from: Option<RegionId>,
    pub present_taxing: i32,
    pub former: Option<UnitId>,
    pub format: i32,
    pub visited: BTreeSet<String>,
    pub raised: i32,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            name: None,
            describe: None,
            num: 0,
            unit_type: UnitType::Normal,
            faction: None,
            form_faction: None,
            object: None,
            alias: 0,
            gm_alias: 0,
            guard: GuardMode::None,
            reveal: 0,
            flags: 0x0002,
            taxing: 0,
            movepoints: 0,
            canattack: 1,
            nomove: 0,
            routed: 0,
            skills: SkillList::default(),
            items: ItemList::default(),
            combat: -1,
            ready_item: -1,
            ready_weapon: [-1; 4],
            ready_armor: [-1; 4],
            old_orders: Vec::new(),
            orders: UnitOrders::default(),
            needed: 0,
            hunger: 0,
            stomach_space: 0,
            losses: 0,
            free: 0,
            practiced: 0,
            moved: 0,
            phase: -1,
            saved_movement: 0,
            saved_move_dir: -1,
            advance_from: None,
            present_taxing: 0,
            former: None,
            format: 0,
            visited: BTreeSet::new(),
            raised: 0,
        }
    }
}

impl Unit {
    /// C++: `unit.cpp` `Unit::Unit(int, Faction *, int)`; movement points come from the active rules.
    pub fn new(
        number: i32,
        faction: Option<FactionId>,
        alias: i32,
        default_move_points: i32,
    ) -> Self {
        Self {
            name: Some(format!("Unit ({number})")),
            num: number,
            faction,
            form_faction: faction,
            alias,
            movepoints: default_move_points,
            ..Self::default()
        }
    }

    /// C++: `unit.cpp` `Unit::ClearOrders`.
    pub fn clear_orders(&mut self) {
        self.canattack = 1;
        self.nomove = 0;
        self.routed = 0;
        self.orders.enter = 0;
        self.orders.build = 0;
        self.orders.destroy = 0;
        self.orders.attack_orders = None;
        self.orders.evict_orders = None;
        self.orders.steal_orders = None;
        self.orders.promote = None;
        self.taxing = 0;
        self.advance_from = None;
        self.orders.month_orders = None;
        self.orders.in_turn_block = 0;
        self.present_taxing = 0;
        self.orders.present_month_orders = None;
        self.orders.cast_orders = None;
        self.orders.teleport_orders = None;
    }
}

#[cfg(test)]
mod tests {
    use super::Unit;
    use crate::model::RegionId;
    use crate::orders::{
        AssassinateOrder, AttackOrder, BuyOrder, CastOrder, CastOrderData, ExchangeOrder,
        FindOrder, ForgetOrder, GiveOrder, JoinOrder, MoveOrder, Order, OrderData, OrderUnitId,
        SellOrder, StealOrderData, TeleportOrder, TransportOrder, TurnOrder, UnitOrders,
        WithdrawOrder,
    };

    #[test]
    fn clear_orders_resets_cpp_order_slots_without_dropping_retained_queues() {
        assert_eq!(Unit::default().canattack, 1);

        let mut unit = Unit {
            canattack: 0,
            nomove: 1,
            routed: 1,
            taxing: 7,
            advance_from: Some(RegionId::new(4)),
            present_taxing: 3,
            orders: UnitOrders {
                destroy: 1,
                enter: 2,
                build: 3,
                promote: Some(OrderUnitId {
                    unitnum: 0,
                    alias: 9,
                    faction: 0,
                }),
                find_orders: vec![FindOrder {
                    find: 1,
                    ..FindOrder::default()
                }],
                give_orders: vec![GiveOrder::default()],
                withdraw_orders: vec![WithdrawOrder::default()],
                bank_orders: vec![Order::default()],
                buy_orders: vec![BuyOrder::default()],
                sell_orders: vec![SellOrder::default()],
                forget_orders: vec![ForgetOrder::default()],
                cast_orders: Some(CastOrderData::Cast(CastOrder::default())),
                teleport_orders: Some(TeleportOrder::default()),
                steal_orders: Some(StealOrderData::Assassinate(AssassinateOrder::default())),
                month_orders: Some(OrderData::Move(MoveOrder::default())),
                attack_orders: Some(AttackOrder::default()),
                evict_orders: Some(crate::orders::EvictOrder::default()),
                exchange_orders: vec![ExchangeOrder::default()],
                turn_orders: vec![TurnOrder::default()],
                in_turn_block: 1,
                present_month_orders: Some(OrderData::Move(MoveOrder::default())),
                transport_orders: vec![TransportOrder::default()],
                join_orders: Some(JoinOrder::default()),
            },
            ..Unit::default()
        };

        unit.clear_orders();

        assert_eq!(unit.canattack, 1);
        assert_eq!(unit.nomove, 0);
        assert_eq!(unit.routed, 0);
        assert_eq!(unit.taxing, 0);
        assert_eq!(unit.advance_from, None);
        assert_eq!(unit.present_taxing, 0);
        assert_eq!(unit.orders.destroy, 0);
        assert_eq!(unit.orders.enter, 0);
        assert_eq!(unit.orders.build, 0);
        assert_eq!(unit.orders.promote, None);
        assert_eq!(unit.orders.attack_orders, None);
        assert_eq!(unit.orders.evict_orders, None);
        assert_eq!(unit.orders.steal_orders, None);
        assert_eq!(unit.orders.month_orders, None);
        assert_eq!(unit.orders.in_turn_block, 0);
        assert_eq!(unit.orders.present_month_orders, None);
        assert_eq!(unit.orders.cast_orders, None);
        assert_eq!(unit.orders.teleport_orders, None);
        assert_eq!(unit.orders.find_orders.len(), 1);
        assert_eq!(unit.orders.give_orders.len(), 1);
        assert_eq!(unit.orders.withdraw_orders.len(), 1);
        assert_eq!(unit.orders.bank_orders.len(), 1);
        assert_eq!(unit.orders.buy_orders.len(), 1);
        assert_eq!(unit.orders.sell_orders.len(), 1);
        assert_eq!(unit.orders.forget_orders.len(), 1);
        assert_eq!(unit.orders.exchange_orders.len(), 1);
        assert_eq!(unit.orders.turn_orders.len(), 1);
        assert_eq!(unit.orders.transport_orders.len(), 1);
        assert_eq!(unit.orders.join_orders, Some(JoinOrder::default()));
    }
}
