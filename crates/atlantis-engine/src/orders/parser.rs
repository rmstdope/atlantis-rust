use std::collections::VecDeque;
use std::io::{self, Cursor};

use crate::defs::{
    enums::flags as tax_flags, skills::skill_flags, BattleItemFlags, BuildTarget, GameDefs,
    ItemFlags, ItemId, ItemTypeFlags, RulesetTables, WeaponFlags,
};
use crate::model::{Faction, FactionId, Game, ModelError, ObjectId, UnitId, UnitType};
use crate::ruleset::Ruleset;
use crate::text::{get_at, get_token, is_equal, value, TextInput};

use super::{
    parse_order_verb, parse_unit_id, BuildOrder, BuyOrder, CastIntOrder, CastMindOrder, CastOrder,
    CastOrderData, CastRegionOrder, CastTransmuteOrder, CastUnitsOrder, ExchangeOrder, GiveOrder,
    JoinOrder, MoveDir, MoveOrder, OrderData, OrderVerb, ProduceOrder, SailOrder, SellOrder,
    TeleportOrder, TransportOrder, TurnOrder, WithdrawOrder,
};

const FLAG_AUTOTAX: i32 = 0x0004;
const TAX_NONE: i32 = 0;
const TAX_TAX: i32 = 1;
const TAX_PILLAGE: i32 = 2;
const GENERIC_CAST_SPELL_NAMES: &[&str] = &[
    "construct portal",
    "enchant swords",
    "enchant armor",
    "enchant shields",
    "construct gate",
    "engrave runes of warding",
    "summon imps",
    "summon demon",
    "summon balrog",
    "summon skeletons",
    "raise undead",
    "summon lich",
    "dragon lore",
    "wolf lore",
    "earth lore",
    "summon wind",
    "create ring of invisibility",
    "create cloak of invulnerability",
    "create staff of fire",
    "create staff of lightning",
    "create amulet of true seeing",
    "create amulet of protection",
    "create runesword",
    "create shieldstone",
    "create magic carpet",
    "create flaming sword",
    "create food",
    "create aegis",
    "create windchime",
    "create gate crystal",
    "create staff of healing",
    "create scrying orb",
    "create cornucopia",
    "create book of exorcism",
    "create holy symbol",
    "create censer",
    "blasphemous ritual",
    "phantasmal entertainment",
];
const TAX_AUTO: i32 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderParseMode {
    Turn,
    Check,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedOrderLine {
    pub raw: String,
    pub indent: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderParseResult {
    pub errors: Vec<String>,
    pub lines: Vec<ParsedOrderLine>,
}

#[derive(Debug)]
pub enum OrderParseError {
    Input(io::Error),
    Model(ModelError),
    FormUnitHasNoObject(UnitId),
}

impl std::fmt::Display for OrderParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => write!(formatter, "could not read order input: {error}"),
            Self::Model(error) => write!(formatter, "{error}"),
            Self::FormUnitHasNoObject(unit_id) => {
                write!(formatter, "form unit {} has no object", unit_id.index())
            }
        }
    }
}

impl std::error::Error for OrderParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::FormUnitHasNoObject(_) => None,
        }
    }
}

impl From<io::Error> for OrderParseError {
    fn from(error: io::Error) -> Self {
        Self::Input(error)
    }
}

impl From<ModelError> for OrderParseError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

pub fn parse_order_file(
    game: &mut Game,
    defs: &GameDefs,
    tables: &RulesetTables,
    input: &[u8],
    mode: OrderParseMode,
) -> Result<OrderParseResult, OrderParseError> {
    Parser::new(game, defs, tables, mode).parse(input)
}

struct Parser<'a> {
    game: &'a mut Game,
    defs: &'a GameDefs,
    tables: &'a RulesetTables,
    mode: OrderParseMode,
    result: OrderParseResult,
    faction_id: Option<FactionId>,
    unit_stack: Vec<UnitId>,
    indent: i32,
}

impl<'a> Parser<'a> {
    fn new(
        game: &'a mut Game,
        defs: &'a GameDefs,
        tables: &'a RulesetTables,
        mode: OrderParseMode,
    ) -> Self {
        Self {
            game,
            defs,
            tables,
            mode,
            result: OrderParseResult::default(),
            faction_id: None,
            unit_stack: Vec::new(),
            indent: 0,
        }
    }

    fn parse(mut self, input: &[u8]) -> Result<OrderParseResult, OrderParseError> {
        let mut source = TextInput::new(Cursor::new(input));
        let mut lines = Vec::new();
        while let Some(line) = source.get_order_line()? {
            lines.push(line);
        }

        let mut cursor = 0;
        let mut pending = VecDeque::new();
        while cursor < lines.len() || !pending.is_empty() {
            let raw_line = if let Some(line) = pending.pop_front() {
                line
            } else {
                let line = lines[cursor].clone();
                cursor += 1;
                line
            };
            self.parse_line(raw_line, &lines, &mut cursor, &mut pending)?;
        }
        self.unwind_units();
        if self.mode == OrderParseMode::Turn {
            self.default_work_order();
        }
        Ok(self.result)
    }

    /// C++: `game.cpp` `Game::DefaultWorkOrder`.
    fn default_work_order(&mut self) {
        let mut tax_tables = None;
        let region_ids = self.game.region_ids().to_vec();
        for region_id in region_ids {
            let Some(region) = self.game.region(region_id) else {
                continue;
            };
            if region.terrain_type == Some(crate::defs::terrains::R_NEXUS) {
                continue;
            }
            let object_ids = region.object_ids().to_vec();
            for object_id in object_ids {
                let Some(object) = self.game.object(object_id) else {
                    continue;
                };
                let unit_ids = object.unit_ids().to_vec();
                for unit_id in unit_ids {
                    let Some(unit) = self.game.unit(unit_id) else {
                        continue;
                    };
                    let Some(faction_id) = unit.faction else {
                        continue;
                    };
                    let Some(faction) = self.game.faction(faction_id) else {
                        continue;
                    };
                    if unit.orders.month_orders.is_some()
                        || faction_is_npc(faction)
                        || (self.defs.tax_pillage_month_long != 0 && unit.taxing != 0)
                    {
                        continue;
                    }

                    let autotax = unit.flags & FLAG_AUTOTAX != 0
                        && self.defs.tax_pillage_month_long != 0
                        && has_taxers(
                            unit,
                            self.defs,
                            tax_tables.get_or_insert_with(|| {
                                <dyn Ruleset>::from_name(self.defs.ruleset_name)
                                    .expect("parser ruleset is supported")
                                    .tables()
                            }),
                        );
                    let unit = self
                        .game
                        .unit_mut(unit_id)
                        .expect("unit remains in the game");
                    if autotax {
                        unit.taxing = TAX_AUTO;
                    } else if self.defs.default_work_order != 0 {
                        let order = super::ProduceOrder {
                            order: super::Order {
                                quiet: 1,
                                ..super::Order::new(OrderVerb::Produce)
                            },
                            item: ItemId::Silver.index() as i32,
                            skill: -1,
                            target: 0,
                            ..super::ProduceOrder::default()
                        };
                        unit.orders.month_orders = Some(super::OrderData::Produce(order));
                    }
                }
            }
        }
    }

    fn parse_line(
        &mut self,
        raw_line: String,
        lines: &[String],
        cursor: &mut usize,
        pending: &mut VecDeque<String>,
    ) -> Result<(), OrderParseError> {
        let mut line = raw_line.clone();
        let at_sign = get_at(&mut line);
        let Some(token) = get_token(&mut line) else {
            if at_sign && self.mode == OrderParseMode::Turn {
                self.store_old_order(&raw_line, true, false);
            }
            self.record_check_line(raw_line);
            return Ok(());
        };
        let verb = parse_order_verb(&token);

        if matches!(verb, Some(OrderVerb::EndDirective | OrderVerb::Unit)) {
            self.indent = 0;
        }
        if matches!(verb, Some(OrderVerb::EndForm | OrderVerb::EndTurn)) {
            self.indent -= 1;
        }

        match verb {
            Some(OrderVerb::Atlantis) => self.atlantis(&mut line),
            Some(OrderVerb::EndDirective) => {
                self.unwind_units();
                self.faction_id = None;
                Ok(())
            }
            Some(OrderVerb::Unit) => self.select_unit(&mut line),
            Some(OrderVerb::Form) => self.form_unit(&mut line, &raw_line, at_sign),
            Some(OrderVerb::EndForm) => {
                self.end_form(&raw_line);
                Ok(())
            }
            Some(OrderVerb::Turn) => self.turn(&raw_line, at_sign, lines, cursor, pending),
            Some(OrderVerb::EndTurn) => {
                self.end_turn();
                Ok(())
            }
            _ => {
                self.dispatch_order(&token, verb, &mut line, &raw_line, at_sign);
                Ok(())
            }
        }?;

        self.record_check_line(raw_line);
        if matches!(verb, Some(OrderVerb::Form | OrderVerb::Turn)) {
            self.indent += 1;
        }
        Ok(())
    }

    fn atlantis(&mut self, line: &mut String) -> Result<(), OrderParseError> {
        if self.faction_id.is_some() {
            self.record_error(self.faction_id, None, "No #END statement given.");
        }
        self.faction_id = None;

        let Some(number) = get_token(line) else {
            self.record_error(None, None, "No faction number given on #atlantis line.");
            return Ok(());
        };
        let number = value(&number);
        let selected_faction = match self.mode {
            OrderParseMode::Turn => self.find_faction(number),
            OrderParseMode::Check => self.game.faction_ids().first().copied(),
        };
        let Some(selected_faction) = selected_faction else {
            return Ok(());
        };

        let password = get_token(line);
        if self.mode == OrderParseMode::Check {
            if password.is_none() {
                self.record_error(
                    Some(selected_faction),
                    None,
                    "Warning: No password on #atlantis line.",
                );
                self.record_error(
                    Some(selected_faction),
                    None,
                    "If this is your first turn, ignore this error.",
                );
            }
        } else {
            let stored_password = self
                .game
                .faction(selected_faction)
                .and_then(|faction| faction.password.as_deref())
                .unwrap_or("none");
            if !is_equal(stored_password, "none")
                && password
                    .as_deref()
                    .is_none_or(|password| !is_equal(stored_password, password))
            {
                self.record_error(
                    Some(selected_faction),
                    None,
                    "Incorrect password on #atlantis line.",
                );
                return Ok(());
            }

            if self.game.guard_faction == Some(selected_faction)
                || self.game.monster_faction == Some(selected_faction)
            {
                return Ok(());
            }

            if self.defs.lastorders_maintained_by_scripts == 0 {
                let turn_number = (self.game.year - 1) * 12 + self.game.month + 1;
                if let Some(faction) = self.game.faction_mut(selected_faction) {
                    faction.last_orders = turn_number;
                }
            }
        }

        self.unit_stack.clear();
        self.faction_id = Some(selected_faction);
        Ok(())
    }

    fn select_unit(&mut self, line: &mut String) -> Result<(), OrderParseError> {
        let Some(faction_id) = self.faction_id else {
            return Ok(());
        };
        self.unwind_units();

        let Some(token) = get_token(line) else {
            self.record_error(Some(faction_id), None, "UNIT without unit number.");
            return Ok(());
        };
        let number = value(&token);
        if self.mode == OrderParseMode::Check {
            if number == 0 {
                self.record_error(Some(faction_id), None, "Invalid unit number.");
                return Ok(());
            }
            self.unit_stack.extend(self.first_unit());
        } else if let Some(unit_id) = self.game.get_unit_id(number) {
            let unit_faction = self.game.unit(unit_id).and_then(|unit| unit.faction);
            if unit_faction == Some(faction_id) {
                self.unit_stack.push(unit_id);
            } else {
                self.record_error(
                    Some(faction_id),
                    None,
                    &format!("{token} is not your unit."),
                );
                return Ok(());
            }
        } else {
            self.record_error(
                Some(faction_id),
                None,
                &format!("{token} is not your unit."),
            );
            return Ok(());
        }

        if let Some(unit_id) = self.current_unit() {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                if self.mode == OrderParseMode::Check {
                    unit.orders.month_orders = None;
                } else {
                    unit.clear_orders();
                }
            }
        }
        Ok(())
    }

    fn form_unit(
        &mut self,
        line: &mut String,
        raw_line: &str,
        at_sign: bool,
    ) -> Result<(), OrderParseError> {
        let Some(former_id) = self.current_unit() else {
            if self.faction_id.is_some() {
                self.record_error(
                    self.faction_id,
                    None,
                    "Order given without a unit selected.",
                );
            }
            return Ok(());
        };
        let (already_formed, in_turn_block) = self
            .game
            .unit(former_id)
            .map(|unit| (unit.former.is_some(), unit.orders.in_turn_block != 0))
            .unwrap_or((false, false));
        if already_formed && !in_turn_block {
            self.record_unit_error(former_id, "FORM: cannot nest.");
            return Ok(());
        }

        let Some(alias) = get_token(line).map(|token| value(&token)) else {
            self.record_unit_error(former_id, "Must give alias in FORM order.");
            return Ok(());
        };
        if alias == 0 {
            self.record_unit_error(former_id, "Must give alias in FORM order.");
            return Ok(());
        }

        let (faction_id, object_id, flags, guard, reveal) = {
            let unit = self.game.unit(former_id).expect("form parent exists");
            (
                unit.faction,
                unit.object
                    .ok_or(OrderParseError::FormUnitHasNoObject(former_id))?,
                unit.flags,
                unit.guard,
                unit.reveal,
            )
        };
        if self.mode == OrderParseMode::Turn && self.alias_exists(object_id, faction_id, alias) {
            self.record_unit_error(former_id, "Alias multiply defined.");
            return Ok(());
        }
        let Some(faction_id) = faction_id else {
            return Ok(());
        };

        let form_id = self
            .game
            .get_new_unit(Some(faction_id), alias, self.defs.max_speed)?;
        self.game.attach_unit(object_id, form_id)?;
        {
            let form = self.game.unit_mut(form_id).expect("new form unit exists");
            form.flags = flags & !FLAG_AUTOTAX;
            form.guard = if guard == crate::model::GuardMode::Avoid {
                crate::model::GuardMode::Avoid
            } else {
                crate::model::GuardMode::None
            };
            form.reveal = reveal;
            form.former = Some(former_id);
        }
        if let Some(former) = self.game.unit_mut(former_id) {
            former.format = i32::from(at_sign);
        }
        if self.mode == OrderParseMode::Turn {
            self.game
                .unit_mut(form_id)
                .expect("new form unit exists")
                .clear_orders();
        }
        self.unit_stack.push(form_id);

        if self.mode == OrderParseMode::Turn
            && self
                .game
                .unit(former_id)
                .is_some_and(|former| former.format != 0)
        {
            self.game
                .unit_mut(former_id)
                .expect("form parent exists")
                .old_orders
                .push(raw_line.to_owned());
        }
        Ok(())
    }

    fn end_form(&mut self, raw_line: &str) {
        let Some(unit_id) = self.current_unit() else {
            if self.faction_id.is_some() {
                self.record_error(self.faction_id, None, "END: without FORM.");
            }
            return;
        };
        let (former_id, in_turn_block) = self
            .game
            .unit(unit_id)
            .map(|unit| (unit.former, unit.orders.in_turn_block != 0))
            .unwrap_or((None, false));
        let Some(former_id) = former_id else {
            self.record_unit_error(unit_id, "END: without FORM.");
            return;
        };
        if in_turn_block {
            self.record_unit_error(unit_id, "TURN: without ENDTURN");
        }
        if self.mode == OrderParseMode::Turn
            && self
                .game
                .unit(former_id)
                .is_some_and(|former| former.format != 0)
        {
            self.game
                .unit_mut(former_id)
                .expect("form parent exists")
                .old_orders
                .push(raw_line.to_owned());
        }
        if self.mode == OrderParseMode::Check {
            self.game
                .unit_mut(unit_id)
                .expect("form unit exists")
                .clear_orders();
        }
        self.unit_stack.pop();
    }

    fn turn(
        &mut self,
        raw_line: &str,
        repeating: bool,
        lines: &[String],
        cursor: &mut usize,
        pending: &mut VecDeque<String>,
    ) -> Result<(), OrderParseError> {
        let Some(faction_id) = self.faction_id else {
            return Ok(());
        };
        let Some(unit_id) = self.current_unit() else {
            self.record_error(
                Some(faction_id),
                None,
                "Order given without a unit selected.",
            );
            return Ok(());
        };
        if self
            .game
            .unit(unit_id)
            .is_some_and(|unit| unit.orders.in_turn_block != 0)
        {
            self.record_unit_error(unit_id, "TURN: cannot nest");
            return Ok(());
        }

        if self.mode == OrderParseMode::Check {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.in_turn_block = 1;
                unit.orders.present_month_orders = unit.orders.month_orders.take();
                unit.present_taxing = unit.taxing;
                unit.taxing = 0;
            }
            return Ok(());
        }

        let former_id = self
            .game
            .unit(unit_id)
            .and_then(|unit| unit.former)
            .filter(|former_id| {
                self.game
                    .unit(*former_id)
                    .is_some_and(|former| former.format != 0)
            });
        if let Some(former_id) = former_id {
            if let Some(former) = self.game.unit_mut(former_id) {
                former.old_orders.push(raw_line.to_owned());
            }
        }

        if let Some(sentinel) = self.capture_turn(unit_id, repeating, lines, cursor) {
            pending.push_front(sentinel);
        }
        Ok(())
    }

    fn end_turn(&mut self) {
        let Some(unit_id) = self.current_unit() else {
            if self.faction_id.is_some() {
                self.record_error(self.faction_id, None, "ENDTURN: without TURN.");
            }
            return;
        };
        let in_turn_block = self
            .game
            .unit(unit_id)
            .is_some_and(|unit| unit.orders.in_turn_block != 0);
        if !in_turn_block {
            self.record_unit_error(unit_id, "ENDTURN: without TURN.");
            return;
        }
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.month_orders = unit.orders.present_month_orders.take();
            unit.taxing = unit.present_taxing;
            unit.present_taxing = 0;
            unit.orders.in_turn_block = 0;
        }
    }

    fn capture_turn(
        &mut self,
        unit_id: UnitId,
        repeating: bool,
        lines: &[String],
        cursor: &mut usize,
    ) -> Option<String> {
        let mut turn_order = TurnOrder {
            repeating: if repeating { 1 } else { 0 },
            ..TurnOrder::default()
        };
        let mut turn_depth = 1;
        let mut turn_last = true;
        let mut form_depth = 0;

        while turn_depth > 0 {
            let raw_line = if *cursor < lines.len() {
                let line = lines[*cursor].clone();
                *cursor += 1;
                line
            } else {
                "#end".to_owned()
            };
            let mut line = raw_line.clone();
            get_at(&mut line);
            let Some(token) = get_token(&mut line) else {
                continue;
            };

            match parse_order_verb(&token) {
                Some(OrderVerb::Turn) => {
                    if turn_last {
                        self.record_unit_error(unit_id, "TURN: cannot nest.");
                    } else {
                        turn_depth += 1;
                        turn_order.turn_orders.push(raw_line.clone());
                        turn_last = true;
                    }
                }
                Some(OrderVerb::Form) => {
                    if !turn_last {
                        self.record_unit_error(unit_id, "FORM: cannot nest.");
                    } else {
                        turn_last = false;
                        form_depth += 1;
                        turn_order.turn_orders.push(raw_line.clone());
                    }
                }
                Some(OrderVerb::EndForm) => {
                    if turn_last {
                        let unit_has_former = self
                            .game
                            .unit(unit_id)
                            .is_some_and(|unit| unit.former.is_some());
                        if form_depth + i32::from(unit_has_former) == 0 {
                            self.record_unit_error(unit_id, "END: without FORM.");
                            continue;
                        }
                        self.record_unit_error(unit_id, "TURN: without ENDTURN.");
                        turn_depth -= 1;
                        if turn_depth == 0 {
                            self.store_turn_order(unit_id, turn_order);
                            return Some(raw_line);
                        }
                    }
                    form_depth -= 1;
                    turn_order.turn_orders.push(raw_line.clone());
                    turn_last = true;
                }
                Some(OrderVerb::Unit | OrderVerb::EndDirective) => {
                    if !turn_last {
                        self.record_unit_error(unit_id, "FORM: without END.");
                    }
                    while turn_depth > 1 {
                        turn_depth -= 1;
                        self.record_unit_error(unit_id, "TURN: without ENDTURN.");
                        self.record_unit_error(unit_id, "FORM: without END.");
                    }
                    self.record_unit_error(unit_id, "TURN: without ENDTURN.");
                    self.store_turn_order(unit_id, turn_order);
                    return Some(raw_line);
                }
                Some(OrderVerb::EndTurn) => {
                    if !turn_last {
                        self.record_unit_error(unit_id, "ENDTURN: without TURN.");
                    } else {
                        turn_depth -= 1;
                        if turn_depth > 0 {
                            turn_order.turn_orders.push(raw_line.clone());
                        }
                        turn_last = false;
                    }
                }
                _ => turn_order.turn_orders.push(raw_line.clone()),
            }

            let former_id =
                self.game
                    .unit(unit_id)
                    .and_then(|unit| unit.former)
                    .filter(|former_id| {
                        self.game
                            .unit(*former_id)
                            .is_some_and(|former| former.format != 0)
                    });
            if let Some(former_id) = former_id {
                if let Some(former) = self.game.unit_mut(former_id) {
                    former.old_orders.push(raw_line);
                }
            }
        }
        self.store_turn_order(unit_id, turn_order);
        None
    }

    fn store_turn_order(&mut self, unit_id: UnitId, order: TurnOrder) {
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.turn_orders.push(order);
        }
    }

    fn dispatch_order(
        &mut self,
        token: &str,
        verb: Option<OrderVerb>,
        line: &mut String,
        raw_line: &str,
        at_sign: bool,
    ) {
        let Some(faction_id) = self.faction_id else {
            return;
        };
        let Some(unit_id) = self.current_unit() else {
            if verb.is_none() {
                self.record_error(
                    Some(faction_id),
                    None,
                    &format!("{token} is not a valid order."),
                );
            } else {
                self.record_error(
                    Some(faction_id),
                    None,
                    "Order given without a unit selected.",
                );
            }
            return;
        };

        if verb.is_none() {
            self.record_unit_error(unit_id, &format!("{token} is not a valid order."));
            return;
        }
        if self.mode == OrderParseMode::Turn {
            self.store_old_order(raw_line, at_sign, true);
        }

        match verb {
            Some(OrderVerb::Cast) => self.parse_cast_order(unit_id, line),
            Some(verb @ (OrderVerb::Move | OrderVerb::Advance | OrderVerb::Sail)) => {
                self.parse_movement_order(unit_id, verb, line);
            }
            Some(OrderVerb::Study) => self.parse_study_order(unit_id, line),
            Some(OrderVerb::Teach) => self.parse_teach_order(unit_id, line),
            Some(OrderVerb::Forget) => self.parse_forget_order(unit_id, line),
            Some(OrderVerb::Attack) => self.parse_attack_order(unit_id, line),
            Some(OrderVerb::Evict) => self.parse_evict_order(unit_id, line),
            Some(OrderVerb::Steal) => self.parse_steal_order(unit_id, line),
            Some(OrderVerb::Assassinate) => self.parse_assassinate_order(unit_id, line),
            Some(OrderVerb::Promote) => self.parse_promote_order(unit_id, line),
            Some(OrderVerb::Destroy) => self.parse_destroy_order(unit_id),
            Some(OrderVerb::Enter) => self.parse_enter_order(unit_id, line),
            Some(OrderVerb::Leave) => self.parse_leave_order(unit_id),
            _ => {}
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessStudyOrder`.
    fn parse_study_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(skill) = get_token(line) else {
            self.record_unit_error(unit_id, "STUDY: No skill given.");
            return;
        };
        let skill = crate::defs::skills::parse_skill(&self.ruleset_tables().skills.skills, &skill)
            .and_then(|skill| i32::try_from(skill).ok())
            .unwrap_or(-1);
        let level = get_token(line).map_or(-1, |token| value(&token));
        let (has_month_order, in_turn_block, is_taxing) = self
            .game
            .unit(unit_id)
            .map(|unit| {
                (
                    unit.orders.month_orders.is_some(),
                    unit.orders.in_turn_block != 0,
                    matches!(unit.taxing, TAX_TAX | TAX_PILLAGE),
                )
            })
            .unwrap_or((false, false, false));
        if has_month_order || (self.defs.tax_pillage_month_long != 0 && is_taxing) {
            self.record_month_order_overwrite(unit_id, "STUDY", in_turn_block);
        }
        if let Some(unit) = self.game.unit_mut(unit_id) {
            if self.defs.tax_pillage_month_long != 0 {
                unit.taxing = 0;
            }
            unit.orders.month_orders = Some(OrderData::Study(super::StudyOrder {
                skill,
                days: 0,
                level,
                ..super::StudyOrder::default()
            }));
        }
    }

    /// C++: `spells.cpp` `Game::ProcessCastOrder`.
    fn parse_cast_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: No skill given.");
            return;
        };
        let Some(skill_index) =
            crate::defs::skills::parse_skill(&self.tables.skills.skills, &token)
        else {
            self.record_unit_error(unit_id, "CAST: Invalid skill.");
            return;
        };
        let skill = self.tables.skills.skills[skill_index];
        if skill.flags & skill_flags::MAGIC == 0 {
            self.record_unit_error(unit_id, "CAST: That is not a magic skill.");
            return;
        }
        if skill.flags & skill_flags::CAST == 0 {
            self.record_unit_error(unit_id, "CAST: That skill cannot be CAST.");
            return;
        }

        if self.mode == OrderParseMode::Check {
            return;
        }
        self.parse_cast_spell(unit_id, skill_index, line);
    }

    /// C++: `spells.cpp` `Game::ProcessCastOrder`.
    fn parse_cast_spell(&mut self, unit_id: UnitId, skill_index: usize, line: &mut String) {
        let Some(spell_name) = self.tables.skills.skills[skill_index].name else {
            return;
        };
        let spell = skill_index as i32;

        if is_equal(spell_name, "mind reading") {
            let Some(id) = parse_unit_id(line) else {
                self.record_unit_error(unit_id, "CAST: No unit specified.");
                return;
            };
            let order = CastMindOrder {
                spell,
                level: 1,
                id: Some(id),
                ..CastMindOrder::default()
            };
            self.replace_cast_order(unit_id, CastOrderData::Mind(order));
        } else if is_equal(spell_name, "farsight")
            || is_equal(spell_name, "weather lore")
            || (is_equal(spell_name, "clear skies")
                && self.spell_range_flags(skill_index).is_some())
        {
            self.parse_region_spell(unit_id, skill_index, line, false);
        } else if is_equal(spell_name, "teleportation") {
            self.parse_region_spell(unit_id, skill_index, line, true);
        } else if is_equal(spell_name, "invisibility") {
            self.parse_invisibility(unit_id, skill_index, line);
        } else if is_equal(spell_name, "gate lore") {
            self.parse_gate_lore(unit_id, skill_index, line);
        } else if is_equal(spell_name, "portal lore") {
            self.parse_portal_lore(unit_id, skill_index, line);
        } else if is_equal(spell_name, "bird lore") {
            self.parse_bird_lore(unit_id, skill_index, line);
        } else if is_equal(spell_name, "create phantasmal beasts") {
            self.parse_phantasmal_beasts(unit_id, skill_index, line);
        } else if is_equal(spell_name, "create phantasmal undead") {
            self.parse_phantasmal_undead(unit_id, skill_index, line);
        } else if is_equal(spell_name, "create phantasmal demons") {
            self.parse_phantasmal_demons(unit_id, skill_index, line);
        } else if is_equal(spell_name, "transmutation") {
            self.parse_transmutation(unit_id, skill_index, line);
        } else if GENERIC_CAST_SPELL_NAMES
            .iter()
            .any(|name| is_equal(spell_name, name))
            || (is_equal(spell_name, "clear skies")
                && self.spell_range_flags(skill_index).is_none())
        {
            let order = CastOrder {
                spell,
                level: 1,
                ..CastOrder::default()
            };
            self.replace_cast_order(unit_id, CastOrderData::Cast(order));
        }
    }

    /// C++: `spells.cpp` `Game::ProcessInvisibility`.
    fn parse_invisibility(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Must specify units to render invisible.");
            return;
        };
        if !is_equal(&token, "units") {
            self.record_unit_error(unit_id, "CAST: Must specify units to render invisible.");
            return;
        }

        let existing =
            self.game
                .unit(unit_id)
                .and_then(|unit| match unit.orders.cast_orders.as_ref()? {
                    CastOrderData::Units(order)
                        if order.spell == spell as i32 && order.level == 1 =>
                    {
                        Some(order.clone())
                    }
                    _ => None,
                });
        let mut order = match existing {
            Some(order) => order,
            None => {
                self.clear_cast_orders(unit_id);
                CastUnitsOrder {
                    spell: spell as i32,
                    level: 1,
                    ..CastUnitsOrder::default()
                }
            }
        };
        while let Some(id) = parse_unit_id(line) {
            order.units.push(id);
        }
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.cast_orders = Some(CastOrderData::Units(order));
        }
    }

    /// C++: `spells.cpp` `Game::ProcessCastGateLore`.
    fn parse_gate_lore(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Missing argument.");
            return;
        };
        if is_equal(&token, "gate") {
            let Some(gate_token) = get_token(line) else {
                self.record_unit_error(unit_id, "CAST: Requires a target gate.");
                return;
            };
            let gate = value(&gate_token);
            if gate < 1 {
                self.record_unit_error(unit_id, "CAST: Requires a target gate.");
                return;
            }

            let existing = self.game.unit(unit_id).and_then(|unit| {
                unit.orders
                    .teleport_orders
                    .as_ref()
                    .filter(|order| order.spell == spell as i32 && order.gate == gate)
                    .cloned()
            });
            let mut order = match existing {
                Some(order) => order,
                None => {
                    self.clear_cast_orders(unit_id);
                    TeleportOrder::default()
                }
            };
            order.gate = gate;
            order.spell = spell as i32;
            order.level = 3;
            if get_token(line).is_some_and(|token| is_equal(&token, "units")) {
                while let Some(id) = parse_unit_id(line) {
                    order.units.push(id);
                }
            }
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.teleport_orders = Some(order);
            }
            return;
        }

        if is_equal(&token, "random") {
            let existing = self.game.unit(unit_id).and_then(|unit| {
                unit.orders
                    .teleport_orders
                    .as_ref()
                    .filter(|order| order.spell == spell as i32 && order.gate == -1)
                    .cloned()
            });
            let mut order = match existing {
                Some(order) => order,
                None => {
                    self.clear_cast_orders(unit_id);
                    TeleportOrder::default()
                }
            };
            order.gate = -1;
            order.spell = spell as i32;
            order.level = 1;

            if let Some(mut token) = get_token(line) {
                if is_equal(&token, "level") {
                    order.gate = -2;
                    order.level = 2;
                    token = match get_token(line) {
                        Some(token) => token,
                        None => {
                            if let Some(unit) = self.game.unit_mut(unit_id) {
                                unit.orders.teleport_orders = Some(order);
                            }
                            return;
                        }
                    };
                }
                if is_equal(&token, "units") {
                    while let Some(id) = parse_unit_id(line) {
                        order.units.push(id);
                    }
                }
            }
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.teleport_orders = Some(order);
            }
            return;
        }

        if is_equal(&token, "detect") {
            self.replace_cast_order(
                unit_id,
                CastOrderData::Cast(CastOrder {
                    spell: spell as i32,
                    level: 2,
                    ..CastOrder::default()
                }),
            );
            return;
        }

        self.record_unit_error(unit_id, "CAST: Invalid argument.");
    }

    /// C++: `spells.cpp` `Game::ProcessCastPortalLore`.
    fn parse_portal_lore(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(gate_token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Requires a target mage.");
            return;
        };
        let gate = value(&gate_token);
        let Some(units_token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: No units to teleport.");
            return;
        };
        if !is_equal(&units_token, "units") {
            self.record_unit_error(unit_id, "CAST: No units to teleport.");
            return;
        }

        let existing = self.game.unit(unit_id).and_then(|unit| {
            unit.orders
                .teleport_orders
                .as_ref()
                .filter(|order| order.spell == spell as i32)
                .cloned()
        });
        let mut order = match existing {
            Some(order) => order,
            None => {
                self.clear_cast_orders(unit_id);
                TeleportOrder::default()
            }
        };
        order.gate = gate;
        order.spell = spell as i32;
        order.level = 1;
        while let Some(id) = parse_unit_id(line) {
            order.units.push(id);
        }
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.teleport_orders = Some(order);
        }
    }

    /// C++: `spells.cpp` `Game::ProcessBirdLore`.
    fn parse_bird_lore(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Missing arguments.");
            return;
        };
        let mut order = CastIntOrder {
            spell: spell as i32,
            ..CastIntOrder::default()
        };
        if is_equal(&token, "eagle") {
            order.level = 3;
        } else if is_equal(&token, "direction") {
            let Some(token) = get_token(line) else {
                self.record_unit_error(unit_id, "CAST: Missing arguments.");
                return;
            };
            let Some(direction) = parse_dir(&token) else {
                self.record_unit_error(unit_id, "CAST: Invalid direction.");
                return;
            };
            if direction > crate::defs::strings::DIRECTION_NAMES.len() as i32 {
                self.record_unit_error(unit_id, "CAST: Invalid direction.");
                return;
            }
            order.level = 1;
            order.target = direction;
        } else {
            self.record_unit_error(unit_id, "CAST: Invalid arguments.");
            return;
        }
        self.replace_cast_order(unit_id, CastOrderData::Int(order));
    }

    /// C++: `spells.cpp` `Game::ProcessPhanBeasts`.
    fn parse_phantasmal_beasts(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Must specify which illusion to summon.");
            return;
        };
        let level = if is_equal(&token, "wolf") || is_equal(&token, "wolves") {
            1
        } else if is_equal(&token, "eagle") || is_equal(&token, "eagles") {
            2
        } else if is_equal(&token, "dragon") {
            3
        } else {
            self.record_unit_error(unit_id, "CAST: Must specify which illusion to summon.");
            return;
        };
        let target = get_token(line).map_or(1, |token| value(&token));
        self.replace_cast_order(
            unit_id,
            CastOrderData::Int(CastIntOrder {
                spell: spell as i32,
                level,
                target,
                ..CastIntOrder::default()
            }),
        );
    }

    /// C++: `spells.cpp` `Game::ProcessPhanUndead`.
    fn parse_phantasmal_undead(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Must specify which illusion to summon.");
            return;
        };
        let level = if is_equal(&token, "skeleton") || is_equal(&token, "skeletons") {
            1
        } else if is_equal(&token, "undead") {
            2
        } else if is_equal(&token, "lich") || is_equal(&token, "liches") {
            3
        } else {
            self.record_unit_error(unit_id, "CAST: Must specify which illusion to summon.");
            return;
        };
        let target = get_token(line).map_or(1, |token| value(&token));
        self.replace_cast_order(
            unit_id,
            CastOrderData::Int(CastIntOrder {
                spell: spell as i32,
                level,
                target,
                ..CastIntOrder::default()
            }),
        );
    }

    /// C++: `spells.cpp` `Game::ProcessPhanDemons`.
    fn parse_phantasmal_demons(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: Illusion to summon must be given.");
            return;
        };
        let level = if is_equal(&token, "imp") || is_equal(&token, "imps") {
            1
        } else if is_equal(&token, "demon") || is_equal(&token, "demons") {
            2
        } else if is_equal(&token, "balrog") || is_equal(&token, "balrogs") {
            3
        } else {
            self.record_unit_error(unit_id, "CAST: Can't summon that illusion.");
            return;
        };
        let target = get_token(line).map_or(1, |token| value(&token));
        self.replace_cast_order(
            unit_id,
            CastOrderData::Int(CastIntOrder {
                spell: spell as i32,
                level,
                target,
                ..CastIntOrder::default()
            }),
        );
    }

    /// C++: `spells.cpp` `Game::ProcessTransmutation`.
    fn parse_transmutation(&mut self, unit_id: UnitId, spell: usize, line: &mut String) {
        let Some(mut token) = get_token(line) else {
            self.record_unit_error(unit_id, "CAST: You must specify what you wish to create.");
            return;
        };
        let mut number = -1;
        if value(&token) > 0 {
            number = value(&token);
            let Some(item_token) = get_token(line) else {
                self.record_unit_error(unit_id, "CAST: You must specify what you wish to create.");
                return;
            };
            token = item_token;
        }
        let Some(item) = self.tables.item_families.parse_enabled_item(&token) else {
            self.record_unit_error(unit_id, "CAST: You must specify what you wish to create.");
            return;
        };
        let item_name = self.tables.item_families.items[item]
            .name
            .as_deref()
            .unwrap_or("");
        let level = if is_equal(item_name, "mithril") || is_equal(item_name, "rootstone") {
            1
        } else if is_equal(item_name, "ironwood") {
            2
        } else if is_equal(item_name, "floater hide") {
            3
        } else if is_equal(item_name, "yew") {
            4
        } else if is_equal(item_name, "winged horse") || is_equal(item_name, "admantium") {
            5
        } else {
            self.record_unit_error(unit_id, "CAST: Can't create that by transmutation.");
            return;
        };
        self.replace_cast_order(
            unit_id,
            CastOrderData::Transmute(CastTransmuteOrder {
                spell: spell as i32,
                level,
                item: item as i32,
                number,
                ..CastTransmuteOrder::default()
            }),
        );
    }

    /// C++: `spells.cpp` `Game::ProcessRegionSpell`.
    fn parse_region_spell(
        &mut self,
        unit_id: UnitId,
        spell: usize,
        line: &mut String,
        teleport: bool,
    ) {
        let range_flags = self.spell_range_flags(spell).unwrap_or(0);
        let mut coordinates = self.caster_coordinates(unit_id);
        if let Some(token) = get_token(line) {
            if is_equal(&token, "region") {
                let Some(x) = get_token(line) else {
                    self.record_unit_error(unit_id, "CAST: Region X coordinate not specified.");
                    return;
                };
                coordinates.0 = value(&x);
                let Some(y) = get_token(line) else {
                    self.record_unit_error(unit_id, "CAST: Region Y coordinate not specified.");
                    return;
                };
                coordinates.1 = value(&y);

                if range_flags & crate::defs::skills::range_flags::CROSS_LEVELS != 0 {
                    if let Some(z) = get_token(line) {
                        coordinates.2 = value(&z);
                        let z_limit = self.defs.underworld_levels
                            + self.defs.underdeep_levels
                            + self.defs.abyss_level
                            + 2;
                        if coordinates.2 < 0 || coordinates.2 >= z_limit {
                            self.record_unit_error(
                                unit_id,
                                "CAST: Invalid Z coordinate specified.",
                            );
                            return;
                        }
                    }
                }
            }
        }

        self.clear_cast_orders(unit_id);
        if teleport {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.teleport_orders = Some(TeleportOrder {
                    spell: spell as i32,
                    level: 1,
                    xloc: coordinates.0,
                    yloc: coordinates.1,
                    zloc: coordinates.2,
                    ..TeleportOrder::default()
                });
            }
        } else if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.cast_orders = Some(CastOrderData::Region(CastRegionOrder {
                spell: spell as i32,
                level: 1,
                xloc: coordinates.0,
                yloc: coordinates.1,
                zloc: coordinates.2,
                ..CastRegionOrder::default()
            }));
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessTeachOrder`.
    fn parse_teach_order(&mut self, unit_id: UnitId, line: &mut String) {
        let mut order = self
            .game
            .unit(unit_id)
            .and_then(|unit| unit.orders.month_orders.as_ref())
            .and_then(|order| match order {
                OrderData::Teach(order) => Some(order.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let mut students = 0;
        while let Some(target) = super::parse_unit_id(line) {
            if target.unitnum == -1 {
                break;
            }
            students += 1;
            order.targets.push(target);
        }
        if students == 0 {
            self.record_unit_error(unit_id, "TEACH: No students given.");
            return;
        }

        let (has_other_month_order, in_turn_block, is_taxing) = self
            .game
            .unit(unit_id)
            .map(|unit| {
                (
                    unit.orders
                        .month_orders
                        .as_ref()
                        .is_some_and(|order| !matches!(order, OrderData::Teach(_))),
                    unit.orders.in_turn_block != 0,
                    matches!(unit.taxing, TAX_TAX | TAX_PILLAGE),
                )
            })
            .unwrap_or((false, false, false));
        if has_other_month_order || (self.defs.tax_pillage_month_long != 0 && is_taxing) {
            self.record_month_order_overwrite(unit_id, "TEACH", in_turn_block);
        }

        if let Some(unit) = self.game.unit_mut(unit_id) {
            if self.defs.tax_pillage_month_long != 0 {
                unit.taxing = 0;
            }
            unit.orders.month_orders = Some(OrderData::Teach(order));
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessForgetOrder`.
    fn parse_forget_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(skill) = get_token(line) else {
            self.record_unit_error(unit_id, "FORGET: No skill given.");
            return;
        };
        let Some(skill) =
            crate::defs::skills::parse_skill(&self.ruleset_tables().skills.skills, &skill)
        else {
            self.record_unit_error(unit_id, "FORGET: Invalid skill.");
            return;
        };
        let skill = i32::try_from(skill).expect("skill index fits in an order field");
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.forget_orders.push(super::ForgetOrder {
                    skill,
                    ..super::ForgetOrder::default()
                });
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessAttackOrder`.
    fn parse_attack_order(&mut self, unit_id: UnitId, line: &mut String) {
        let targets = parse_unit_targets(line);
        if self.mode == OrderParseMode::Turn && !targets.is_empty() {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders
                    .attack_orders
                    .get_or_insert_with(super::AttackOrder::default)
                    .targets
                    .extend(targets);
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessEvictOrder`.
    fn parse_evict_order(&mut self, unit_id: UnitId, line: &mut String) {
        let targets = parse_unit_targets(line);
        if self.mode == OrderParseMode::Turn && !targets.is_empty() {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders
                    .evict_orders
                    .get_or_insert_with(super::EvictOrder::default)
                    .targets
                    .extend(targets);
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessStealOrder`.
    fn parse_steal_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(target) = super::parse_unit_id(line).filter(|target| target.unitnum != -1) else {
            self.record_unit_error(unit_id, "STEAL: No target given.");
            return;
        };
        let Some(item) = get_token(line) else {
            self.record_unit_error(unit_id, "STEAL: No item given.");
            return;
        };
        let Some(item) = self
            .ruleset_tables()
            .item_families
            .parse_enabled_item(&item)
        else {
            if self.mode == OrderParseMode::Turn {
                self.record_unit_error(unit_id, "STEAL: Bad item given.");
            }
            return;
        };
        let item_def = &self.ruleset_tables().item_families.items[item];
        if item_def.item_type & (ItemTypeFlags::MAN | ItemTypeFlags::MONSTER) != 0 {
            if self.mode == OrderParseMode::Turn {
                self.record_unit_error(unit_id, "STEAL: Can't steal that.");
            }
            return;
        }
        let item = i32::try_from(item).expect("item index fits in an order field");
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.steal_orders = Some(super::StealOrderData::Steal(super::StealOrder {
                    target: Some(target),
                    item,
                    ..super::StealOrder::default()
                }));
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessAssassinateOrder`.
    fn parse_assassinate_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(target) = super::parse_unit_id(line).filter(|target| target.unitnum != -1) else {
            self.record_unit_error(unit_id, "ASSASSINATE: No target given.");
            return;
        };
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.steal_orders = Some(super::StealOrderData::Assassinate(
                    super::AssassinateOrder {
                        target: Some(target),
                        ..super::AssassinateOrder::default()
                    },
                ));
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessPromoteOrder`.
    fn parse_promote_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(target) = super::parse_unit_id(line).filter(|target| target.unitnum != -1) else {
            self.record_unit_error(unit_id, "PROMOTE: No target given.");
            return;
        };
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.promote = Some(target);
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessDestroyOrder`.
    fn parse_destroy_order(&mut self, unit_id: UnitId) {
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.destroy = 1;
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessEnterOrder`.
    fn parse_enter_order(&mut self, unit_id: UnitId, line: &mut String) {
        let object = get_token(line).map_or(0, |token| value(&token));
        if object == 0 {
            self.record_unit_error(unit_id, "ENTER: No object specified.");
            return;
        }
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.enter = object;
            }
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessLeaveOrder`.
    fn parse_leave_order(&mut self, unit_id: UnitId) {
        if self.mode == OrderParseMode::Turn {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                if unit.orders.enter == 0 {
                    unit.orders.enter = -1;
                }
            }
        }
    }

    fn record_month_order_overwrite(&mut self, unit_id: UnitId, verb: &str, delayed: bool) {
        let delayed = if delayed { "DELAYED " } else { "" };
        self.record_unit_error(
            unit_id,
            &format!("{verb}: Overwriting previous {delayed}month-long order."),
        );
    }

    fn caster_coordinates(&self, unit_id: UnitId) -> (i32, i32, i32) {
        let region_id = self
            .game
            .unit(unit_id)
            .and_then(|unit| unit.object)
            .and_then(|object_id| self.game.object(object_id))
            .and_then(|object| object.region)
            .expect("a selected caster belongs to a region");
        let region = self
            .game
            .region(region_id)
            .expect("a selected caster's region exists");
        (region.xloc, region.yloc, region.zloc)
    }

    fn spell_range_flags(&self, spell: usize) -> Option<i32> {
        let range = self.tables.skills.skills.get(spell)?.range?;
        self.tables
            .skills
            .ranges
            .get(range)
            .map(|range| range.flags)
    }

    fn clear_cast_orders(&mut self, unit_id: UnitId) {
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.cast_orders = None;
            unit.orders.teleport_orders = None;
        }
    }

    fn replace_cast_order(&mut self, unit_id: UnitId, order: CastOrderData) {
        self.clear_cast_orders(unit_id);
        if let Some(unit) = self.game.unit_mut(unit_id) {
            unit.orders.cast_orders = Some(order);
        }
        match verb {
            Some(OrderVerb::Buy) => self.parse_buy_sell_order(unit_id, line, true),
            Some(OrderVerb::Sell) => self.parse_buy_sell_order(unit_id, line, false),
            Some(OrderVerb::Produce) => self.parse_produce_order(unit_id, line),
            Some(OrderVerb::Work) => self.parse_work_order(unit_id),
            Some(OrderVerb::Entertain) => self.parse_entertain_order(unit_id),
            Some(OrderVerb::Build) => self.parse_build_order(unit_id, line),
            Some(OrderVerb::Give) => self.parse_give_order(unit_id, line, false),
            Some(OrderVerb::Take) => self.parse_give_order(unit_id, line, true),
            Some(OrderVerb::Withdraw) => self.parse_withdraw_order(unit_id, line),
            Some(OrderVerb::Exchange) => self.parse_exchange_order(unit_id, line),
            Some(OrderVerb::Transport) => self.parse_transport_order(unit_id, line, false),
            Some(OrderVerb::Distribute) => self.parse_transport_order(unit_id, line, true),
            Some(OrderVerb::Join) => self.parse_join_order(unit_id, line),
            Some(OrderVerb::Claim) => self.parse_claim_order(unit_id, line),
            Some(OrderVerb::Tax) => self.parse_tax_pillage_order(unit_id, false),
            Some(OrderVerb::Pillage) => self.parse_tax_pillage_order(unit_id, true),
            _ => {}
        }
    }

    /// C++: `parseorders.cpp` `Game::ProcessAdvanceOrder`, `ProcessMoveOrder`, and `ProcessSailOrder`.
    fn parse_movement_order(&mut self, unit_id: UnitId, verb: OrderVerb, line: &mut String) {
        let (has_other_month_order, in_turn_block, is_taxing) = self
            .game
            .unit(unit_id)
            .map(|unit| {
                let has_other_month_order =
                    unit.orders
                        .month_orders
                        .as_ref()
                        .is_some_and(|order| match order {
                            OrderData::Move(order) => order.order.kind != verb,
                            OrderData::Sail(order) => order.order.kind != verb,
                            _ => true,
                        });
                (
                    has_other_month_order,
                    unit.orders.in_turn_block != 0,
                    matches!(unit.taxing, TAX_TAX | TAX_PILLAGE),
                )
            })
            .unwrap_or((false, false, false));
        let overwrite =
            has_other_month_order || (self.defs.tax_pillage_month_long != 0 && is_taxing);

        if overwrite {
            let delayed = if in_turn_block { "DELAYED " } else { "" };
            let message = match verb {
                OrderVerb::Advance => {
                    format!("ADVANCE: Overwriting previous {delayed}month-long orders")
                }
                OrderVerb::Move => {
                    format!("MOVE: Overwriting previous {delayed}month-long order.")
                }
                OrderVerb::Sail => {
                    format!("SAIL: Overwriting previous {delayed}month-long order.")
                }
                _ => return,
            };
            self.record_unit_error(unit_id, &message);
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.orders.month_orders = None;
            }
        }

        let mut order_data = self
            .game
            .unit_mut(unit_id)
            .and_then(|unit| unit.orders.month_orders.take());
        if self.defs.tax_pillage_month_long != 0 {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.taxing = 0;
            }
        }

        match verb {
            OrderVerb::Move | OrderVerb::Advance => {
                let mut order = match order_data.take() {
                    Some(OrderData::Move(order)) if order.order.kind == verb => order,
                    _ => MoveOrder {
                        order: super::Order::new(verb),
                        advancing: i32::from(verb == OrderVerb::Advance),
                        ..MoveOrder::default()
                    },
                };
                order.advancing = i32::from(verb == OrderVerb::Advance);

                while let Some(token) = get_token(line) {
                    let Some(direction) = parse_dir(&token) else {
                        self.record_unit_error(
                            unit_id,
                            if verb == OrderVerb::Advance {
                                "ADVANCE: Warning, bad direction."
                            } else {
                                "MOVE: Warning, bad direction."
                            },
                        );
                        break;
                    };
                    if self.mode == OrderParseMode::Turn {
                        order.dirs.push(MoveDir { dir: direction });
                    }
                }
                if let Some(unit) = self.game.unit_mut(unit_id) {
                    unit.orders.month_orders = Some(OrderData::Move(order));
                }
            }
            OrderVerb::Sail => {
                let mut order = match order_data.take() {
                    Some(OrderData::Sail(order)) if order.order.kind == OrderVerb::Sail => order,
                    _ => SailOrder::default(),
                };

                while let Some(token) = get_token(line) {
                    let Some(direction) = parse_dir(&token) else {
                        self.record_unit_error(unit_id, "SAIL: Warning, bad direction.");
                        break;
                    };
                    if direction >= crate::defs::strings::DIRECTION_NAMES.len() as i32
                        && direction != super::MOVE_PAUSE
                    {
                        self.record_unit_error(unit_id, "SAIL: Warning, bad direction.");
                        break;
                    }
                    if self.mode == OrderParseMode::Turn {
                        order.dirs.push(MoveDir { dir: direction });
                    }
                }
                if let Some(unit) = self.game.unit_mut(unit_id) {
                    unit.orders.month_orders = Some(OrderData::Sail(order));
                }
            }
            _ => {}
        }
    }

    /// C++: `parseorders.cpp` `ProcessBuyOrder` and `ProcessSellOrder`.
    fn parse_buy_sell_order(&mut self, unit_id: UnitId, line: &mut String, is_buy: bool) {
        let (verb, side) = if is_buy {
            ("BUY", "buy")
        } else {
            ("SELL", "sell")
        };
        let Some(number) = get_token(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: Number to {side} not given."));
            return;
        };
        let number = if is_equal(&number, "all") {
            -1
        } else {
            value(&number)
        };
        if number == 0 {
            self.record_unit_error(unit_id, &format!("{verb}: Number to {side} not given."));
            return;
        }
        let Some(item_token) = get_token(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: Item not given."));
            return;
        };

        let ruleset =
            <dyn Ruleset>::from_name(self.defs.ruleset_name).expect("parser ruleset is supported");
        let tables = ruleset.tables();
        let item = tables
            .item_families
            .parse_giveable_item(&item_token)
            .map(|item| item as i32)
            .or_else(|| {
                if is_buy
                    && (is_equal(&item_token, "peasant")
                        || is_equal(&item_token, "peasants")
                        || is_equal(&item_token, "peas"))
                {
                    self.game
                        .unit(unit_id)
                        .and_then(|unit| unit.object)
                        .and_then(|object_id| self.game.object(object_id))
                        .and_then(|object| object.region)
                        .and_then(|region_id| self.game.region(region_id))
                        .map(|region| region.race)
                } else {
                    None
                }
            })
            .unwrap_or(-1);

        if self.mode == OrderParseMode::Turn {
            let unit = self.game.unit_mut(unit_id).expect("selected unit exists");
            if is_buy {
                unit.orders.buy_orders.push(BuyOrder {
                    item,
                    num: number,
                    ..BuyOrder::default()
                });
            } else {
                unit.orders.sell_orders.push(SellOrder {
                    item,
                    num: number,
                    ..SellOrder::default()
                });
            }
        }
    }

    /// C++: `parseorders.cpp` `ProcessProduceOrder`.
    fn parse_produce_order(&mut self, unit_id: UnitId, line: &mut String) {
        let mut target = 0;
        let mut token = get_token(line);
        if let Some(quantity) = token.as_deref().map(value).filter(|quantity| *quantity > 0) {
            target = quantity;
            token = get_token(line);
        }
        let Some(token) = token else {
            self.record_unit_error(unit_id, "PRODUCE: No item given.");
            return;
        };

        let ruleset =
            <dyn Ruleset>::from_name(self.defs.ruleset_name).expect("parser ruleset is supported");
        let tables = ruleset.tables();
        let Some(item_index) = tables.item_families.parse_enabled_item(&token) else {
            self.record_unit_error(unit_id, "PRODUCE: Invalid item.");
            return;
        };
        let item = &tables.item_families.items[item_index];
        if item.item_type == ItemTypeFlags::SHIP {
            self.record_unit_error(unit_id, "PRODUCE: Use BUILD for ships.");
            return;
        }
        let skill = item
            .production_skill
            .as_deref()
            .and_then(|skill| crate::defs::skills::lookup_skill(&tables.skills.skills, skill))
            .map_or(-1, |skill| skill as i32);
        self.set_month_order(
            unit_id,
            "PRODUCE",
            OrderData::Produce(ProduceOrder {
                item: item_index as i32,
                skill,
                target,
                ..ProduceOrder::default()
            }),
            true,
        );
    }

    /// C++: `parseorders.cpp` `ProcessWorkOrder`.
    fn parse_work_order(&mut self, unit_id: UnitId) {
        self.set_month_order(
            unit_id,
            "WORK",
            OrderData::Produce(ProduceOrder {
                item: ItemId::Silver.index() as i32,
                skill: -1,
                target: 0,
                ..ProduceOrder::default()
            }),
            true,
        );
    }

    /// C++: `parseorders.cpp` `ProcessEntertainOrder`.
    fn parse_entertain_order(&mut self, unit_id: UnitId) {
        let ruleset =
            <dyn Ruleset>::from_name(self.defs.ruleset_name).expect("parser ruleset is supported");
        let tables = ruleset.tables();
        let skill = tables
            .skills
            .skills
            .iter()
            .position(|skill| {
                skill
                    .name
                    .is_some_and(|name| is_equal(name, "entertainment"))
            })
            .expect("entertainment skill is defined") as i32;
        self.set_month_order(
            unit_id,
            "ENTERTAIN",
            OrderData::Produce(ProduceOrder {
                item: ItemId::Silver.index() as i32,
                skill,
                target: 0,
                ..ProduceOrder::default()
            }),
            false,
        );
    }

    /// C++: `parseorders.cpp` `ProcessBuildOrder`.
    fn parse_build_order(&mut self, unit_id: UnitId, line: &mut String) {
        self.game
            .unit_mut(unit_id)
            .expect("selected unit exists")
            .orders
            .build = 0;
        let mut order = BuildOrder::default();
        let mut need_to_complete = 0;
        let token = get_token(line);

        if let Some(token) = token {
            if is_equal(&token, "help") {
                if self.mode == OrderParseMode::Turn {
                    let Some(target) =
                        super::parse_unit_id(line).filter(|target| target.unitnum != -1)
                    else {
                        self.record_unit_error(unit_id, "BUILD: Non-existent unit to help.");
                        return;
                    };
                    order.target = Some(target);
                }
            } else {
                let ruleset = <dyn Ruleset>::from_name(self.defs.ruleset_name)
                    .expect("parser ruleset is supported");
                let tables = ruleset.tables();
                let Some(target) = crate::defs::parse_object(
                    &token,
                    1,
                    &tables.objects,
                    &tables.item_families.items,
                ) else {
                    self.record_unit_error(unit_id, "BUILD: Not a valid object name.");
                    return;
                };

                if self.mode == OrderParseMode::Turn {
                    let region_id = self
                        .game
                        .unit(unit_id)
                        .and_then(|unit| unit.object)
                        .and_then(|object_id| self.game.object(object_id))
                        .and_then(|object| object.region);
                    let is_ocean = region_id
                        .and_then(|region_id| self.game.region(region_id))
                        .and_then(|region| region.terrain_type)
                        .and_then(|terrain| tables.terrains.get(terrain))
                        .is_some_and(|terrain| {
                            terrain.similar_type == crate::defs::terrains::R_OCEAN as i32
                        });
                    if is_ocean {
                        self.record_unit_error(unit_id, "BUILD: Can't build in an ocean.");
                        return;
                    }

                    match target {
                        BuildTarget::Object(object_id) => {
                            let object = &tables.objects[object_id];
                            if object.flags & crate::defs::objects::object_flags::CANENTER == 0
                                || object
                                    .skill
                                    .and_then(|skill| {
                                        crate::defs::skills::lookup_skill(
                                            &tables.skills.skills,
                                            skill,
                                        )
                                    })
                                    .is_none()
                            {
                                self.record_unit_error(unit_id, "BUILD: Can't build that.");
                                return;
                            }
                            order.new_building = object_id as i32;
                        }
                        BuildTarget::ShipItem(item_index) => {
                            let item = &tables.item_families.items[item_index];
                            let coastal_or_lakeside = region_id.is_some_and(|region_id| {
                                is_coastal_or_lakeside(self.game, region_id, &tables)
                            });
                            if !coastal_or_lakeside && item.fly <= 0 {
                                self.record_unit_error(
                                    unit_id,
                                    "BUILD: Can't build ship in non-coastal or lakeside region.",
                                );
                                return;
                            }

                            let item_id = ItemId::from_index(item_index)
                                .expect("ship item index is in the item table");
                            let item_count = self
                                .game
                                .unit(unit_id)
                                .expect("selected unit exists")
                                .items
                                .get_num(item_id);
                            self.game
                                .unit_mut(unit_id)
                                .expect("selected unit exists")
                                .orders
                                .build = -(item_index as i32);
                            need_to_complete = if item_count > 0 {
                                item_count
                            } else {
                                item.production_months
                            };
                        }
                    }
                }
            }
        } else if self.mode == OrderParseMode::Turn {
            let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
                .expect("parser ruleset is supported")
                .tables();
            let item = self
                .game
                .unit(unit_id)
                .expect("selected unit exists")
                .items
                .entries()
                .iter()
                .find(|entry| {
                    tables
                        .item_families
                        .items
                        .get(entry.item_type.index())
                        .is_some_and(|item| {
                            item.item_type & ItemTypeFlags::SHIP != 0
                                && item.flags & ItemFlags::DISABLED == 0
                        })
                })
                .map(|entry| (entry.item_type.index(), entry.num));
            if let Some((item_index, item_count)) = item {
                self.game
                    .unit_mut(unit_id)
                    .expect("selected unit exists")
                    .orders
                    .build = -(item_index as i32);
                need_to_complete = item_count;
            }
        }

        order.needtocomplete = need_to_complete;
        self.set_month_order(unit_id, "BUILD", OrderData::Build(order), true);
    }

    /// C++: `parseorders.cpp` `ProcessGiveOrder`.
    fn parse_give_order(&mut self, unit_id: UnitId, line: &mut String, is_take: bool) {
        let verb = if is_take { "TAKE" } else { "GIVE" };
        if is_take
            && !get_token(line)
                .as_deref()
                .is_some_and(|token| is_equal(token, "from"))
        {
            self.record_unit_error(unit_id, "TAKE: Missing FROM.");
            return;
        }

        let Some(target) = super::parse_unit_id(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: Invalid target."));
            return;
        };
        if is_take && target.unitnum == -1 {
            self.record_unit_error(unit_id, "TAKE: Invalid target.");
            return;
        }

        let Some(amount_token) = get_token(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: No amount given."));
            return;
        };
        let amount = if !is_take && is_equal(&amount_token, "unit") {
            -1
        } else if is_equal(&amount_token, "all") {
            -2
        } else {
            let amount = value(&amount_token);
            if amount < 1 {
                self.record_unit_error(unit_id, &format!("{verb}: Illegal amount given."));
                return;
            }
            amount
        };

        let mut item = -1;
        let mut unfinished = 0;
        let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
            .expect("parser ruleset is supported")
            .tables();
        if amount != -1 {
            let Some(mut item_token) = get_token(line) else {
                self.record_unit_error(unit_id, &format!("{verb}: No item given."));
                return;
            };
            if is_equal(&item_token, "unfinished") {
                unfinished = 1;
                let Some(next_token) = get_token(line) else {
                    self.record_unit_error(unit_id, &format!("{verb}: No item given."));
                    return;
                };
                item_token = next_token;
            }

            item = if target.unitnum == -1 {
                tables
                    .item_families
                    .parse_enabled_item(&item_token)
                    .map(|index| index as i32)
                    .unwrap_or(-1)
            } else {
                tables
                    .item_families
                    .parse_giveable_item(&item_token)
                    .map(|index| index as i32)
                    .unwrap_or(-1)
            };

            if amount == -2 {
                if let Some(item_class) = give_all_item_class(&item_token) {
                    item = item_class;
                } else if item == -1 {
                    self.record_unit_error(
                        unit_id,
                        &format!("{verb}: Invalid item or item class."),
                    );
                    return;
                }
            } else if item == -1 {
                self.record_unit_error(unit_id, &format!("{verb}: Invalid item."));
                return;
            }

            if unfinished != 0
                && item != -(ItemTypeFlags::SHIP)
                && item != -(crate::defs::NITEMS as i32)
                && !(item >= 0
                    && tables
                        .item_families
                        .items
                        .get(item as usize)
                        .is_some_and(|definition| definition.item_type & ItemTypeFlags::SHIP != 0))
            {
                self.record_unit_error(
                    unit_id,
                    &format!("{verb}: That item does not have an unfinished version."),
                );
                return;
            }
        }

        let mut except = 0;
        if get_token(line).is_some_and(|token| is_equal(&token, "except")) {
            if amount != -2 {
                self.record_unit_error(unit_id, &format!("{verb}: EXCEPT only valid with ALL"));
                return;
            }
            if item < 0 {
                self.record_unit_error(
                    unit_id,
                    &format!("{verb}: EXCEPT only valid with specific items."),
                );
                return;
            }
            let Some(value_token) = get_token(line) else {
                self.record_unit_error(unit_id, &format!("{verb}: EXCEPT requires a value."));
                return;
            };
            except = value(&value_token);
            if except <= 0 {
                self.record_unit_error(unit_id, &format!("{verb}: Invalid EXCEPT value."));
                return;
            }
        }

        if self.mode == OrderParseMode::Turn {
            let mut order = if is_take {
                GiveOrder::take()
            } else {
                GiveOrder::default()
            };
            order.item = item;
            order.amount = amount;
            order.except = except;
            order.unfinished = unfinished;
            order.target = Some(target);
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .give_orders
                .push(order);
        }
    }

    /// C++: `parseorders.cpp` `ProcessWithdrawOrder`.
    fn parse_withdraw_order(&mut self, unit_id: UnitId, line: &mut String) {
        if self.defs.allow_withdraw == 0 {
            self.record_unit_error(unit_id, "WITHDRAW is not a valid order.");
            return;
        }

        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "WITHDRAW: No amount given.");
            return;
        };
        let parsed_amount = value(&token);
        let (amount, item_token) = if parsed_amount < 1 {
            (1, token)
        } else {
            let Some(item) = get_token(line) else {
                self.record_unit_error(unit_id, "WITHDRAW: No item given.");
                return;
            };
            (parsed_amount, item)
        };
        let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
            .expect("parser ruleset is supported")
            .tables();
        let Some(item) = tables.item_families.parse_giveable_item(&item_token) else {
            self.record_unit_error(unit_id, "WITHDRAW: Invalid item.");
            return;
        };
        let definition = &tables.item_families.items[item];
        if definition.flags & ItemFlags::DISABLED != 0
            || definition.item_type & ItemTypeFlags::NORMAL == 0
            || item == ItemId::Silver.index()
        {
            self.record_unit_error(unit_id, "WITHDRAW: Invalid item.");
            return;
        }

        if self.mode == OrderParseMode::Turn {
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .withdraw_orders
                .push(WithdrawOrder {
                    item: item as i32,
                    amount,
                    ..WithdrawOrder::default()
                });
        }
    }

    /// C++: `parseorders.cpp` `ProcessExchangeOrder`.
    fn parse_exchange_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(target) = super::parse_unit_id(line) else {
            self.record_unit_error(unit_id, "EXCHANGE: Invalid target.");
            return;
        };
        let Some(amount_token) = get_token(line) else {
            self.record_unit_error(unit_id, "EXCHANGE: No amount given.");
            return;
        };
        let give_amount = value(&amount_token);
        if give_amount < 0 {
            self.record_unit_error(unit_id, "EXCHANGE: Illegal amount given.");
            return;
        }
        let Some(item_token) = get_token(line) else {
            self.record_unit_error(unit_id, "EXCHANGE: No item given.");
            return;
        };
        let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
            .expect("parser ruleset is supported")
            .tables();
        let Some(give_item) = tables.item_families.parse_giveable_item(&item_token) else {
            self.record_unit_error(unit_id, "EXCHANGE: Invalid item.");
            return;
        };
        if tables.item_families.items[give_item].item_type & ItemTypeFlags::SHIP != 0 {
            self.record_unit_error(unit_id, "EXCHANGE: Can't exchange ships.");
            return;
        }

        let Some(amount_token) = get_token(line) else {
            self.record_unit_error(unit_id, "EXCHANGE: No amount expected.");
            return;
        };
        let expect_amount = value(&amount_token);
        if expect_amount < 0 {
            self.record_unit_error(unit_id, "EXCHANGE: Illegal amount given.");
            return;
        }
        let Some(item_token) = get_token(line) else {
            self.record_unit_error(unit_id, "EXCHANGE: No item expected.");
            return;
        };
        let Some(expect_item) = tables.item_families.parse_giveable_item(&item_token) else {
            self.record_unit_error(unit_id, "EXCHANGE: Invalid item.");
            return;
        };
        if tables.item_families.items[expect_item].item_type & ItemTypeFlags::SHIP != 0 {
            self.record_unit_error(unit_id, "EXCHANGE: Can't exchange ships.");
            return;
        }

        if self.mode == OrderParseMode::Turn {
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .exchange_orders
                .push(ExchangeOrder {
                    give_item: give_item as i32,
                    give_amount,
                    expect_item: expect_item as i32,
                    expect_amount,
                    target: Some(target),
                    ..ExchangeOrder::default()
                });
        }
    }

    /// C++: `parseorders.cpp` `ProcessTransportOrder` and `ProcessDistributeOrder`.
    fn parse_transport_order(&mut self, unit_id: UnitId, line: &mut String, distribute: bool) {
        let verb = if distribute {
            "DISTRIBUTE"
        } else {
            "TRANSPORT"
        };
        let Some(target) = super::parse_unit_id(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: Invalid target."));
            return;
        };
        let Some(amount_token) = get_token(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: No amount given."));
            return;
        };
        let amount = if is_equal(&amount_token, "all") {
            -1
        } else {
            value(&amount_token)
        };
        let Some(item_token) = get_token(line) else {
            self.record_unit_error(unit_id, &format!("{verb}: No item given."));
            return;
        };
        let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
            .expect("parser ruleset is supported")
            .tables();
        let Some(item) = tables.item_families.parse_transportable_item(&item_token) else {
            let message = if distribute {
                format!("{verb}: Invalid item {item_token}.")
            } else {
                format!("{verb}: Invalid item.")
            };
            self.record_unit_error(unit_id, &message);
            return;
        };

        let mut except = 0;
        if get_token(line).is_some_and(|token| is_equal(&token, "except")) {
            let Some(value_token) = get_token(line) else {
                self.record_unit_error(unit_id, &format!("{verb}: EXCEPT requires a value."));
                return;
            };
            except = value(&value_token);
            if except <= 0 {
                self.record_unit_error(unit_id, &format!("{verb}: Invalid except value."));
                return;
            }
        }

        if self.mode == OrderParseMode::Turn {
            let mut order = if distribute {
                TransportOrder::distribute()
            } else {
                TransportOrder::default()
            };
            order.item = item as i32;
            order.amount = amount;
            order.except = except;
            order.target = Some(target);
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .transport_orders
                .push(order);
        }
    }

    /// C++: `parseorders.cpp` `ProcessJoinOrder`.
    fn parse_join_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(target) = super::parse_unit_id(line).filter(|target| target.unitnum != -1) else {
            self.record_unit_error(unit_id, "JOIN: No target given.");
            return;
        };
        let mut overload = 1;
        let mut merge = 0;
        if let Some(option) = get_token(line) {
            if is_equal(&option, "nooverload") {
                overload = 0;
            } else if is_equal(&option, "merge") {
                merge = 1;
            }
        }
        if self.mode == OrderParseMode::Turn {
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .join_orders = Some(JoinOrder {
                target: Some(target),
                overload,
                merge,
                ..JoinOrder::default()
            });
        }
    }

    /// C++: `parseorders.cpp` `ProcessClaimOrder`.
    fn parse_claim_order(&mut self, unit_id: UnitId, line: &mut String) {
        let Some(token) = get_token(line) else {
            self.record_unit_error(unit_id, "CLAIM: No amount given.");
            return;
        };
        let mut amount = value(&token);
        if amount == 0 {
            self.record_unit_error(unit_id, "CLAIM: No amount given.");
            return;
        }
        if self.mode != OrderParseMode::Turn {
            return;
        }

        let faction_id = self
            .game
            .unit(unit_id)
            .and_then(|unit| unit.faction)
            .or(self.faction_id)
            .expect("selected unit has a faction");
        let unclaimed = self
            .game
            .faction(faction_id)
            .expect("selected faction exists")
            .unclaimed;
        if amount > unclaimed {
            self.record_unit_error(unit_id, "CLAIM: Don't have that much unclaimed silver.");
            amount = unclaimed;
        }

        let faction = self
            .game
            .faction_mut(faction_id)
            .expect("selected faction exists");
        faction.unclaimed -= amount;
        let money = self
            .game
            .unit(unit_id)
            .expect("selected unit exists")
            .items
            .get_num(ItemId::Silver);
        self.game
            .unit_mut(unit_id)
            .expect("selected unit exists")
            .items
            .set_num(ItemId::Silver, money.wrapping_add(amount));
        let tables = <dyn Ruleset>::from_name(self.defs.ruleset_name)
            .expect("parser ruleset is supported")
            .tables();
        crate::players::discover_item(
            self.game,
            faction_id,
            ItemId::Silver,
            ItemId::Silver.index(),
            &tables,
        );
        crate::players::unit_event(self.game, faction_id, unit_id, format!("Claims ${amount}."));
    }

    /// C++: `parseorders.cpp` `ProcessTaxOrder` and `ProcessPillageOrder`.
    fn parse_tax_pillage_order(&mut self, unit_id: UnitId, pillage: bool) {
        let taxing = self
            .game
            .unit(unit_id)
            .map(|unit| unit.taxing)
            .unwrap_or(TAX_NONE);
        if (!pillage && taxing == TAX_PILLAGE) || (pillage && taxing == TAX_TAX) {
            let error = if pillage {
                "PILLAGE: The unit is already taxing."
            } else {
                "TAX: The unit is already pillaging."
            };
            self.record_unit_error(unit_id, error);
            return;
        }

        let (has_month_order, delayed) = self
            .game
            .unit(unit_id)
            .map(|unit| {
                (
                    unit.orders.month_orders.is_some(),
                    unit.orders.in_turn_block != 0,
                )
            })
            .unwrap_or((false, false));
        if self.defs.tax_pillage_month_long != 0 && has_month_order {
            self.game
                .unit_mut(unit_id)
                .expect("selected unit exists")
                .orders
                .month_orders = None;
            let verb = if pillage { "PILLAGE" } else { "TAX" };
            let delayed = if delayed { "DELAYED " } else { "" };
            self.record_unit_error(
                unit_id,
                &format!("{verb}: Overwriting previous {delayed}month-long order."),
            );
        }
        self.game
            .unit_mut(unit_id)
            .expect("selected unit exists")
            .taxing = if pillage { TAX_PILLAGE } else { TAX_TAX };
    }

    fn set_month_order(
        &mut self,
        unit_id: UnitId,
        verb: &str,
        order: OrderData,
        clear_month_long_tax: bool,
    ) {
        let (has_month_order, has_month_long_tax, delayed) = self
            .game
            .unit(unit_id)
            .map(|unit| {
                (
                    unit.orders.month_orders.is_some(),
                    self.defs.tax_pillage_month_long != 0
                        && matches!(unit.taxing, TAX_TAX | TAX_PILLAGE),
                    unit.orders.in_turn_block != 0,
                )
            })
            .unwrap_or((false, false, false));
        if has_month_order || has_month_long_tax {
            if has_month_order {
                self.game
                    .unit_mut(unit_id)
                    .expect("selected unit exists")
                    .orders
                    .month_orders = None;
            }
            let delayed = if delayed { "DELAYED " } else { "" };
            self.record_unit_error(
                unit_id,
                &format!("{verb}: Overwriting previous {delayed}month-long order."),
            );
        }

        let unit = self.game.unit_mut(unit_id).expect("selected unit exists");
        if self.defs.tax_pillage_month_long != 0 && clear_month_long_tax {
            unit.taxing = TAX_NONE;
        }
        unit.orders.month_orders = Some(order);
    }

    fn store_old_order(&mut self, raw_line: &str, at_sign: bool, include_former: bool) {
        let Some(unit_id) = self.current_unit() else {
            return;
        };
        let former_id = self
            .game
            .unit(unit_id)
            .and_then(|unit| unit.former)
            .filter(|former_id| {
                self.game
                    .unit(*former_id)
                    .is_some_and(|former| former.format != 0)
            });
        if at_sign {
            if let Some(unit) = self.game.unit_mut(unit_id) {
                unit.old_orders.push(raw_line.to_owned());
            }
        }
        if include_former {
            if let Some(former_id) = former_id {
                if let Some(former) = self.game.unit_mut(former_id) {
                    former.old_orders.push(raw_line.to_owned());
                }
            }
        }
    }

    fn unwind_units(&mut self) {
        while let Some(unit_id) = self.unit_stack.pop() {
            let (in_turn_block, has_former) = self
                .game
                .unit(unit_id)
                .map(|unit| (unit.orders.in_turn_block != 0, unit.former.is_some()))
                .unwrap_or((false, false));
            if in_turn_block {
                self.record_unit_error(unit_id, "TURN: without ENDTURN");
            }
            if has_former {
                self.record_unit_error(unit_id, "FORM: without END.");
            }
            if self.mode == OrderParseMode::Check {
                if let Some(unit) = self.game.unit_mut(unit_id) {
                    unit.clear_orders();
                }
            }
        }
    }

    fn alias_exists(&self, object_id: ObjectId, faction_id: Option<FactionId>, alias: i32) -> bool {
        let Some(faction_number) = faction_id
            .and_then(|id| self.game.faction(id))
            .map(|faction| faction.num)
        else {
            return false;
        };
        let Some(region_id) = self.game.object(object_id).and_then(|object| object.region) else {
            return false;
        };
        let Some(region) = self.game.region(region_id) else {
            return false;
        };
        region.object_ids().iter().any(|object_id| {
            self.game.object(*object_id).is_some_and(|object| {
                object.unit_ids().iter().any(|unit_id| {
                    self.game.unit(*unit_id).is_some_and(|unit| {
                        unit.alias == alias
                            && (unit
                                .form_faction
                                .and_then(|id| self.game.faction(id))
                                .is_some_and(|faction| faction.num == faction_number)
                                || unit
                                    .faction
                                    .and_then(|id| self.game.faction(id))
                                    .is_some_and(|faction| faction.num == faction_number))
                    })
                })
            })
        })
    }

    fn first_unit(&self) -> Option<UnitId> {
        self.game.region_ids().iter().find_map(|region_id| {
            let region = self.game.region(*region_id)?;
            region
                .object_ids()
                .iter()
                .find_map(|object_id| self.game.object(*object_id)?.unit_ids().first().copied())
        })
    }

    fn find_faction(&self, number: i32) -> Option<FactionId> {
        self.game.faction_ids().iter().copied().find(|id| {
            self.game
                .faction(*id)
                .is_some_and(|faction| faction.num == number)
        })
    }

    fn current_unit(&self) -> Option<UnitId> {
        self.unit_stack.last().copied()
    }

    fn ruleset_tables(&self) -> &RulesetTables {
        self.tables
    }

    fn record_unit_error(&mut self, unit_id: UnitId, message: &str) {
        let (faction_id, name) = self
            .game
            .unit(unit_id)
            .map(|unit| (unit.faction.or(self.faction_id), unit.name.clone()))
            .unwrap_or((self.faction_id, None));
        self.record_error(faction_id, name.as_deref(), message);
    }

    fn record_error(
        &mut self,
        faction_id: Option<FactionId>,
        unit_name: Option<&str>,
        message: &str,
    ) {
        if self.mode == OrderParseMode::Check {
            self.result.errors.push(message.to_owned());
            return;
        }

        let Some(faction_id) = faction_id else {
            return;
        };
        let Some(faction) = self.game.faction_mut(faction_id) else {
            return;
        };
        if faction_is_npc(faction) {
            return;
        }

        let message = match unit_name {
            Some(name) => format!("{name}: {message}"),
            None => message.to_owned(),
        };
        if faction.errors.len() > 1000 {
            if faction.errors.len() == 1001 {
                faction.errors.push("Too many errors!".to_owned());
            }
            return;
        }
        faction.errors.push(message);
    }

    fn record_check_line(&mut self, raw_line: String) {
        if self.mode == OrderParseMode::Check {
            self.result.lines.push(ParsedOrderLine {
                raw: raw_line,
                indent: self.indent.max(0) as usize,
            });
        }
    }
}

fn parse_unit_targets(line: &mut String) -> Vec<super::OrderUnitId> {
    let mut targets = Vec::new();
    while let Some(target) = super::parse_unit_id(line) {
        if target.unitnum == -1 {
            break;
        }
        targets.push(target);
    }
    targets
}

/// C++: `parseorders.cpp` `Game::ParseDir`.
fn parse_dir(token: &str) -> Option<i32> {
    for (index, (name, abbreviation)) in crate::defs::strings::DIRECTION_NAMES
        .iter()
        .zip(crate::defs::strings::DIRECTION_ABBREVIATIONS)
        .enumerate()
    {
        if is_equal(token, name) || is_equal(token, abbreviation) {
            return Some(index as i32);
        }
    }
    if is_equal(token, "in") {
        return Some(super::MOVE_IN);
    }
    if is_equal(token, "out") {
        return Some(super::MOVE_OUT);
    }
    if is_equal(token, "pause") || is_equal(token, "p") {
        return Some(super::MOVE_PAUSE);
    }
    let number = value(token);
    (number != 0).then_some(super::MOVE_ENTER.wrapping_add(number))
}

fn faction_is_npc(faction: &Faction) -> bool {
    faction
        .faction_types
        .iter()
        .any(|(name, value)| *value == -1 && (is_equal(name, "War") || is_equal(name, "Martial")))
}

/// C++: `parseorders.cpp` `ProcessGiveOrder` item-class aliases.
fn give_all_item_class(token: &str) -> Option<i32> {
    [
        (ItemTypeFlags::NORMAL, &["normal"][..]),
        (ItemTypeFlags::ADVANCED, &["advanced"][..]),
        (ItemTypeFlags::TRADE, &["trade"][..]),
        (ItemTypeFlags::MAN, &["man", "men"][..]),
        (ItemTypeFlags::MONSTER, &["monster", "monsters"][..]),
        (ItemTypeFlags::MAGIC, &["magic"][..]),
        (ItemTypeFlags::WEAPON, &["weapon", "weapons"][..]),
        (ItemTypeFlags::ARMOR, &["armor"][..]),
        (ItemTypeFlags::MOUNT, &["mount", "mounts"][..]),
        (ItemTypeFlags::BATTLE, &["battle"][..]),
        (ItemTypeFlags::SPECIAL, &["special"][..]),
        (ItemTypeFlags::FOOD, &["food"][..]),
        (ItemTypeFlags::TOOL, &["tool", "tools"][..]),
        (crate::defs::NITEMS as i32, &["item", "items"][..]),
        (ItemTypeFlags::SHIP, &["ship", "ships"][..]),
    ]
    .into_iter()
    .find_map(|(item_class, names)| {
        names
            .iter()
            .any(|name| is_equal(token, name))
            .then_some(-item_class)
    })
}

/// C++: `aregion.cpp` `ARegion::IsCoastalOrLakeside`.
fn is_coastal_or_lakeside(
    game: &Game,
    region_id: crate::model::RegionId,
    tables: &RulesetTables,
) -> bool {
    let is_ocean = |region: &crate::model::Region| {
        region
            .terrain_type
            .and_then(|terrain| tables.terrains.get(terrain))
            .is_some_and(|terrain| terrain.similar_type == crate::defs::terrains::R_OCEAN as i32)
    };
    let Some(region) = game.region(region_id) else {
        return false;
    };
    is_ocean(region)
        || region
            .neighbors
            .iter()
            .flatten()
            .any(|neighbor_id| game.region(*neighbor_id).is_some_and(is_ocean))
}

/// C++: `unit.cpp` `Unit::Taxers`; only the positive-taxer result is needed here.
fn has_taxers(unit: &crate::model::Unit, defs: &GameDefs, tables: &RulesetTables) -> bool {
    let total_men = crate::players::unit_men(unit, tables);
    let can_tax_anyone = defs.who_can_tax & tax_flags::TAX_ANYONE != 0;
    let can_tax_by_skill = (defs.who_can_tax & tax_flags::TAX_COMBAT_SKILL != 0
        && skill_level(unit, "COMB", tables) > 0)
        || (defs.who_can_tax & tax_flags::TAX_BOW_SKILL != 0
            && (skill_level(unit, "XBOW", tables) > 0 || skill_level(unit, "LBOW", tables) > 0))
        || (defs.who_can_tax & tax_flags::TAX_RIDING_SKILL != 0
            && skill_level(unit, "RIDI", tables) > 0)
        || (defs.who_can_tax & tax_flags::TAX_STEALTH_SKILL != 0
            && skill_level(unit, "STEA", tables) > 0);

    let mut melee: i32 = 0;
    let mut usable_melee: i32 = 0;
    let mut bows: i32 = 0;
    let mut usable_bows: i32 = 0;
    let mut mounted_weapons: i32 = 0;
    let mut usable_mounted: i32 = 0;
    let mut mounts: i32 = 0;
    let mut usable_mounts: i32 = 0;
    let mut battle_items: i32 = 0;
    let mut usable_battle_items: i32 = 0;
    let mut creatures: i32 = 0;
    let mut illusions: i32 = 0;

    for item in unit.items.entries() {
        let Some(item_def) = tables.item_families.items.get(item.item_type.index()) else {
            continue;
        };
        let item_type = item_def.item_type;
        if item_type & ItemTypeFlags::BATTLE != 0 {
            let battle_item = item_def.abbreviation.as_deref().and_then(|abbreviation| {
                tables
                    .item_families
                    .battle_items
                    .iter()
                    .find(|battle_item| battle_item.abbreviation.as_deref() == Some(abbreviation))
            });
            if battle_item
                .is_some_and(|battle_item| battle_item.flags & BattleItemFlags::SPECIAL != 0)
            {
                let battle_item = battle_item.expect("special battle item was found");
                if defs.who_can_tax & tax_flags::TAX_USABLE_BATTLE_ITEM != 0
                    && (battle_item.flags & BattleItemFlags::MAGEONLY == 0
                        || matches!(unit.unit_type, UnitType::Mage | UnitType::Apprentice))
                {
                    usable_battle_items = usable_battle_items.wrapping_add(item.num);
                    battle_items = battle_items.wrapping_add(item.num);
                    continue;
                }
                if defs.who_can_tax & tax_flags::TAX_BATTLE_ITEM != 0 {
                    battle_items = battle_items.wrapping_add(item.num);
                    continue;
                }
            }
        }

        if item_type & ItemTypeFlags::WEAPON != 0 {
            if let Some(weapon) = item_def.abbreviation.as_deref().and_then(|abbreviation| {
                tables
                    .item_families
                    .weapons
                    .iter()
                    .find(|weapon| weapon.abbreviation.as_deref() == Some(abbreviation))
            }) {
                let base_skill = weapon
                    .base_skill
                    .as_deref()
                    .map_or(0, |skill| skill_level(unit, skill, tables));
                let base_skill = if base_skill == 0 {
                    weapon
                        .or_skill
                        .as_deref()
                        .map_or(0, |skill| skill_level(unit, skill, tables))
                } else {
                    base_skill
                };
                if weapon.flags & WeaponFlags::NEEDSKILL == 0 {
                    if base_skill > 0 {
                        usable_melee = usable_melee.wrapping_add(item.num);
                    }
                    melee = melee.wrapping_add(item.num);
                } else if weapon.flags & WeaponFlags::NOFOOT != 0 {
                    if base_skill > 0 {
                        usable_mounted = usable_mounted.wrapping_add(item.num);
                    }
                    mounted_weapons = mounted_weapons.wrapping_add(item.num);
                } else if weapon.flags & WeaponFlags::RANGED != 0 {
                    if base_skill > 0 {
                        usable_bows = usable_bows.wrapping_add(item.num);
                    }
                    bows = bows.wrapping_add(item.num);
                } else {
                    if base_skill > 0 {
                        usable_melee = usable_melee.wrapping_add(item.num);
                    }
                    melee = melee.wrapping_add(item.num);
                }
            }
        }

        if item_type & ItemTypeFlags::MOUNT != 0 {
            let usable = item_def.abbreviation.as_deref().and_then(|abbreviation| {
                tables
                    .item_families
                    .mounts
                    .iter()
                    .find(|mount| mount.abbreviation.as_deref() == Some(abbreviation))
            });
            if usable.is_some_and(|mount| {
                mount
                    .skill
                    .as_deref()
                    .is_none_or(|skill| skill_level(unit, skill, tables) >= mount.minimum_bonus)
            }) {
                usable_mounts = usable_mounts.wrapping_add(item.num);
            }
            mounts = mounts.wrapping_add(item.num);
        }

        if item_type & ItemTypeFlags::MONSTER != 0 {
            if item_type & ItemTypeFlags::ILLUSION != 0 {
                illusions = illusions.wrapping_add(item.num);
            } else {
                creatures = creatures.wrapping_add(item.num);
            }
        }
    }

    let mut taxers = if can_tax_anyone || can_tax_by_skill {
        total_men
    } else {
        let mut total: i32 = 0;
        if defs.who_can_tax & tax_flags::TAX_USABLE_WEAPON != 0 {
            if usable_mounted > usable_mounts {
                total = usable_mounts;
                mounts = mounts.wrapping_sub(usable_mounts);
                usable_mounts = 0;
            } else {
                total = usable_mounted;
                mounts = mounts.wrapping_sub(usable_mounted);
                usable_mounts = usable_mounts.wrapping_sub(usable_mounted);
            }
            total = total.wrapping_add(melee).wrapping_add(usable_bows);
        } else if defs.who_can_tax & tax_flags::TAX_ANY_WEAPON != 0 {
            total = melee.wrapping_add(bows).wrapping_add(mounted_weapons);
        } else {
            if defs.who_can_tax & tax_flags::TAX_MELEE_WEAPON_AND_MATCHING_SKILL != 0 {
                if usable_mounted > usable_mounts {
                    total = total.wrapping_add(usable_mounts);
                    mounts = mounts.wrapping_sub(usable_mounts);
                    usable_mounts = 0;
                } else {
                    total = total.wrapping_add(usable_mounted);
                    mounts = mounts.wrapping_sub(usable_mounted);
                    usable_mounts = usable_mounts.wrapping_sub(usable_mounted);
                }
                total = total.wrapping_add(usable_melee);
            }
            if defs.who_can_tax & tax_flags::TAX_BOW_SKILL_AND_MATCHING_WEAPON != 0 {
                total = total.wrapping_add(usable_bows);
            }
        }
        if defs.who_can_tax & tax_flags::TAX_HORSE != 0 {
            total = total.wrapping_add(mounts);
        } else if defs.who_can_tax & tax_flags::TAX_HORSE_AND_RIDING_SKILL != 0 {
            total = total.wrapping_add(usable_mounts);
        }
        if defs.who_can_tax & tax_flags::TAX_BATTLE_ITEM != 0 {
            total = total.wrapping_add(battle_items);
        } else if defs.who_can_tax & tax_flags::TAX_USABLE_BATTLE_ITEM != 0 {
            total = total.wrapping_add(usable_battle_items);
        }
        total
    };

    if unit.unit_type == UnitType::Mage {
        let mage_can_tax = if defs.who_can_tax & tax_flags::TAX_ANY_MAGE != 0 {
            true
        } else if defs.who_can_tax & tax_flags::TAX_MAGE_COMBAT_SPELL != 0 {
            usize::try_from(unit.combat)
                .ok()
                .and_then(|skill_id| tables.skills.skills.get(skill_id))
                .is_some_and(|skill| mage_skill_can_tax(skill.flags, defs.who_can_tax))
        } else {
            unit.skills.entries().iter().any(|entry| {
                tables
                    .skills
                    .skills
                    .get(entry.skill_type)
                    .is_some_and(|skill| mage_skill_can_tax(skill.flags, defs.who_can_tax))
            })
        };
        if mage_can_tax {
            taxers = total_men;
        }
    }

    if taxers > total_men {
        taxers = total_men;
    }
    if defs.who_can_tax & tax_flags::TAX_CREATURES != 0 {
        taxers = taxers.wrapping_add(creatures);
    }
    if defs.who_can_tax & tax_flags::TAX_ILLUSIONS != 0 {
        taxers = taxers.wrapping_add(illusions);
    }
    taxers > 0
}

fn mage_skill_can_tax(skill_flags: i32, tax_flags: i32) -> bool {
    (tax_flags & tax_flags::TAX_MAGE_DAMAGE != 0 && skill_flags & skill_flags::DAMAGE != 0)
        || (tax_flags & tax_flags::TAX_MAGE_FEAR != 0 && skill_flags & skill_flags::FEAR != 0)
        || (tax_flags & tax_flags::TAX_MAGE_OTHER != 0
            && skill_flags & skill_flags::MAGE_OTHER != 0)
}

fn skill_level(unit: &crate::model::Unit, abbreviation: &str, tables: &RulesetTables) -> i32 {
    let Some(skill_id) = crate::defs::skills::lookup_skill(&tables.skills.skills, abbreviation)
    else {
        return 0;
    };
    let mut level = crate::players::real_skill_level(unit, skill_id, tables);
    let Some(skill) = tables.skills.skills.get(skill_id) else {
        return level;
    };
    if skill.flags & skill_flags::MAGIC != 0
        && !matches!(
            unit.unit_type,
            UnitType::Mage | UnitType::Apprentice | UnitType::GuardMage
        )
    {
        return level;
    }
    let total_men = crate::players::unit_men(unit, tables);
    for item in unit.items.entries() {
        let Some(item_def) = tables.item_families.items.get(item.item_type.index()) else {
            continue;
        };
        if item_def.flags & ItemFlags::DISABLED != 0
            || (item_def.item_type & ItemTypeFlags::MAGE_ONLY != 0
                && !matches!(
                    unit.unit_type,
                    UnitType::Mage | UnitType::Apprentice | UnitType::GuardMage
                ))
            || (item.num < total_men)
            || item_def
                .grant_skill
                .as_deref()
                .and_then(|grant| crate::defs::skills::lookup_skill(&tables.skills.skills, grant))
                != Some(skill_id)
        {
            continue;
        }
        let mut granted = item_def
            .grant_from
            .iter()
            .flatten()
            .filter_map(|source| crate::defs::skills::lookup_skill(&tables.skills.skills, source))
            .map(|source| crate::players::real_skill_level(unit, source, tables))
            .max()
            .unwrap_or(0);
        granted = granted.max(item_def.min_grant);
        granted = granted.min(item_def.max_grant);
        level = level.max(granted);
    }
    level
}

#[cfg(test)]
mod tests {
    use super::{parse_order_file as parse_order_file_with_tables, OrderParseMode};
    use crate::defs::{skills::skill_flags, GameDefs, ItemId};
    use crate::model::{Faction, Game, Object, Region, Unit};
    use crate::orders::{
        BuildOrder, BuyOrder, CastIntOrder, CastOrder, CastOrderData, CastRegionOrder,
        CastTransmuteOrder, CastUnitsOrder, ExchangeOrder, GiveOrder, JoinOrder, MoveDir, MoveOrder,
        Order, OrderData, OrderUnitId, OrderVerb, ProduceOrder, SellOrder, TeleportOrder,
        TransportOrder, TurnOrder, WithdrawOrder, MOVE_ENTER, MOVE_IN, MOVE_OUT, MOVE_PAUSE,
    };
    use crate::ruleset::Ruleset;
    use crate::text::is_equal;

    fn parse_order_file(
        game: &mut Game,
        defs: &GameDefs,
        input: &[u8],
        mode: OrderParseMode,
    ) -> Result<super::OrderParseResult, super::OrderParseError> {
        let ruleset = <dyn Ruleset>::from_name(defs.ruleset_name)
            .expect("parser test uses a supported ruleset");
        let tables = ruleset.tables();
        parse_order_file_with_tables(game, defs, &tables, input, mode)
    }

    #[test]
    fn cast_validation_matches_cpp_for_both_rulesets() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let tables = ruleset.tables();
            let skills = &tables.skills.skills;
            let non_magic_skill = skills
                .iter()
                .find(|skill| {
                    skill.abbreviation.is_some()
                        && skill.flags & (skill_flags::MAGIC | skill_flags::DISABLED) == 0
                })
                .and_then(|skill| skill.abbreviation)
                .expect("ruleset has an enabled non-magic skill");
            let magic_non_cast_skill = skills
                .iter()
                .find(|skill| {
                    skill.abbreviation.is_some()
                        && skill.flags & skill_flags::MAGIC != 0
                        && skill.flags & (skill_flags::CAST | skill_flags::DISABLED) == 0
                })
                .and_then(|skill| skill.abbreviation)
                .expect("ruleset has a magic skill that is not castable");
            let castable_skill = skills
                .iter()
                .find(|skill| {
                    skill.abbreviation.is_some()
                        && skill.flags & (skill_flags::MAGIC | skill_flags::CAST)
                            == (skill_flags::MAGIC | skill_flags::CAST)
                        && skill.flags & skill_flags::DISABLED == 0
                })
                .and_then(|skill| skill.abbreviation)
                .expect("ruleset has an enabled castable skill");
            let mut invalid_orders = vec![
                ("cast".to_owned(), "CAST: No skill given."),
                ("cast not_a_skill".to_owned(), "CAST: Invalid skill."),
                (
                    format!("cast {non_magic_skill}"),
                    "CAST: That is not a magic skill.",
                ),
                (
                    format!("cast {magic_non_cast_skill}"),
                    "CAST: That skill cannot be CAST.",
                ),
            ];
            if let Some(disabled_skill) = skills
                .iter()
                .find(|skill| {
                    skill.abbreviation.is_some() && skill.flags & skill_flags::DISABLED != 0
                })
                .and_then(|skill| skill.abbreviation)
            {
                invalid_orders.push((format!("cast {disabled_skill}"), "CAST: Invalid skill."));
            }

            for (order, expected_error) in invalid_orders {
                let (mut game, faction_id, _, _) = game_with_unit();
                let input = format!("#atlantis 7 none\nunit 10\n{order}\n#end\n");
                parse_order_file_with_tables(
                    &mut game,
                    ruleset.game_defs(),
                    &tables,
                    input.as_bytes(),
                    OrderParseMode::Turn,
                )
                .unwrap();
                assert_eq!(
                    game.faction(faction_id).unwrap().errors,
                    [format!("Unit (10): {expected_error}")],
                    "{ruleset_name}: {order}"
                );
            }

            let (mut check_game, _, _, check_unit) = game_with_unit();
            let input = format!("#atlantis 7 none\nunit 10\ncast {castable_skill}\n#end\n");
            let check = parse_order_file_with_tables(
                &mut check_game,
                ruleset.game_defs(),
                &tables,
                input.as_bytes(),
                OrderParseMode::Check,
            )
            .unwrap();
            assert!(check.errors.is_empty(), "{ruleset_name}");
            assert!(
                check_game
                    .unit(check_unit)
                    .unwrap()
                    .orders
                    .cast_orders
                    .is_none(),
                "{ruleset_name}"
            );
        }
    }

    #[test]
    fn cast_generic_and_mind_reading_orders_store_expected_data() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let tables = ruleset.tables();
            let generic_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "CPOR").unwrap();
            let (mut generic_game, generic_faction, _, generic_unit) = game_with_unit();
            let generic = parse_order_file_with_tables(
                &mut generic_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast CPOR\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert!(generic.errors.is_empty(), "{ruleset_name}");
            let Some(CastOrderData::Cast(order)) =
                &generic_game.unit(generic_unit).unwrap().orders.cast_orders
            else {
                panic!("expected generic cast order for {ruleset_name}");
            };
            assert_eq!(order.spell, generic_spell as i32);
            assert_eq!(order.level, 1);
            assert!(generic_game
                .unit(generic_unit)
                .unwrap()
                .orders
                .teleport_orders
                .is_none());
            assert!(generic_game
                .faction(generic_faction)
                .unwrap()
                .errors
                .is_empty());

            let mind_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "MIND").unwrap();
            let (mut mind_game, mind_faction, _, mind_unit) = game_with_unit();
            let mind = parse_order_file_with_tables(
                &mut mind_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast MIND 11\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert!(mind.errors.is_empty(), "{ruleset_name}");
            let Some(CastOrderData::Mind(order)) =
                &mind_game.unit(mind_unit).unwrap().orders.cast_orders
            else {
                panic!("expected mind-reading cast order for {ruleset_name}");
            };
            assert_eq!(order.spell, mind_spell as i32);
            assert_eq!(order.level, 1);
            assert_eq!(
                order.id,
                Some(OrderUnitId {
                    unitnum: 11,
                    alias: 0,
                    faction: 0,
                })
            );
            assert!(mind_game.faction(mind_faction).unwrap().errors.is_empty());
        }
    }

    #[test]
    fn cast_region_spells_use_caster_coordinates_and_validate_cross_level_targets() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let mut tables = ruleset.tables();
            let farsight_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "FARS").unwrap();
            let range = tables.skills.skills[farsight_spell]
                .range
                .expect("Farsight has a range");
            tables.skills.ranges[range].flags |= crate::defs::skills::range_flags::CROSS_LEVELS;
            let teleport_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "TELE").unwrap();

            let (mut region_game, region_faction, object_id, region_unit) = game_with_unit();
            let region_id = region_game.object(object_id).unwrap().region.unwrap();
            let region = region_game.region_mut(region_id).unwrap();
            region.xloc = 4;
            region.yloc = 8;
            region.zloc = 2;
            let result = parse_order_file_with_tables(
                &mut region_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast FARS\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert!(result.errors.is_empty(), "{ruleset_name}");
            assert_eq!(
                region_game.unit(region_unit).unwrap().orders.cast_orders,
                Some(CastOrderData::Region(CastRegionOrder {
                    spell: farsight_spell as i32,
                    level: 1,
                    xloc: 4,
                    yloc: 8,
                    zloc: 2,
                    ..CastRegionOrder::default()
                }))
            );
            assert!(region_game
                .faction(region_faction)
                .unwrap()
                .errors
                .is_empty());

            let (mut explicit_game, _, explicit_object, explicit_unit) = game_with_unit();
            let explicit_region = explicit_game
                .object(explicit_object)
                .unwrap()
                .region
                .unwrap();
            explicit_game.region_mut(explicit_region).unwrap().zloc = 2;
            parse_order_file_with_tables(
                &mut explicit_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast FARS region 12 8 1\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                explicit_game
                    .unit(explicit_unit)
                    .unwrap()
                    .orders
                    .cast_orders,
                Some(CastOrderData::Region(CastRegionOrder {
                    spell: farsight_spell as i32,
                    level: 1,
                    xloc: 12,
                    yloc: 8,
                    zloc: 1,
                    ..CastRegionOrder::default()
                }))
            );
            let (mut teleport_game, _, teleport_object, teleport_unit) = game_with_unit();
            let teleport_region = teleport_game
                .object(teleport_object)
                .unwrap()
                .region
                .unwrap();
            {
                let region = teleport_game.region_mut(teleport_region).unwrap();
                region.xloc = 3;
                region.yloc = 6;
                region.zloc = 1;
            }
            parse_order_file_with_tables(
                &mut teleport_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast TELE\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                teleport_game
                    .unit(teleport_unit)
                    .unwrap()
                    .orders
                    .teleport_orders,
                Some(TeleportOrder {
                    spell: teleport_spell as i32,
                    level: 1,
                    xloc: 3,
                    yloc: 6,
                    zloc: 1,
                    ..TeleportOrder::default()
                })
            );
            assert!(teleport_game
                .unit(teleport_unit)
                .unwrap()
                .orders
                .cast_orders
                .is_none());

            let invalid_z = ruleset.game_defs().underworld_levels
                + ruleset.game_defs().underdeep_levels
                + ruleset.game_defs().abyss_level
                + 2;
            let (mut invalid_game, invalid_faction, _, _) = game_with_unit();
            parse_order_file_with_tables(
                &mut invalid_game,
                ruleset.game_defs(),
                &tables,
                format!("#atlantis 7 none\nunit 10\ncast FARS region 12 8 {invalid_z}\n#end\n")
                    .as_bytes(),
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                invalid_game.faction(invalid_faction).unwrap().errors,
                ["Unit (10): CAST: Invalid Z coordinate specified."],
                "{ruleset_name}"
            );
        }
    }

    #[test]
    fn cast_gate_portal_and_invisibility_orders_preserve_target_lists() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let tables = ruleset.tables();
            let invisibility_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "INVI").unwrap();
            let gate_lore_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "GATE").unwrap();
            let portal_lore_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "PORT").unwrap();

            let (mut invisibility_game, _, _, invisibility_unit) = game_with_unit();
            let invisibility = parse_order_file_with_tables(
                &mut invisibility_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast INVI units 11 12\ncast INVI units 13\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert!(invisibility.errors.is_empty(), "{ruleset_name}");
            assert_eq!(
                invisibility_game
                    .unit(invisibility_unit)
                    .unwrap()
                    .orders
                    .cast_orders,
                Some(CastOrderData::Units(CastUnitsOrder {
                    spell: invisibility_spell as i32,
                    level: 1,
                    units: [11, 12, 13]
                        .map(|unitnum| OrderUnitId {
                            unitnum,
                            alias: 0,
                            faction: 0,
                        })
                        .to_vec(),
                    ..CastUnitsOrder::default()
                }))
            );

            let (mut gate_game, _, _, gate_unit) = game_with_unit();
            parse_order_file_with_tables(
                &mut gate_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast GATE gate 4 units 11 12\ncast GATE gate 4 units 13\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                gate_game.unit(gate_unit).unwrap().orders.teleport_orders,
                Some(TeleportOrder {
                    spell: gate_lore_spell as i32,
                    level: 3,
                    gate: 4,
                    units: [11, 12, 13]
                        .map(|unitnum| OrderUnitId {
                            unitnum,
                            alias: 0,
                            faction: 0,
                        })
                        .to_vec(),
                    ..TeleportOrder::default()
                })
            );

            let (mut random_gate_game, _, _, random_gate_unit) = game_with_unit();
            parse_order_file_with_tables(
                &mut random_gate_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast GATE random level units 14\ncast GATE random level units 15\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                random_gate_game
                    .unit(random_gate_unit)
                    .unwrap()
                    .orders
                    .teleport_orders,
                Some(TeleportOrder {
                    spell: gate_lore_spell as i32,
                    level: 2,
                    gate: -2,
                    units: vec![OrderUnitId {
                        unitnum: 15,
                        alias: 0,
                        faction: 0,
                    }],
                    ..TeleportOrder::default()
                })
            );

            let (mut detect_game, _, _, detect_unit) = game_with_unit();
            parse_order_file_with_tables(
                &mut detect_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast GATE detect\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                detect_game.unit(detect_unit).unwrap().orders.cast_orders,
                Some(CastOrderData::Cast(CastOrder {
                    spell: gate_lore_spell as i32,
                    level: 2,
                    ..CastOrder::default()
                }))
            );

            let (mut portal_game, _, _, portal_unit) = game_with_unit();
            parse_order_file_with_tables(
                &mut portal_game,
                ruleset.game_defs(),
                &tables,
                b"#atlantis 7 none\nunit 10\ncast PORT 7 units 15\ncast PORT 8 units 16\n#end\n",
                OrderParseMode::Turn,
            )
            .unwrap();
            assert_eq!(
                portal_game
                    .unit(portal_unit)
                    .unwrap()
                    .orders
                    .teleport_orders,
                Some(TeleportOrder {
                    spell: portal_lore_spell as i32,
                    level: 1,
                    gate: 8,
                    units: [15, 16]
                        .map(|unitnum| OrderUnitId {
                            unitnum,
                            alias: 0,
                            faction: 0,
                        })
                        .to_vec(),
                    ..TeleportOrder::default()
                })
            );

            for (order, expected_error) in [
                ("cast INVI", "CAST: Must specify units to render invisible."),
                ("cast GATE", "CAST: Missing argument."),
                ("cast GATE gate 0", "CAST: Requires a target gate."),
                ("cast PORT", "CAST: Requires a target mage."),
                ("cast PORT 7", "CAST: No units to teleport."),
                ("cast PORT 7 allies", "CAST: No units to teleport."),
            ] {
                let (mut game, faction_id, _, _) = game_with_unit();
                let input = format!("#atlantis 7 none\nunit 10\n{order}\n#end\n");
                parse_order_file_with_tables(
                    &mut game,
                    ruleset.game_defs(),
                    &tables,
                    input.as_bytes(),
                    OrderParseMode::Turn,
                )
                .unwrap();
                assert_eq!(
                    game.faction(faction_id).unwrap().errors,
                    [format!("Unit (10): {expected_error}")],
                    "{ruleset_name}: {order}"
                );
            }
        }
    }

    #[test]
    fn cast_integer_spell_and_transmutation_variants_match_cpp() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let tables = ruleset.tables();
            let integer_cases = [
                ("BIRD", "eagle", 3, 0),
                ("BIRD", "direction northwest", 1, 5),
                ("PHBE", "wolf 2", 1, 2),
                ("PHUN", "skeleton 3", 1, 3),
                ("PHDE", "demon 4", 2, 4),
                ("PHBE", "dragon", 3, 1),
            ];
            for (abbreviation, arguments, level, target) in integer_cases {
                let Some(spell) =
                    crate::defs::skills::parse_skill(&tables.skills.skills, abbreviation)
                else {
                    assert_eq!(abbreviation, "PHUN", "{ruleset_name}");
                    continue;
                };
                let (mut game, faction_id, _, unit_id) = game_with_unit();
                let input =
                    format!("#atlantis 7 none\nunit 10\ncast {abbreviation} {arguments}\n#end\n");
                let result = parse_order_file_with_tables(
                    &mut game,
                    ruleset.game_defs(),
                    &tables,
                    input.as_bytes(),
                    OrderParseMode::Turn,
                )
                .unwrap();
                assert!(result.errors.is_empty(), "{ruleset_name}: {abbreviation}");
                assert_eq!(
                    game.unit(unit_id).unwrap().orders.cast_orders,
                    Some(CastOrderData::Int(CastIntOrder {
                        spell: spell as i32,
                        level,
                        target,
                        ..CastIntOrder::default()
                    })),
                    "{ruleset_name}: {abbreviation}"
                );
                assert!(game.faction(faction_id).unwrap().errors.is_empty());
            }

            let transmutation_spell =
                crate::defs::skills::parse_skill(&tables.skills.skills, "TRNS");
            if let Some(spell) = transmutation_spell {
                for (item_token, item_name, level) in [
                    ("mithril", "mithril", 1),
                    ("rootstone", "rootstone", 1),
                    ("ironwood", "ironwood", 2),
                    ("FLOA", "floater hide", 3),
                    ("yew", "yew", 4),
                    ("WING", "winged horse", 5),
                    ("admantium", "admantium", 5),
                ] {
                    let mut case_tables = tables.clone();
                    if item_token == "WING" {
                        case_tables.item_families.items[ItemId::Whorse.index()].flags &=
                            !crate::defs::ItemFlags::DISABLED;
                    }
                    let item = case_tables
                        .item_families
                        .parse_enabled_item(item_token)
                        .unwrap_or_else(|| panic!("{ruleset_name} has enabled item {item_name}"));
                    let (mut game, _, _, unit_id) = game_with_unit();
                    let input =
                        format!("#atlantis 7 none\nunit 10\ncast TRNS 5 {item_token}\n#end\n");
                    parse_order_file_with_tables(
                        &mut game,
                        ruleset.game_defs(),
                        &case_tables,
                        input.as_bytes(),
                        OrderParseMode::Turn,
                    )
                    .unwrap();
                    assert_eq!(
                        game.unit(unit_id).unwrap().orders.cast_orders,
                        Some(CastOrderData::Transmute(CastTransmuteOrder {
                            spell: spell as i32,
                            level,
                            item: item as i32,
                            number: 5,
                            ..CastTransmuteOrder::default()
                        })),
                        "{ruleset_name}: {item_name}"
                    );
                }
            } else {
                assert_ne!(
                    tables.skills.skills
                        [crate::defs::skills::lookup_skill(&tables.skills.skills, "TRNS").unwrap()]
                    .flags
                        & skill_flags::DISABLED,
                    0,
                    "{ruleset_name}"
                );
            }

            let mut error_cases = vec![
                ("cast BIRD", "CAST: Missing arguments."),
                ("cast BIRD direction invalid", "CAST: Invalid direction."),
                ("cast BIRD invalid", "CAST: Invalid arguments."),
                ("cast PHBE", "CAST: Must specify which illusion to summon."),
                (
                    "cast PHBE dragons",
                    "CAST: Must specify which illusion to summon.",
                ),
                ("cast PHDE", "CAST: Illusion to summon must be given."),
                ("cast PHDE unicorn", "CAST: Can't summon that illusion."),
            ];
            error_cases.push((
                "cast PHUN unknown",
                if crate::defs::skills::parse_skill(&tables.skills.skills, "PHUN").is_some() {
                    "CAST: Must specify which illusion to summon."
                } else {
                    "CAST: Invalid skill."
                },
            ));
            if transmutation_spell.is_some() {
                error_cases.extend([
                    (
                        "cast TRNS",
                        "CAST: You must specify what you wish to create.",
                    ),
                    (
                        "cast TRNS iron",
                        "CAST: Can't create that by transmutation.",
                    ),
                ]);
            } else {
                error_cases.push(("cast TRNS", "CAST: Invalid skill."));
            }
            for (order, expected_error) in error_cases {
                let (mut game, faction_id, _, _) = game_with_unit();
                let input = format!("#atlantis 7 none\nunit 10\n{order}\n#end\n");
                parse_order_file_with_tables(
                    &mut game,
                    ruleset.game_defs(),
                    &tables,
                    input.as_bytes(),
                    OrderParseMode::Turn,
                )
                .unwrap();
                assert_eq!(
                    game.faction(faction_id).unwrap().errors,
                    [format!("Unit (10): {expected_error}")],
                    "{ruleset_name}: {order}"
                );
            }
        }
    }

    #[test]
    fn cast_every_castable_skill_routes_for_both_rulesets() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum ExpectedOrder {
            Generic,
            Mind,
            Region,
            Teleport,
            Integer,
            Units,
            Transmutation,
            NoHandler,
        }

        const GENERIC_SPELLS: &[&str] = &[
            "construct portal",
            "enchant swords",
            "enchant armor",
            "enchant shields",
            "construct gate",
            "engrave runes of warding",
            "summon imps",
            "summon demon",
            "summon balrog",
            "summon skeletons",
            "raise undead",
            "summon lich",
            "dragon lore",
            "wolf lore",
            "earth lore",
            "summon wind",
            "create ring of invisibility",
            "create cloak of invulnerability",
            "create staff of fire",
            "create staff of lightning",
            "create amulet of true seeing",
            "create amulet of protection",
            "create runesword",
            "create shieldstone",
            "create magic carpet",
            "create flaming sword",
            "create food",
            "create aegis",
            "create windchime",
            "create gate crystal",
            "create staff of healing",
            "create scrying orb",
            "create cornucopia",
            "create book of exorcism",
            "create holy symbol",
            "create censer",
            "blasphemous ritual",
            "phantasmal entertainment",
        ];

        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let tables = ruleset.tables();
            let mut castable_count = 0;
            for (spell, skill) in tables.skills.skills.iter().enumerate() {
                if skill.flags & (skill_flags::MAGIC | skill_flags::CAST)
                    != (skill_flags::MAGIC | skill_flags::CAST)
                {
                    continue;
                }
                castable_count += 1;
                let token = skill
                    .abbreviation
                    .expect("castable skills have an abbreviation");
                let name = skill.name.unwrap_or("");
                let (expected, arguments) = if skill.flags & skill_flags::DISABLED != 0 {
                    let (mut game, faction_id, _, _) = game_with_unit();
                    let input = format!("#atlantis 7 none\nunit 10\ncast {token}\n#end\n");
                    parse_order_file_with_tables(
                        &mut game,
                        ruleset.game_defs(),
                        &tables,
                        input.as_bytes(),
                        OrderParseMode::Turn,
                    )
                    .unwrap();
                    assert_eq!(
                        game.faction(faction_id).unwrap().errors,
                        ["Unit (10): CAST: Invalid skill."],
                        "{ruleset_name}: {name}"
                    );
                    continue;
                } else if is_equal(name, "mind reading") {
                    (ExpectedOrder::Mind, "11")
                } else if is_equal(name, "farsight")
                    || is_equal(name, "weather lore")
                    || (is_equal(name, "clear skies")
                        && skill
                            .range
                            .and_then(|range| tables.skills.ranges.get(range))
                            .is_some())
                {
                    (ExpectedOrder::Region, "")
                } else if is_equal(name, "teleportation") {
                    (ExpectedOrder::Teleport, "")
                } else if is_equal(name, "invisibility") {
                    (ExpectedOrder::Units, "units 10")
                } else if is_equal(name, "gate lore") {
                    (ExpectedOrder::Generic, "detect")
                } else if is_equal(name, "portal lore") {
                    (ExpectedOrder::Teleport, "1 units 10")
                } else if is_equal(name, "bird lore") {
                    (ExpectedOrder::Integer, "eagle")
                } else if is_equal(name, "create phantasmal beasts") {
                    (ExpectedOrder::Integer, "wolf")
                } else if is_equal(name, "create phantasmal undead") {
                    (ExpectedOrder::Integer, "skeleton")
                } else if is_equal(name, "create phantasmal demons") {
                    (ExpectedOrder::Integer, "imp")
                } else if is_equal(name, "transmutation") {
                    (ExpectedOrder::Transmutation, "mithril")
                } else if GENERIC_SPELLS.iter().any(|spell| is_equal(name, spell))
                    || is_equal(name, "clear skies")
                {
                    (ExpectedOrder::Generic, "")
                } else {
                    (ExpectedOrder::NoHandler, "")
                };

                let (mut game, faction_id, _, unit_id) = game_with_unit();
                let input = format!("#atlantis 7 none\nunit 10\ncast {token} {arguments}\n#end\n");
                parse_order_file_with_tables(
                    &mut game,
                    ruleset.game_defs(),
                    &tables,
                    input.as_bytes(),
                    OrderParseMode::Turn,
                )
                .unwrap();
                assert!(
                    game.faction(faction_id).unwrap().errors.is_empty(),
                    "{ruleset_name}: {name}"
                );
                let unit = game.unit(unit_id).unwrap();
                match expected {
                    ExpectedOrder::Generic => assert!(matches!(
                        unit.orders.cast_orders.as_ref(),
                        Some(CastOrderData::Cast(_))
                    )),
                    ExpectedOrder::Mind => assert!(matches!(
                        unit.orders.cast_orders.as_ref(),
                        Some(CastOrderData::Mind(_))
                    )),
                    ExpectedOrder::Region => assert!(matches!(
                        unit.orders.cast_orders.as_ref(),
                        Some(CastOrderData::Region(_))
                    )),
                    ExpectedOrder::Teleport => {
                        assert!(unit.orders.teleport_orders.as_ref().is_some())
                    }
                    ExpectedOrder::Integer => assert!(matches!(
                        unit.orders.cast_orders.as_ref(),
                        Some(CastOrderData::Int(_))
                    )),
                    ExpectedOrder::Units => assert!(matches!(
                        unit.orders.cast_orders.as_ref(),
                        Some(CastOrderData::Units(_))
                    )),
                    ExpectedOrder::Transmutation => {
                        let Some(CastOrderData::Transmute(order)) =
                            unit.orders.cast_orders.as_ref()
                        else {
                            panic!("expected transmutation order for {ruleset_name}: {name}");
                        };
                        assert_eq!(order.level, 1, "{ruleset_name}: {name}");
                        assert_eq!(
                            order.item,
                            tables
                                .item_families
                                .parse_enabled_item("mithril")
                                .expect("mithril is enabled") as i32,
                            "{ruleset_name}: {name}"
                        );
                        assert_eq!(order.number, -1, "{ruleset_name}: {name}");
                    }
                    ExpectedOrder::NoHandler => {
                        assert!(unit.orders.cast_orders.as_ref().is_none());
                        assert!(unit.orders.teleport_orders.as_ref().is_none());
                    }
                }
                if expected != ExpectedOrder::Teleport {
                    assert!(
                        unit.orders.teleport_orders.as_ref().is_none(),
                        "{ruleset_name}: {name}"
                    );
                } else if !matches!(name, "gate lore") {
                    assert!(
                        unit.orders.cast_orders.as_ref().is_none(),
                        "{ruleset_name}: {name}"
                    );
                }
                assert_eq!(
                    unit.orders
                        .cast_orders
                        .as_ref()
                        .map(|order| match order {
                            CastOrderData::Cast(order) => order.spell,
                            CastOrderData::Mind(order) => order.spell,
                            CastOrderData::Region(order) => order.spell,
                            CastOrderData::Int(order) => order.spell,
                            CastOrderData::Units(order) => order.spell,
                            CastOrderData::Transmute(order) => order.spell,
                        })
                        .or_else(|| {
                            unit.orders
                                .teleport_orders
                                .as_ref()
                                .map(|order| order.spell)
                        }),
                    if expected == ExpectedOrder::NoHandler {
                        None
                    } else {
                        Some(spell as i32)
                    },
                    "{ruleset_name}: {name}"
                );
            }
            assert!(castable_count > 0, "{ruleset_name}");
        }
    }

    #[test]
    fn study_parses_skill_and_optional_level_and_replaces_month_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let tables = ruleset.tables();
        let combat = crate::defs::skills::parse_skill(&tables.skills.skills, "COMB").unwrap();
        let riding = crate::defs::skills::parse_skill(&tables.skills.skills, "RIDI").unwrap();
        let (mut game, faction_id, object_id, unit_id) = game_with_unit();
        let second_unit = game
            .add_unit(object_id, Unit::new(11, Some(faction_id), 1, 5))
            .unwrap();
        let unknown_skill_unit = game
            .add_unit(object_id, Unit::new(12, Some(faction_id), 1, 5))
            .unwrap();
        game.setup_unit_nums().unwrap();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nmove north\nstudy COMB 3\nunit 11\nstudy RIDI\nstudy\nunit 12\nstudy not-a-skill\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): STUDY: Overwriting previous month-long order.",
                "Unit (11): STUDY: No skill given."
            ]
        );
        match &game.unit(unit_id).unwrap().orders.month_orders {
            Some(OrderData::Study(order)) => {
                assert_eq!(order.skill, i32::try_from(combat).unwrap());
                assert_eq!(order.days, 0);
                assert_eq!(order.level, 3);
            }
            other => panic!("expected STUDY order, got {other:?}"),
        }
        match &game.unit(second_unit).unwrap().orders.month_orders {
            Some(OrderData::Study(order)) => {
                assert_eq!(order.skill, i32::try_from(riding).unwrap());
                assert_eq!(order.days, 0);
                assert_eq!(order.level, -1);
            }
            other => panic!("expected STUDY order, got {other:?}"),
        }
        let unknown_skill = match &game.unit(unknown_skill_unit).unwrap().orders.month_orders {
            Some(OrderData::Study(order)) => order,
            other => panic!("expected STUDY order, got {other:?}"),
        };
        assert_eq!(unknown_skill.skill, -1);
        assert_eq!(unknown_skill.level, -1);
    }

    #[test]
    fn teach_requires_students_and_appends_to_matching_month_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nmove north\nteach 11\nteach 12\nteach\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): TEACH: Overwriting previous month-long order.",
                "Unit (10): TEACH: No students given."
            ]
        );
        match &game.unit(unit_id).unwrap().orders.month_orders {
            Some(OrderData::Teach(order)) => assert_eq!(
                order.targets,
                [
                    crate::orders::OrderUnitId {
                        unitnum: 11,
                        alias: 0,
                        faction: 0
                    },
                    crate::orders::OrderUnitId {
                        unitnum: 12,
                        alias: 0,
                        faction: 0
                    }
                ]
            ),
            other => panic!("expected TEACH order, got {other:?}"),
        }
    }

    #[test]
    fn forget_requires_valid_skill_and_appends_orders() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let tables = ruleset.tables();
        let combat =
            i32::try_from(crate::defs::skills::parse_skill(&tables.skills.skills, "COMB").unwrap())
                .unwrap();
        let riding =
            i32::try_from(crate::defs::skills::parse_skill(&tables.skills.skills, "RIDI").unwrap())
                .unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nforget COMB\nforget RIDI\nforget\nforget not-a-skill\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): FORGET: No skill given.",
                "Unit (10): FORGET: Invalid skill."
            ]
        );
        assert_eq!(
            game.unit(unit_id)
                .unwrap()
                .orders
                .forget_orders
                .iter()
                .map(|order| order.skill)
                .collect::<Vec<_>>(),
            [combat, riding]
        );
    }

    #[test]
    fn attack_appends_targets_in_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, _, _, unit_id) = game_with_unit();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nattack 11 faction 8 new 3\nattack 12\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        let targets = &game
            .unit(unit_id)
            .unwrap()
            .orders
            .attack_orders
            .as_ref()
            .expect("ATTACK targets were stored")
            .targets;
        assert_eq!(
            targets,
            &[
                crate::orders::OrderUnitId {
                    unitnum: 11,
                    alias: 0,
                    faction: 0
                },
                crate::orders::OrderUnitId {
                    unitnum: 0,
                    alias: 3,
                    faction: 8
                },
                crate::orders::OrderUnitId {
                    unitnum: 12,
                    alias: 0,
                    faction: 0
                }
            ]
        );
    }

    #[test]
    fn evict_appends_targets_in_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, _, _, unit_id) = game_with_unit();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nevict 11\nevict 12\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        let targets = &game
            .unit(unit_id)
            .unwrap()
            .orders
            .evict_orders
            .as_ref()
            .expect("EVICT targets were stored")
            .targets;
        assert_eq!(
            targets,
            &[
                crate::orders::OrderUnitId {
                    unitnum: 11,
                    alias: 0,
                    faction: 0
                },
                crate::orders::OrderUnitId {
                    unitnum: 12,
                    alias: 0,
                    faction: 0
                }
            ]
        );
    }

    #[test]
    fn steal_requires_target_and_stealable_item() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let tables = ruleset.tables();
        let silver = i32::try_from(
            tables
                .item_families
                .parse_enabled_item("SILV")
                .expect("silver is enabled"),
        )
        .unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nsteal\nsteal 11\nsteal 11 not-an-item\nsteal 11 SILV\nsteal 12 LEAD\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): STEAL: No target given.",
                "Unit (10): STEAL: No item given.",
                "Unit (10): STEAL: Bad item given.",
                "Unit (10): STEAL: Can't steal that."
            ]
        );
        match game.unit(unit_id).unwrap().orders.steal_orders.as_ref() {
            Some(super::super::StealOrderData::Steal(order)) => {
                assert_eq!(
                    order.target,
                    Some(crate::orders::OrderUnitId {
                        unitnum: 11,
                        alias: 0,
                        faction: 0
                    })
                );
                assert_eq!(order.item, silver);
            }
            other => panic!("expected STEAL order, got {other:?}"),
        }
    }

    #[test]
    fn assassinate_requires_target_and_replaces_steal_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nsteal 11 SILV\nassassinate 0\nassassinate 12\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            ["Unit (10): ASSASSINATE: No target given."]
        );
        match game.unit(unit_id).unwrap().orders.steal_orders.as_ref() {
            Some(super::super::StealOrderData::Assassinate(order)) => {
                assert_eq!(
                    order.target,
                    Some(crate::orders::OrderUnitId {
                        unitnum: 12,
                        alias: 0,
                        faction: 0
                    })
                );
            }
            other => panic!("expected ASSASSINATE order, got {other:?}"),
        }
    }

    #[test]
    fn promote_requires_target_and_replaces_previous_target() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\npromote 0\npromote 11\npromote 12\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            ["Unit (10): PROMOTE: No target given."]
        );
        assert_eq!(
            game.unit(unit_id).unwrap().orders.promote,
            Some(crate::orders::OrderUnitId {
                unitnum: 12,
                alias: 0,
                faction: 0
            })
        );
    }

    #[test]
    fn destroy_sets_destroy_flag() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, _, _, unit_id) = game_with_unit();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\ndestroy\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        assert_eq!(game.unit(unit_id).unwrap().orders.destroy, 1);
    }

    #[test]
    fn enter_requires_object_number() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nenter\nenter 7\nenter invalid\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): ENTER: No object specified.",
                "Unit (10): ENTER: No object specified."
            ]
        );
        assert_eq!(game.unit(unit_id).unwrap().orders.enter, 7);
    }

    #[test]
    fn leave_does_not_override_enter() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, faction_id, object_id, entering_unit) = game_with_unit();
        let leaving_unit = game
            .add_unit(object_id, Unit::new(11, Some(faction_id), 1, 5))
            .unwrap();
        game.setup_unit_nums().unwrap();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nenter 7\nleave\nunit 11\nleave\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        assert_eq!(game.unit(entering_unit).unwrap().orders.enter, 7);
        assert_eq!(game.unit(leaving_unit).unwrap().orders.enter, -1);
    }

    #[test]
    fn check_mode_tracks_study_and_teach_overwrites() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut game, _, _, unit_id) = game_with_unit();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nstudy COMB\nstudy RIDI\nteach 11\nteach 12\nsteal 13 not-an-item\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();

        assert_eq!(
            result.errors,
            [
                "STUDY: Overwriting previous month-long order.",
                "TEACH: Overwriting previous month-long order."
            ]
        );
        assert!(game.unit(unit_id).unwrap().orders.month_orders.is_none());
    }

    #[test]
    fn buy_and_sell_parse_all_quantities_aliases_and_errors() {
        let (mut game, _, object_id, unit_id) = game_with_unit();
        let region_id = game.object(object_id).unwrap().region.unwrap();
        let race = ItemId::Tribesman.index() as i32;
        game.region_mut(region_id).unwrap().race = race;
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              buy 1 peasant\nbuy 2 peasants\nbuy 3 peas\nbuy ALL WOOD\n\
              sell 4 iron\nsell all horse\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        let unit = game.unit(unit_id).unwrap();
        assert_eq!(
            unit.orders.buy_orders,
            [
                BuyOrder {
                    item: race,
                    num: 1,
                    ..BuyOrder::default()
                },
                BuyOrder {
                    item: race,
                    num: 2,
                    ..BuyOrder::default()
                },
                BuyOrder {
                    item: race,
                    num: 3,
                    ..BuyOrder::default()
                },
                BuyOrder {
                    item: ItemId::Wood.index() as i32,
                    num: -1,
                    ..BuyOrder::default()
                }
            ]
        );
        assert_eq!(
            unit.orders.sell_orders,
            [
                SellOrder {
                    item: ItemId::Iron.index() as i32,
                    num: 4,
                    ..SellOrder::default()
                },
                SellOrder {
                    item: ItemId::Horse.index() as i32,
                    num: -1,
                    ..SellOrder::default()
                }
            ]
        );

        let (mut check_game, _, _, check_unit) = game_with_unit();
        let check = parse_order_file(
            &mut check_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              buy 0 wood\nbuy 1\nsell\nsell 1\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(
            check.errors,
            [
                "BUY: Number to buy not given.",
                "BUY: Item not given.",
                "SELL: Number to sell not given.",
                "SELL: Item not given."
            ]
        );
        assert!(check_game
            .unit(check_unit)
            .unwrap()
            .orders
            .buy_orders
            .is_empty());
        assert!(check_game
            .unit(check_unit)
            .unwrap()
            .orders
            .sell_orders
            .is_empty());
    }

    #[test]
    fn give_take_and_withdraw_parse_amount_classes_exceptions_and_errors() {
        let (mut game, _, _, unit_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              give 20 2 wood\ngive 21 unit\ngive 22 all wood except 1\n\
              take from 23 all wood\nwithdraw wood\nwithdraw 3 iron\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        let unit = game.unit(unit_id).unwrap();
        assert_eq!(
            unit.orders.give_orders,
            [
                GiveOrder {
                    item: ItemId::Wood.index() as i32,
                    amount: 2,
                    target: Some(OrderUnitId {
                        unitnum: 20,
                        ..OrderUnitId::default()
                    }),
                    ..GiveOrder::default()
                },
                GiveOrder {
                    item: -1,
                    amount: -1,
                    target: Some(OrderUnitId {
                        unitnum: 21,
                        ..OrderUnitId::default()
                    }),
                    ..GiveOrder::default()
                },
                GiveOrder {
                    item: ItemId::Wood.index() as i32,
                    amount: -2,
                    except: 1,
                    target: Some(OrderUnitId {
                        unitnum: 22,
                        ..OrderUnitId::default()
                    }),
                    ..GiveOrder::default()
                },
                GiveOrder {
                    order: Order::new(OrderVerb::Take),
                    item: ItemId::Wood.index() as i32,
                    amount: -2,
                    target: Some(OrderUnitId {
                        unitnum: 23,
                        ..OrderUnitId::default()
                    }),
                    ..GiveOrder::take()
                }
            ]
        );
        assert_eq!(
            unit.orders.withdraw_orders,
            [
                WithdrawOrder {
                    item: ItemId::Wood.index() as i32,
                    amount: 1,
                    ..WithdrawOrder::default()
                },
                WithdrawOrder {
                    item: ItemId::Iron.index() as i32,
                    amount: 3,
                    ..WithdrawOrder::default()
                }
            ]
        );
    }

    #[test]
    fn give_all_item_classes_accept_cpp_aliases() {
        let (mut game, _, _, unit_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let classes = [
            ("normal", -crate::defs::ItemTypeFlags::NORMAL),
            ("advanced", -crate::defs::ItemTypeFlags::ADVANCED),
            ("trade", -crate::defs::ItemTypeFlags::TRADE),
            ("man", -crate::defs::ItemTypeFlags::MAN),
            ("men", -crate::defs::ItemTypeFlags::MAN),
            ("monster", -crate::defs::ItemTypeFlags::MONSTER),
            ("monsters", -crate::defs::ItemTypeFlags::MONSTER),
            ("magic", -crate::defs::ItemTypeFlags::MAGIC),
            ("weapon", -crate::defs::ItemTypeFlags::WEAPON),
            ("weapons", -crate::defs::ItemTypeFlags::WEAPON),
            ("armor", -crate::defs::ItemTypeFlags::ARMOR),
            ("mount", -crate::defs::ItemTypeFlags::MOUNT),
            ("mounts", -crate::defs::ItemTypeFlags::MOUNT),
            ("battle", -crate::defs::ItemTypeFlags::BATTLE),
            ("special", -crate::defs::ItemTypeFlags::SPECIAL),
            ("food", -crate::defs::ItemTypeFlags::FOOD),
            ("tool", -crate::defs::ItemTypeFlags::TOOL),
            ("tools", -crate::defs::ItemTypeFlags::TOOL),
            ("item", -(crate::defs::NITEMS as i32)),
            ("items", -(crate::defs::NITEMS as i32)),
            ("ship", -crate::defs::ItemTypeFlags::SHIP),
            ("ships", -crate::defs::ItemTypeFlags::SHIP),
        ];
        let mut input = String::from("#atlantis 7 none\nunit 10\n");
        for (index, (token, _)) in classes.iter().enumerate() {
            input.push_str(&format!("give {} ALL {token}\n", 200 + index));
        }
        input.push_str("#end\n");

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            input.as_bytes(),
            OrderParseMode::Turn,
        )
        .unwrap();

        let expected = classes
            .iter()
            .enumerate()
            .map(|(index, (_, item))| GiveOrder {
                item: *item,
                amount: -2,
                target: Some(OrderUnitId {
                    unitnum: 200 + index as i32,
                    ..OrderUnitId::default()
                }),
                ..GiveOrder::default()
            })
            .collect::<Vec<_>>();
        assert_eq!(game.unit(unit_id).unwrap().orders.give_orders, expected);
    }

    #[test]
    fn exchange_transport_distribute_claim_and_join_parse_valid_and_invalid_forms() {
        let (mut game, faction_id, object_id, unit_id) = game_with_unit();
        let no_overload_unit = game
            .add_unit(object_id, Unit::new(11, Some(faction_id), 0, 0))
            .unwrap();
        game.setup_unit_nums().unwrap();
        game.faction_mut(faction_id).unwrap().unclaimed = 50;
        game.unit_mut(unit_id)
            .unwrap()
            .items
            .set_num(ItemId::Silver, 10);
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              exchange 30 2 wood 3 iron\n\
              transport 31 all wood except 4\n\
              distribute 32 5 iron\n\
              join 33 nooverload\njoin 34 merge\nclaim 20\nclaim 80\n\
              unit 11\njoin 35 nooverload\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        let unit = game.unit(unit_id).unwrap();
        assert_eq!(
            unit.orders.exchange_orders,
            [ExchangeOrder {
                give_item: ItemId::Wood.index() as i32,
                give_amount: 2,
                expect_item: ItemId::Iron.index() as i32,
                expect_amount: 3,
                target: Some(OrderUnitId {
                    unitnum: 30,
                    ..OrderUnitId::default()
                }),
                ..ExchangeOrder::default()
            }]
        );
        assert_eq!(
            unit.orders.transport_orders,
            [
                TransportOrder {
                    item: ItemId::Wood.index() as i32,
                    amount: -1,
                    except: 4,
                    target: Some(OrderUnitId {
                        unitnum: 31,
                        ..OrderUnitId::default()
                    }),
                    ..TransportOrder::default()
                },
                TransportOrder {
                    order: Order::new(OrderVerb::Distribute),
                    item: ItemId::Iron.index() as i32,
                    amount: 5,
                    target: Some(OrderUnitId {
                        unitnum: 32,
                        ..OrderUnitId::default()
                    }),
                    ..TransportOrder::distribute()
                }
            ]
        );
        assert_eq!(
            unit.orders.join_orders,
            Some(JoinOrder {
                target: Some(OrderUnitId {
                    unitnum: 34,
                    ..OrderUnitId::default()
                }),
                overload: 1,
                merge: 1,
                ..JoinOrder::default()
            })
        );
        assert_eq!(
            game.unit(no_overload_unit).unwrap().orders.join_orders,
            Some(JoinOrder {
                target: Some(OrderUnitId {
                    unitnum: 35,
                    ..OrderUnitId::default()
                }),
                overload: 0,
                ..JoinOrder::default()
            })
        );
        assert_eq!(game.faction(faction_id).unwrap().unclaimed, 0);
        assert_eq!(
            game.unit(unit_id).unwrap().items.get_num(ItemId::Silver),
            60
        );
        assert_eq!(
            game.faction(faction_id).unwrap().events,
            ["Unit (10): Claims $20.", "Unit (10): Claims $30."]
        );
        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            ["Unit (10): CLAIM: Don't have that much unclaimed silver."]
        );
        assert_eq!(
            game.faction(faction_id)
                .unwrap()
                .items
                .get_num(ItemId::Silver),
            2
        );
        assert_eq!(
            game.faction(faction_id).unwrap().item_shows,
            [crate::model::FactionItemShow {
                item_type: ItemId::Silver,
                full: true
            }]
        );

        let tables = ruleset.tables();
        let ship = tables
            .item_families
            .items
            .iter()
            .find(|item| {
                item.item_type & crate::defs::ItemTypeFlags::SHIP != 0
                    && item.flags & crate::defs::ItemFlags::DISABLED == 0
            })
            .and_then(|item| item.abbreviation.as_deref())
            .unwrap();
        let mut check_game = game_with_unit().0;
        let input = format!(
            "#atlantis 7 none\nunit 10\n\
             exchange\nexchange 30\nexchange 30 1\n\
             exchange 30 1 {ship} 1 wood\nexchange 30 1 wood\n\
             exchange 30 1 wood 1\nexchange 30 1 wood 1 {ship}\n\
             transport\ntransport 31\ntransport 31 all\n\
             transport 31 all wood except\ntransport 31 all wood except 0\n\
             transport 31 all fake\ndistribute 32 all fake\n\
             distribute 32 all wood except\ndistribute 32 all wood except 0\n\
             join 0\nclaim\n#end\n"
        );
        let check = parse_order_file(
            &mut check_game,
            ruleset.game_defs(),
            input.as_bytes(),
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(
            check.errors,
            [
                "EXCHANGE: Invalid target.",
                "EXCHANGE: No amount given.",
                "EXCHANGE: No item given.",
                "EXCHANGE: Can't exchange ships.",
                "EXCHANGE: No amount expected.",
                "EXCHANGE: No item expected.",
                "EXCHANGE: Can't exchange ships.",
                "TRANSPORT: Invalid target.",
                "TRANSPORT: No amount given.",
                "TRANSPORT: No item given.",
                "TRANSPORT: EXCEPT requires a value.",
                "TRANSPORT: Invalid except value.",
                "TRANSPORT: Invalid item.",
                "DISTRIBUTE: Invalid item fake.",
                "DISTRIBUTE: EXCEPT requires a value.",
                "DISTRIBUTE: Invalid except value.",
                "JOIN: No target given.",
                "CLAIM: No amount given."
            ]
        );
    }

    #[test]
    fn give_take_and_withdraw_reject_invalid_forms() {
        let (mut game, _, _, unit_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let check = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              give 20 0 wood\ngive\ntake 20 1 wood\ntake from 0 1 wood\n\
              give 20\ngive 20 2\ngive 20 1 fake\ngive 20 all fake\n\
              give 20 all weapon except 1\ngive 20 1 wood except 1\n\
              give 20 all wood except\ngive 20 all wood except 0\n\
              give 20 1 unfinished wood\ntake from 20 unit\n\
              withdraw\nwithdraw 1\nwithdraw 1 silver\n\
              exchange 30 1 wood 1\ntransport 31 2 fake\n\
              distribute 32 2 fake\njoin 0\nclaim 0\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(
            check.errors,
            [
                "GIVE: Illegal amount given.",
                "GIVE: Invalid target.",
                "TAKE: Missing FROM.",
                "TAKE: Invalid target.",
                "GIVE: No amount given.",
                "GIVE: No item given.",
                "GIVE: Invalid item.",
                "GIVE: Invalid item or item class.",
                "GIVE: EXCEPT only valid with specific items.",
                "GIVE: EXCEPT only valid with ALL",
                "GIVE: EXCEPT requires a value.",
                "GIVE: Invalid EXCEPT value.",
                "GIVE: That item does not have an unfinished version.",
                "TAKE: Illegal amount given.",
                "WITHDRAW: No amount given.",
                "WITHDRAW: No item given.",
                "WITHDRAW: Invalid item.",
                "EXCHANGE: No item expected.",
                "TRANSPORT: Invalid item.",
                "DISTRIBUTE: Invalid item fake.",
                "JOIN: No target given.",
                "CLAIM: No amount given."
            ]
        );
        let unit = game.unit(unit_id).unwrap();
        assert!(unit.orders.give_orders.is_empty());
        assert!(unit.orders.withdraw_orders.is_empty());
        assert!(unit.orders.exchange_orders.is_empty());
        assert!(unit.orders.transport_orders.is_empty());
        assert_eq!(unit.orders.join_orders, None);
        assert_eq!(unit.items.get_num(ItemId::Silver), 0);
    }

    #[test]
    fn withdraw_rejects_orders_when_disabled() {
        let (mut game, _, _, _) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let mut defs = *ruleset.game_defs();
        defs.allow_withdraw = 0;
        let check = parse_order_file(
            &mut game,
            &defs,
            b"#atlantis 7 none\nunit 10\nwithdraw wood\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(check.errors, ["WITHDRAW is not a valid order."]);
    }

    #[test]
    fn production_and_tax_orders_match_month_long_replacement_rules() {
        let standard = <dyn Ruleset>::from_name("standard").unwrap();
        let standard_tables = standard.tables();
        let entertainment_skill = standard_tables
            .skills
            .skills
            .iter()
            .position(|skill| skill.name == Some("entertainment"))
            .unwrap() as i32;
        let enabled_ship = standard_tables
            .item_families
            .items
            .iter()
            .find(|item| {
                item.item_type == crate::defs::ItemTypeFlags::SHIP
                    && item.flags & crate::defs::ItemFlags::DISABLED == 0
            })
            .and_then(|item| item.name.as_deref())
            .unwrap()
            .to_owned();
        let (mut standard_game, faction_id, _, unit_id) = game_with_unit();

        parse_order_file(
            &mut standard_game,
            standard.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              produce 2 wood\nwork\nentertain\ntax\npillage\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            standard_game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): WORK: Overwriting previous month-long order.",
                "Unit (10): ENTERTAIN: Overwriting previous month-long order.",
                "Unit (10): PILLAGE: The unit is already taxing."
            ]
        );
        assert_eq!(
            standard_game.unit(unit_id).unwrap().orders.month_orders,
            Some(OrderData::Produce(ProduceOrder {
                order: Order::new(OrderVerb::Produce),
                item: ItemId::Silver.index() as i32,
                skill: entertainment_skill,
                target: 0,
                ..ProduceOrder::default()
            }))
        );
        assert_eq!(standard_game.unit(unit_id).unwrap().taxing, 1);

        let (mut neworigins_game, neworigins_faction, _, neworigins_unit) = game_with_unit();
        let neworigins = <dyn Ruleset>::from_name("neworigins").unwrap();
        let neworigins_tables = neworigins.tables();
        let iron_skill = neworigins_tables.item_families.items[ItemId::Iron.index()]
            .production_skill
            .as_deref()
            .and_then(|skill| {
                crate::defs::skills::lookup_skill(&neworigins_tables.skills.skills, skill)
            })
            .map_or(-1, |skill| skill as i32);
        parse_order_file(
            &mut neworigins_game,
            neworigins.game_defs(),
            b"#atlantis 7 none\nunit 10\nproduce 1 wood\npillage\nproduce 3 iron\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(
            neworigins_game.faction(neworigins_faction).unwrap().errors,
            [
                "Unit (10): PILLAGE: Overwriting previous month-long order.",
                "Unit (10): PRODUCE: Overwriting previous month-long order."
            ]
        );
        assert_eq!(
            neworigins_game
                .unit(neworigins_unit)
                .unwrap()
                .orders
                .month_orders,
            Some(OrderData::Produce(ProduceOrder {
                item: ItemId::Iron.index() as i32,
                skill: iron_skill,
                target: 3,
                ..ProduceOrder::default()
            }))
        );
        assert_eq!(neworigins_game.unit(neworigins_unit).unwrap().taxing, 0);

        let (mut invalid_game, _, _, invalid_unit) = game_with_unit();
        let invalid_orders = format!(
            "#atlantis 7 none\nunit 10\nproduce\nproduce 1\n\
             produce 1 nonexistent\nproduce 1 {enabled_ship}\n#end\n"
        );
        let invalid = parse_order_file(
            &mut invalid_game,
            standard.game_defs(),
            invalid_orders.as_bytes(),
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(
            invalid.errors,
            [
                "PRODUCE: No item given.",
                "PRODUCE: No item given.",
                "PRODUCE: Invalid item.",
                "PRODUCE: Use BUILD for ships."
            ]
        );
        assert!(invalid_game
            .unit(invalid_unit)
            .unwrap()
            .orders
            .month_orders
            .is_none());

        let (mut check_game, check_faction, _, check_unit) = game_with_unit();
        let mut parser =
            super::Parser::new(&mut check_game, standard.game_defs(), OrderParseMode::Check);
        parser.faction_id = Some(check_faction);
        parser.unit_stack.push(check_unit);
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        for line in [
            "produce 2 iron",
            "turn",
            "produce 1 wood",
            "work",
            "endturn",
        ] {
            parser
                .parse_line(line.to_owned(), &[], &mut cursor, &mut pending)
                .unwrap();
        }
        assert_eq!(
            parser.result.errors,
            ["WORK: Overwriting previous DELAYED month-long order."]
        );
        assert_eq!(
            parser.game.unit(check_unit).unwrap().orders.month_orders,
            Some(OrderData::Produce(ProduceOrder {
                item: ItemId::Iron.index() as i32,
                skill: standard_tables.item_families.items[ItemId::Iron.index()]
                    .production_skill
                    .as_deref()
                    .and_then(|skill| {
                        crate::defs::skills::lookup_skill(&standard_tables.skills.skills, skill)
                    })
                    .map_or(-1, |skill| skill as i32),
                target: 2,
                ..ProduceOrder::default()
            }))
        );
    }

    #[test]
    fn build_orders_parse_build_help_objects_ships_and_errors() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let tables = ruleset.tables();
        let tower = crate::defs::objects::lookup_object("tower", &tables.objects).unwrap();
        let (mut building_game, building_faction, _, building_unit) = game_with_unit();
        let region_id = building_game
            .unit(building_unit)
            .and_then(|unit| unit.object)
            .and_then(|object_id| building_game.object(object_id))
            .and_then(|object| object.region)
            .unwrap();
        building_game.region_mut(region_id).unwrap().terrain_type =
            Some(crate::defs::terrains::R_PLAIN);
        parse_order_file(
            &mut building_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nbuild tower\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert!(building_game
            .faction(building_faction)
            .unwrap()
            .errors
            .is_empty());
        assert_eq!(
            building_game
                .unit(building_unit)
                .unwrap()
                .orders
                .month_orders,
            Some(OrderData::Build(BuildOrder {
                new_building: tower as i32,
                ..BuildOrder::default()
            }))
        );
        assert_eq!(building_game.unit(building_unit).unwrap().orders.build, 0);

        let (mut help_game, _, _, help_unit) = game_with_unit();
        parse_order_file(
            &mut help_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nbuild help new 44\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert_eq!(
            help_game.unit(help_unit).unwrap().orders.month_orders,
            Some(OrderData::Build(BuildOrder {
                target: Some(OrderUnitId {
                    alias: 44,
                    ..OrderUnitId::default()
                }),
                ..BuildOrder::default()
            }))
        );

        let ships = tables
            .item_families
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.item_type & crate::defs::ItemTypeFlags::SHIP != 0
                    && item.flags & crate::defs::ItemFlags::DISABLED == 0
            })
            .map(|(index, _)| index)
            .take(2)
            .collect::<Vec<_>>();
        assert_eq!(ships.len(), 2);
        let (mut ship_game, _, _, ship_unit) = game_with_unit();
        for (index, ship) in ships.iter().enumerate() {
            ship_game
                .unit_mut(ship_unit)
                .unwrap()
                .items
                .set_num(ItemId::from_index(*ship).unwrap(), (index + 2) as i32);
        }
        parse_order_file(
            &mut ship_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nbuild\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert_eq!(
            ship_game.unit(ship_unit).unwrap().orders.build,
            -(ships[0] as i32)
        );
        assert_eq!(
            ship_game.unit(ship_unit).unwrap().orders.month_orders,
            Some(OrderData::Build(BuildOrder {
                needtocomplete: 2,
                ..BuildOrder::default()
            }))
        );

        let (mut invalid_game, invalid_faction, _, _) = game_with_unit();
        parse_order_file(
            &mut invalid_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n\
              build help\nbuild help 0\nbuild no_such_object\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert_eq!(
            invalid_game.faction(invalid_faction).unwrap().errors,
            [
                "Unit (10): BUILD: Non-existent unit to help.",
                "Unit (10): BUILD: Non-existent unit to help.",
                "Unit (10): BUILD: Not a valid object name."
            ]
        );

        let ship_name = tables.item_families.items[ships[0]]
            .name
            .as_deref()
            .unwrap()
            .to_owned();
        let (mut check_game, check_faction, _, check_unit) = game_with_unit();
        let mut parser =
            super::Parser::new(&mut check_game, ruleset.game_defs(), OrderParseMode::Check);
        parser.faction_id = Some(check_faction);
        parser.unit_stack.push(check_unit);
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        parser
            .parse_line(format!("build {ship_name}"), &[], &mut cursor, &mut pending)
            .unwrap();
        assert!(parser.result.errors.is_empty());
    }

    #[test]
    fn default_work_runs_after_each_real_order_file() {
        let (mut game, faction_id, work_unit, other_work_unit, ordered_unit, npc_unit, nexus_unit) =
            default_work_game();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        let work = match &game.unit(work_unit).unwrap().orders.month_orders {
            Some(OrderData::Produce(order)) => order,
            other => panic!("expected default WORK order, got {other:?}"),
        };
        assert_eq!(work.item, ItemId::Silver.index() as i32);
        assert_eq!(work.skill, -1);
        assert_eq!(work.target, 0);
        assert_eq!(work.order.quiet, 1);
        assert!(matches!(
            game.unit(other_work_unit).unwrap().orders.month_orders,
            Some(OrderData::Produce(_))
        ));
        assert!(matches!(
            game.unit(ordered_unit).unwrap().orders.month_orders,
            Some(OrderData::Move(_))
        ));
        assert!(game.unit(npc_unit).unwrap().orders.month_orders.is_none());
        assert!(game.unit(nexus_unit).unwrap().orders.month_orders.is_none());
        assert!(game.faction(faction_id).unwrap().errors.is_empty());

        let (mut check_game, _, check_unit, _, _, _, _) = default_work_game();
        parse_order_file(
            &mut check_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert!(check_game
            .unit(check_unit)
            .unwrap()
            .orders
            .month_orders
            .is_none());

        let (mut disabled_game, _, disabled_unit, _, _, _, _) = default_work_game();
        let mut defs = *ruleset.game_defs();
        defs.default_work_order = 0;
        parse_order_file(
            &mut disabled_game,
            &defs,
            b"#atlantis 7 none\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert!(disabled_game
            .unit(disabled_unit)
            .unwrap()
            .orders
            .month_orders
            .is_none());
    }

    #[test]
    fn autotax_uses_month_long_taxing_only_for_units_that_can_tax() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").unwrap();
        let tables = ruleset.tables();
        let combat = crate::defs::skills::lookup_skill(&tables.skills.skills, "COMB").unwrap();
        let mut game = Game::default();
        let faction_id = game.add_faction(Faction::new(7, &[]));
        let region_id = game.add_region(Region::default());
        let object_id = game.add_object(region_id, Object::default()).unwrap();

        let mut taxer = Unit::new(10, Some(faction_id), 0, 5);
        taxer.flags |= super::FLAG_AUTOTAX;
        taxer.items.set_num(ItemId::Leaders, 1);
        taxer.skills.set_days(combat, 30);
        let taxer_id = game.add_unit(object_id, taxer).unwrap();

        let mut non_taxer = Unit::new(11, Some(faction_id), 0, 5);
        non_taxer.flags |= super::FLAG_AUTOTAX;
        let non_taxer_id = game.add_unit(object_id, non_taxer).unwrap();

        let mut already_taxing = Unit::new(12, Some(faction_id), 0, 5);
        already_taxing.taxing = 1;
        let already_taxing_id = game.add_unit(object_id, already_taxing).unwrap();
        game.setup_unit_nums().unwrap();

        parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert_eq!(game.unit(taxer_id).unwrap().taxing, super::TAX_AUTO);
        assert!(game.unit(taxer_id).unwrap().orders.month_orders.is_none());
        assert!(matches!(
            game.unit(non_taxer_id).unwrap().orders.month_orders,
            Some(OrderData::Produce(_))
        ));
        assert_eq!(game.unit(already_taxing_id).unwrap().taxing, 1);
        assert!(game
            .unit(already_taxing_id)
            .unwrap()
            .orders
            .month_orders
            .is_none());
    }

    #[test]
    fn trade_and_transfer_orders_parse_committed_fixture_inputs() {
        macro_rules! fixture {
            ($ruleset:literal, $turn:literal) => {
                (
                    $ruleset,
                    $turn,
                    include_str!(concat!(
                        "../../../../fixtures/",
                        $ruleset,
                        "/turn_",
                        $turn,
                        "/game.in"
                    )),
                    include_str!(concat!(
                        "../../../../fixtures/",
                        $ruleset,
                        "/turn_",
                        $turn,
                        "/orders.3"
                    )),
                    include_str!(concat!(
                        "../../../../fixtures/",
                        $ruleset,
                        "/turn_",
                        $turn,
                        "/report.3"
                    )),
                )
            };
        }

        let fixtures = [
            fixture!("standard", "0"),
            fixture!("standard", "1"),
            fixture!("standard", "2"),
            fixture!("standard", "3"),
            fixture!("standard", "4"),
            fixture!("standard", "5"),
            fixture!("standard", "6"),
            fixture!("standard", "7"),
            fixture!("standard", "8"),
            fixture!("standard", "9"),
            fixture!("standard", "10"),
            fixture!("standard", "11"),
            fixture!("standard", "12"),
            fixture!("standard", "13"),
            fixture!("neworigins", "0"),
            fixture!("neworigins", "1"),
            fixture!("neworigins", "2"),
            fixture!("neworigins", "3"),
            fixture!("neworigins", "4"),
            fixture!("neworigins", "5"),
            fixture!("neworigins", "6"),
            fixture!("neworigins", "7"),
            fixture!("neworigins", "8"),
            fixture!("neworigins", "9"),
            fixture!("neworigins", "10"),
            fixture!("neworigins", "11"),
            fixture!("neworigins", "12"),
            fixture!("neworigins", "13"),
        ];

        for (ruleset_name, turn, game_input, orders_input, expected_report) in fixtures {
            let mut loaded = crate::save::load_legacy(game_input)
                .unwrap_or_else(|error| panic!("{ruleset_name} turn {turn}: {error}"));
            let original_errors = loaded
                .game
                .faction_ids()
                .iter()
                .copied()
                .filter_map(|faction_id| {
                    loaded
                        .game
                        .faction(faction_id)
                        .map(|faction| (faction_id, faction.num, faction.errors.len()))
                })
                .collect::<Vec<_>>();
            let defs = loaded.ruleset.game_defs();

            parse_order_file(
                &mut loaded.game,
                defs,
                orders_input.as_bytes(),
                OrderParseMode::Turn,
            )
            .unwrap_or_else(|error| panic!("{ruleset_name} turn {turn}: {error}"));

            for (faction_id, faction_number, error_count) in original_errors {
                let faction = loaded.game.faction(faction_id).unwrap();
                let new_errors = &faction.errors[error_count..];
                for error in new_errors {
                    assert!(
                        expected_report.lines().any(|line| line == error),
                        "unexpected parser error for faction {faction_number} in \
                         {ruleset_name} turn {turn}: {error:?}"
                    );
                }
            }

            match (ruleset_name, turn) {
                ("standard", "9") => {
                    assert_eq!(
                        loaded.game.get_unit(178).unwrap().orders.give_orders,
                        [
                            GiveOrder {
                                item: ItemId::HighElf.index() as i32,
                                amount: 22,
                                target: Some(OrderUnitId {
                                    unitnum: 167,
                                    ..OrderUnitId::default()
                                }),
                                ..GiveOrder::default()
                            },
                            GiveOrder {
                                item: ItemId::Silver.index() as i32,
                                amount: -2,
                                target: Some(OrderUnitId {
                                    unitnum: 167,
                                    ..OrderUnitId::default()
                                }),
                                ..GiveOrder::default()
                            }
                        ]
                    );
                }
                ("standard", "13") => {
                    assert_eq!(
                        loaded.game.get_unit(164).unwrap().orders.give_orders,
                        [GiveOrder {
                            order: Order::new(OrderVerb::Take),
                            item: ItemId::Silver.index() as i32,
                            amount: -2,
                            target: Some(OrderUnitId {
                                unitnum: 167,
                                ..OrderUnitId::default()
                            }),
                            ..GiveOrder::take()
                        }]
                    );
                }
                ("neworigins", "4") => {
                    assert_eq!(
                        loaded.game.get_unit(52).unwrap().orders.withdraw_orders,
                        [WithdrawOrder {
                            item: ItemId::Longbow.index() as i32,
                            amount: 1,
                            ..WithdrawOrder::default()
                        }]
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn atlantis_blocks_select_line_faction_and_reject_bad_password_until_next_block() {
        let mut game = Game::default();
        game.year = 3;
        game.month = 4;
        let first = game.add_faction(Faction::new(7, &[]));
        let second = game.add_faction(Faction::new(8, &[]));
        game.faction_mut(first).unwrap().password = Some("red fox".to_owned());
        game.faction_mut(second).unwrap().password = Some("blue fox".to_owned());

        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 RED_FOX\n#end\n\
              #atlantis 8 wrong\nthis is ignored\n\
              #atlantis 7 red_fox\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        assert_eq!(game.faction(first).unwrap().last_orders, 29);
        assert_eq!(
            game.faction(second).unwrap().errors,
            ["Incorrect password on #atlantis line."]
        );
    }

    #[test]
    fn unit_and_end_directives_unwind_form_stack_with_original_errors() {
        let (mut game, faction_id, object_id, root_id) = game_with_unit();
        game.unit_mut(root_id).unwrap().orders.enter = 4;
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();

        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n@form 3\n@; child line\nform 4\n@end\nunit 999\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        assert!(result.errors.is_empty());
        assert_eq!(game.unit(root_id).unwrap().orders.enter, 0);
        assert_eq!(game.unit(root_id).unwrap().format, 1);
        assert_eq!(game.unit(root_id).unwrap().old_orders, ["@form 3", "@end"]);
        let form_id = game.object(object_id).unwrap().unit_ids()[1];
        let form = game.unit(form_id).unwrap();
        assert_eq!(form.alias, 3);
        assert_eq!(form.former, Some(root_id));
        assert_eq!(form.old_orders, ["@; child line"]);
        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            ["Unit (1): FORM: cannot nest.", "999 is not your unit."]
        );
    }

    #[test]
    fn check_mode_form_month_orders_do_not_leak_after_end() {
        let (mut game, _, object_id, root_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nform 1\nwork\nend\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert!(result.errors.is_empty());

        let form_id = game
            .object(object_id)
            .unwrap()
            .unit_ids()
            .iter()
            .copied()
            .find(|unit_id| game.unit(*unit_id).unwrap().former == Some(root_id))
            .unwrap();
        let form = game.unit(form_id).unwrap();
        assert_eq!(form.orders.month_orders, None);
        assert_eq!(form.taxing, 0);
    }

    #[test]
    fn turn_blocks_capture_raw_lines_in_run_mode_and_parse_normally_in_check_mode() {
        let (mut game, _faction_id, object_id, root_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\n@turn\nmove north\nform 3\nmove east\nend\nendturn\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        let orders = &game.unit(root_id).unwrap().orders;
        assert_eq!(orders.turn_orders.len(), 1);
        assert_eq!(
            orders.turn_orders[0],
            TurnOrder {
                repeating: 1,
                turn_orders: ["move north", "form 3", "move east", "end"]
                    .map(str::to_owned)
                    .to_vec(),
                ..TurnOrder::default()
            }
        );
        assert!(result.errors.is_empty());
        assert_eq!(game.object(object_id).unwrap().unit_ids().len(), 1);

        let (mut check_game, _, _, _) = game_with_unit();
        let check = parse_order_file(
            &mut check_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nturn\nturn\nmove north\nendturn\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(check.errors, ["TURN: cannot nest"]);
        assert_eq!(
            check
                .lines
                .iter()
                .map(|line| line.indent)
                .collect::<Vec<_>>(),
            [0, 0, 0, 1, 2, 1, 0]
        );

        let (mut restore_game, restore_faction, _, restore_unit) = game_with_unit();
        let saved_month_order = OrderData::Move(crate::orders::MoveOrder::default());
        restore_game
            .unit_mut(restore_unit)
            .unwrap()
            .orders
            .month_orders = Some(saved_month_order.clone());
        restore_game.unit_mut(restore_unit).unwrap().taxing = 5;
        let tables = ruleset.tables();
        let mut parser = super::Parser::new(
            &mut restore_game,
            ruleset.game_defs(),
            &tables,
            OrderParseMode::Check,
        );
        parser.faction_id = Some(restore_faction);
        parser.unit_stack.push(restore_unit);
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        parser
            .parse_line("turn".to_owned(), &[], &mut cursor, &mut pending)
            .unwrap();
        assert_eq!(
            parser
                .game
                .unit(restore_unit)
                .unwrap()
                .orders
                .present_month_orders,
            Some(saved_month_order.clone())
        );
        parser
            .parse_line("endturn".to_owned(), &[], &mut cursor, &mut pending)
            .unwrap();
        let restored = parser.game.unit(restore_unit).unwrap();
        assert_eq!(restored.orders.month_orders, Some(saved_month_order));
        assert_eq!(restored.orders.present_month_orders, None);
        assert_eq!(restored.taxing, 5);
        assert_eq!(restored.present_taxing, 0);
        assert_eq!(restored.orders.in_turn_block, 0);

        let (mut sentinel_game, sentinel_faction, _, sentinel_unit) = game_with_unit();
        parse_order_file(
            &mut sentinel_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nturn\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert_eq!(
            sentinel_game.faction(sentinel_faction).unwrap().errors,
            ["Unit (10): TURN: without ENDTURN."]
        );
        assert_eq!(
            sentinel_game
                .unit(sentinel_unit)
                .unwrap()
                .orders
                .turn_orders
                .len(),
            1
        );
    }

    #[test]
    fn parse_errors_route_to_check_unit_or_faction_and_cap_faction_errors() {
        let (mut no_unit_game, no_unit_faction, _, _) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        parse_order_file(
            &mut no_unit_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunknown\nmove north\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert_eq!(
            no_unit_game.faction(no_unit_faction).unwrap().errors,
            [
                "unknown is not a valid order.",
                "Order given without a unit selected."
            ]
        );

        let (mut check_game, _, _, _) = game_with_unit();
        let check = parse_order_file(
            &mut check_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nunknown\n#end\n",
            OrderParseMode::Check,
        )
        .unwrap();
        assert_eq!(check.errors, ["unknown is not a valid order."]);
        assert!(check_game
            .faction(check_game.faction_ids()[0])
            .unwrap()
            .errors
            .is_empty());

        let (mut npc_game, npc_faction, _, _) = game_with_unit();
        npc_game
            .faction_mut(npc_faction)
            .unwrap()
            .faction_types
            .push(("War".to_owned(), -1));
        parse_order_file(
            &mut npc_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nunknown\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert!(npc_game.faction(npc_faction).unwrap().errors.is_empty());

        let (mut capped_game, capped_faction, _, _) = game_with_unit();
        let input = format!(
            "#atlantis 7 none\nunit 10\n{}#end\n",
            "unknown\n".repeat(1005)
        );
        parse_order_file(
            &mut capped_game,
            ruleset.game_defs(),
            input.as_bytes(),
            OrderParseMode::Turn,
        )
        .unwrap();
        let errors = &capped_game.faction(capped_faction).unwrap().errors;
        assert_eq!(errors.len(), 1002);
        assert_eq!(errors[1000], "Unit (10): unknown is not a valid order.");
        assert_eq!(errors[1001], "Too many errors!");
    }

    #[test]
    fn parse_dir_accepts_names_abbreviations_controls_and_object_numbers() {
        let (mut game, faction_id, _, unit_id) = game_with_unit();
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let result = parse_order_file(
            &mut game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nmove North n nOrThEaSt NE Southeast SE South s Southwest SW Northwest NW 12tail in out pause p\nmove 0 north\nmove northeast\nmove invalid south\nmove -100 north\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();

        let Some(OrderData::Move(order)) = &game.unit(unit_id).unwrap().orders.month_orders else {
            panic!("expected MOVE order");
        };
        assert_eq!(order.order.kind, OrderVerb::Move);
        assert_eq!(
            order.dirs,
            [
                0,
                0,
                1,
                1,
                2,
                2,
                3,
                3,
                4,
                4,
                5,
                5,
                MOVE_ENTER + 12,
                MOVE_IN,
                MOVE_OUT,
                MOVE_PAUSE,
                MOVE_PAUSE,
                1,
            ]
            .map(|dir| MoveDir { dir })
        );
        assert_eq!(
            game.faction(faction_id).unwrap().errors,
            [
                "Unit (10): MOVE: Warning, bad direction.",
                "Unit (10): MOVE: Warning, bad direction.",
                "Unit (10): MOVE: Warning, bad direction."
            ]
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn move_advance_and_sail_parse_accepted_and_rejected_directions() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let (mut advance_game, advance_faction, _, advance_unit) = game_with_unit();
        parse_order_file(
            &mut advance_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nadvance north 2 in out p invalid northwest\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        let Some(OrderData::Move(advance)) =
            &advance_game.unit(advance_unit).unwrap().orders.month_orders
        else {
            panic!("expected ADVANCE order");
        };
        assert_eq!(advance.order.kind, OrderVerb::Advance);
        assert_eq!(advance.advancing, 1);
        assert_eq!(
            advance.dirs,
            [0, MOVE_ENTER + 2, MOVE_IN, MOVE_OUT, MOVE_PAUSE].map(|dir| MoveDir { dir })
        );
        assert_eq!(
            advance_game.faction(advance_faction).unwrap().errors,
            ["Unit (10): ADVANCE: Warning, bad direction."]
        );

        let (mut sail_game, sail_faction, _, sail_unit) = game_with_unit();
        parse_order_file(
            &mut sail_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nsail north NE SE south SW northwest p in south\nsail out\nsail 12\nsail invalid\nsail -100 north\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        let Some(OrderData::Sail(sail)) = &sail_game.unit(sail_unit).unwrap().orders.month_orders
        else {
            panic!("expected SAIL order");
        };
        assert_eq!(
            sail.dirs,
            [0, 1, 2, 3, 4, 5, MOVE_PAUSE].map(|dir| MoveDir { dir })
        );
        assert_eq!(
            sail_game.faction(sail_faction).unwrap().errors,
            [
                "Unit (10): SAIL: Warning, bad direction.",
                "Unit (10): SAIL: Warning, bad direction.",
                "Unit (10): SAIL: Warning, bad direction.",
                "Unit (10): SAIL: Warning, bad direction.",
                "Unit (10): SAIL: Warning, bad direction."
            ]
        );

        let (mut check_game, check_faction, _, check_unit) = game_with_unit();
        let tables = ruleset.tables();
        let mut parser = super::Parser::new(
            &mut check_game,
            ruleset.game_defs(),
            &tables,
            OrderParseMode::Check,
        );
        parser.faction_id = Some(check_faction);
        parser.unit_stack.push(check_unit);
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        parser
            .parse_line("move north".to_owned(), &[], &mut cursor, &mut pending)
            .unwrap();
        assert!(matches!(
            &parser.game.unit(check_unit).unwrap().orders.month_orders,
            Some(OrderData::Move(order)) if order.dirs.is_empty()
        ));
    }

    #[test]
    fn movement_orders_append_and_overwrite_with_original_errors() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let tables = ruleset.tables();

        let (mut move_game, move_faction, _, move_unit) = game_with_unit();
        parse_order_file(
            &mut move_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nmove north\nmove NE\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        let Some(OrderData::Move(move_order)) =
            &move_game.unit(move_unit).unwrap().orders.month_orders
        else {
            panic!("expected MOVE order");
        };
        assert_eq!(move_order.dirs, [0, 1].map(|dir| MoveDir { dir }));
        assert!(move_game.faction(move_faction).unwrap().errors.is_empty());

        let (mut advance_game, advance_faction, _, advance_unit) = game_with_unit();
        parse_order_file(
            &mut advance_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nadvance north\nadvance south\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        let Some(OrderData::Move(advance_order)) =
            &advance_game.unit(advance_unit).unwrap().orders.month_orders
        else {
            panic!("expected ADVANCE order");
        };
        assert_eq!(advance_order.advancing, 1);
        assert_eq!(advance_order.dirs, [0, 3].map(|dir| MoveDir { dir }));
        assert!(advance_game
            .faction(advance_faction)
            .unwrap()
            .errors
            .is_empty());

        let (mut sail_game, sail_faction, _, sail_unit) = game_with_unit();
        parse_order_file(
            &mut sail_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nsail north\nsail south\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        let Some(OrderData::Sail(sail_order)) =
            &sail_game.unit(sail_unit).unwrap().orders.month_orders
        else {
            panic!("expected SAIL order");
        };
        assert_eq!(sail_order.dirs, [0, 3].map(|dir| MoveDir { dir }));
        assert!(sail_game.faction(sail_faction).unwrap().errors.is_empty());

        let (mut overwrite_game, overwrite_faction, _, overwrite_unit) = game_with_unit();
        parse_order_file(
            &mut overwrite_game,
            ruleset.game_defs(),
            b"#atlantis 7 none\nunit 10\nmove north\nadvance south\nsail northwest\n#end\n",
            OrderParseMode::Turn,
        )
        .unwrap();
        assert!(matches!(
            &overwrite_game
                .unit(overwrite_unit)
                .unwrap()
                .orders
                .month_orders,
            Some(OrderData::Sail(order)) if order.dirs == [MoveDir { dir: 5 }]
        ));
        assert_eq!(
            overwrite_game.faction(overwrite_faction).unwrap().errors,
            [
                "Unit (10): ADVANCE: Overwriting previous month-long orders",
                "Unit (10): SAIL: Overwriting previous month-long order."
            ]
        );

        let (mut delayed_game, delayed_faction, _, delayed_unit) = game_with_unit();
        delayed_game
            .unit_mut(delayed_unit)
            .unwrap()
            .orders
            .month_orders = Some(OrderData::Move(MoveOrder::default()));
        delayed_game
            .unit_mut(delayed_unit)
            .unwrap()
            .orders
            .in_turn_block = 1;
        let mut parser = super::Parser::new(
            &mut delayed_game,
            ruleset.game_defs(),
            &tables,
            OrderParseMode::Turn,
        );
        parser.faction_id = Some(delayed_faction);
        parser.unit_stack.push(delayed_unit);
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        parser
            .parse_line("sail north".to_owned(), &[], &mut cursor, &mut pending)
            .unwrap();
        assert_eq!(
            parser.game.faction(delayed_faction).unwrap().errors,
            ["Unit (10): SAIL: Overwriting previous DELAYED month-long order."]
        );

        let mut defs = *ruleset.game_defs();
        defs.tax_pillage_month_long = 1;
        let (mut taxed_game, taxed_faction, _, taxed_unit) = game_with_unit();
        let mut parser = super::Parser::new(&mut taxed_game, &defs, &tables, OrderParseMode::Turn);
        parser.faction_id = Some(taxed_faction);
        parser.unit_stack.push(taxed_unit);
        parser.game.unit_mut(taxed_unit).unwrap().taxing = 1;
        let mut cursor = 0;
        let mut pending = std::collections::VecDeque::new();
        parser
            .parse_line("move north".to_owned(), &[], &mut cursor, &mut pending)
            .unwrap();
        assert_eq!(parser.game.unit(taxed_unit).unwrap().taxing, 0);
        assert_eq!(
            parser.game.faction(taxed_faction).unwrap().errors,
            ["Unit (10): MOVE: Overwriting previous month-long order."]
        );
    }

    fn game_with_unit() -> (
        Game,
        crate::model::FactionId,
        crate::model::ObjectId,
        crate::model::UnitId,
    ) {
        let mut game = Game::default();
        let faction_id = game.add_faction(Faction::new(7, &[]));
        let region_id = game.add_region(Region::default());
        let object_id = game.add_object(region_id, Object::default()).unwrap();
        let unit_id = game
            .add_unit(object_id, Unit::new(10, Some(faction_id), 1, 5))
            .unwrap();
        game.setup_unit_nums().unwrap();
        (game, faction_id, object_id, unit_id)
    }

    fn default_work_game() -> (
        Game,
        crate::model::FactionId,
        crate::model::UnitId,
        crate::model::UnitId,
        crate::model::UnitId,
        crate::model::UnitId,
        crate::model::UnitId,
    ) {
        let mut game = Game::default();
        let faction_id = game.add_faction(Faction::new(7, &[]));
        let mut npc = Faction::new(8, &[]);
        npc.faction_types.push(("War".to_owned(), -1));
        let npc_faction_id = game.add_faction(npc);

        let region_id = game.add_region(Region::default());
        let other_region_id = game.add_region(Region::default());
        let mut nexus_region = Region::default();
        nexus_region.terrain_type = Some(crate::defs::terrains::R_NEXUS);
        let nexus_id = game.add_region(nexus_region);
        let object_id = game.add_object(region_id, Object::default()).unwrap();
        let other_object_id = game.add_object(other_region_id, Object::default()).unwrap();
        let nexus_object_id = game.add_object(nexus_id, Object::default()).unwrap();
        let mut work_unit_model = Unit::new(10, Some(faction_id), 0, 5);
        work_unit_model.flags |= super::FLAG_AUTOTAX;
        let work_unit = game.add_unit(object_id, work_unit_model).unwrap();
        let other_work_unit = game
            .add_unit(other_object_id, Unit::new(14, Some(faction_id), 0, 5))
            .unwrap();
        let ordered_unit = game
            .add_unit(object_id, Unit::new(11, Some(faction_id), 0, 5))
            .unwrap();
        let npc_unit = game
            .add_unit(object_id, Unit::new(12, Some(npc_faction_id), 0, 5))
            .unwrap();
        let nexus_unit = game
            .add_unit(nexus_object_id, Unit::new(13, Some(faction_id), 0, 5))
            .unwrap();
        game.unit_mut(ordered_unit).unwrap().orders.month_orders =
            Some(OrderData::Move(crate::orders::MoveOrder::default()));
        game.setup_unit_nums().unwrap();
        (
            game,
            faction_id,
            work_unit,
            other_work_unit,
            ordered_unit,
            npc_unit,
            nexus_unit,
        )
    }
}
