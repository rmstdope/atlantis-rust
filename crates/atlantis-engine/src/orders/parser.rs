use std::collections::VecDeque;
use std::io::{self, Cursor};

use crate::defs::{
    enums::flags as tax_flags, skills::skill_flags, BattleItemFlags, GameDefs, ItemFlags, ItemId,
    ItemTypeFlags, RulesetTables, WeaponFlags,
};
use crate::model::{Faction, FactionId, Game, ModelError, ObjectId, UnitId, UnitType};
use crate::ruleset::Ruleset;
use crate::text::{get_at, get_token, is_equal, value, TextInput};

use super::{parse_order_verb, OrderVerb, TurnOrder};

const FLAG_AUTOTAX: i32 = 0x0004;
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
    input: &[u8],
    mode: OrderParseMode,
) -> Result<OrderParseResult, OrderParseError> {
    Parser::new(game, defs, mode).parse(input)
}

struct Parser<'a> {
    game: &'a mut Game,
    defs: &'a GameDefs,
    mode: OrderParseMode,
    result: OrderParseResult,
    faction_id: Option<FactionId>,
    unit_stack: Vec<UnitId>,
    indent: i32,
}

impl<'a> Parser<'a> {
    fn new(game: &'a mut Game, defs: &'a GameDefs, mode: OrderParseMode) -> Self {
        Self {
            game,
            defs,
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
                self.dispatch_order(&token, verb, &raw_line, at_sign);
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

fn faction_is_npc(faction: &Faction) -> bool {
    faction
        .faction_types
        .iter()
        .any(|(name, value)| *value == -1 && (is_equal(name, "War") || is_equal(name, "Martial")))
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
    use super::{parse_order_file, OrderParseMode};
    use crate::defs::ItemId;
    use crate::model::{Faction, Game, Object, Region, Unit};
    use crate::orders::{OrderData, TurnOrder};
    use crate::ruleset::Ruleset;

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
    fn committed_fixture_orders_parse_without_new_faction_errors() {
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

        for (ruleset_name, turn, game_input, orders_input) in fixtures {
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
                assert!(
                    new_errors.is_empty(),
                    "unexpected parser errors for faction {faction_number} in \
                     {ruleset_name} turn {turn}: {new_errors:?}"
                );
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
        let mut parser = super::Parser::new(
            &mut restore_game,
            ruleset.game_defs(),
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
