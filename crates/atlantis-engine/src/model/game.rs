use serde::{Deserialize, Serialize};

use super::entities::{Faction, Object, Quest, Region, Unit};
use super::{FactionId, ObjectId, RegionId, UnitId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum RegionLevelType {
    Nexus = 0,
    Surface = 1,
    Underworld = 2,
    Underdeep = 3,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegionArray {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub level_type: RegionLevelType,
    regions: Vec<Option<RegionId>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionArrayError {
    InvalidDimensions,
    OutOfBounds,
    CoordinateHasNoRegion,
}

impl std::fmt::Display for RegionArrayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidDimensions => {
                "region array dimensions must be positive and have even width, except for the 1x1 nexus"
            }
            Self::OutOfBounds => "region coordinate is outside the array",
            Self::CoordinateHasNoRegion => "region coordinate is not a hex-grid cell",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RegionArrayError {}

impl RegionArray {
    pub fn new(x: i32, y: i32, level_type: RegionLevelType) -> Result<Self, RegionArrayError> {
        let single_cell_nexus = x == 1 && y == 1 && level_type == RegionLevelType::Nexus;
        if x <= 0 || y <= 0 || (!single_cell_nexus && x % 2 != 0) {
            return Err(RegionArrayError::InvalidDimensions);
        }
        let cells = if single_cell_nexus {
            1
        } else {
            x.checked_mul(y)
                .and_then(|area| usize::try_from(area / 2).ok())
                .ok_or(RegionArrayError::InvalidDimensions)?
        };

        Ok(Self {
            x,
            y,
            name: String::new(),
            level_type,
            regions: vec![None; cells],
        })
    }

    /// C++: `aregion.cpp` `ARegionArray::SetRegion`.
    pub fn set_region(
        &mut self,
        x: i32,
        y: i32,
        region_id: RegionId,
    ) -> Result<(), RegionArrayError> {
        let index = self.index_for_cell(x, y)?;
        self.regions[index] = Some(region_id);
        Ok(())
    }

    /// C++: `aregion.cpp` `ARegionArray::GetRegion`; wraps coordinates and rejects non-cells.
    pub fn get_region(&self, x: i32, y: i32) -> Option<RegionId> {
        let x = wrap_coordinate(x, self.x)?;
        let y = wrap_coordinate(y, self.y)?;
        if (i64::from(x) + i64::from(y)) % 2 != 0 {
            return None;
        }
        let index = self.cell_index(x, y);
        self.regions.get(index).copied().flatten()
    }

    fn index_for_cell(&self, x: i32, y: i32) -> Result<usize, RegionArrayError> {
        if x < 0 || y < 0 || x >= self.x || y >= self.y {
            return Err(RegionArrayError::OutOfBounds);
        }
        if (i64::from(x) + i64::from(y)) % 2 != 0 {
            return Err(RegionArrayError::CoordinateHasNoRegion);
        }
        Ok(self.cell_index(x, y))
    }

    fn cell_index(&self, x: i32, y: i32) -> usize {
        (x / 2 + y * self.x / 2) as usize
    }
}

fn wrap_coordinate(coordinate: i32, size: i32) -> Option<i32> {
    let wrapped = (i64::from(coordinate) + i64::from(size)) % i64::from(size);
    i32::try_from(wrapped).ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelError {
    MissingRegion(RegionId),
    MissingObject(ObjectId),
    ObjectHasNoRegion(ObjectId),
    ObjectContainsUnits(ObjectId),
    MissingUnit(UnitId),
    UnitNumberOverflow,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRegion(id) => write!(formatter, "region {} does not exist", id.index()),
            Self::MissingObject(id) => write!(formatter, "object {} does not exist", id.index()),
            Self::ObjectHasNoRegion(id) => {
                write!(formatter, "object {} has no region", id.index())
            }
            Self::ObjectContainsUnits(id) => {
                write!(formatter, "object {} still contains units", id.index())
            }
            Self::MissingUnit(id) => write!(formatter, "unit {} does not exist", id.index()),
            Self::UnitNumberOverflow => {
                formatter.write_str("unit number exceeds the supported range")
            }
        }
    }
}

impl std::error::Error for ModelError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitNumberingIssue {
    Duplicate {
        unit: UnitId,
        duplicate_number: i32,
        reassigned_number: i32,
    },
    Invalid {
        unit: UnitId,
        invalid_number: i32,
        reassigned_number: i32,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Game {
    factions: Vec<Option<Faction>>,
    faction_ids: Vec<FactionId>,
    regions: Vec<Option<Region>>,
    region_ids: Vec<RegionId>,
    objects: Vec<Option<Object>>,
    units: Vec<Option<Unit>>,
    pub region_arrays: Vec<RegionArray>,
    pub faction_seq: i32,
    pub unit_seq: u32,
    pub ship_seq: i32,
    pub number_of_gates: i32,
    pub year: i32,
    pub month: i32,
    pub guard_faction: Option<FactionId>,
    pub monster_faction: Option<FactionId>,
    pub new_factions: Vec<String>,
    pub battles: Vec<String>,
    pub game_status: i32,
    pub do_extra_init: i32,
    pub events: Vec<String>,
    pub quests: Vec<Quest>,
    unit_numbers: Vec<Option<UnitId>>,
    next_free_unit_number: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            factions: Vec::new(),
            faction_ids: Vec::new(),
            regions: Vec::new(),
            region_ids: Vec::new(),
            objects: Vec::new(),
            units: Vec::new(),
            region_arrays: Vec::new(),
            faction_seq: 1,
            unit_seq: 1,
            ship_seq: 100,
            number_of_gates: 0,
            year: 1,
            month: -1,
            guard_faction: None,
            monster_faction: None,
            new_factions: Vec::new(),
            battles: Vec::new(),
            game_status: 0,
            do_extra_init: 0,
            events: Vec::new(),
            quests: Vec::new(),
            unit_numbers: Vec::new(),
            next_free_unit_number: 1,
        }
    }
}

impl Game {
    pub fn faction_ids(&self) -> &[FactionId] {
        &self.faction_ids
    }

    pub fn region_ids(&self) -> &[RegionId] {
        &self.region_ids
    }

    pub fn faction(&self, id: FactionId) -> Option<&Faction> {
        self.factions.get(id.index()).and_then(Option::as_ref)
    }

    pub fn faction_mut(&mut self, id: FactionId) -> Option<&mut Faction> {
        self.factions.get_mut(id.index()).and_then(Option::as_mut)
    }

    pub fn region(&self, id: RegionId) -> Option<&Region> {
        self.regions.get(id.index()).and_then(Option::as_ref)
    }

    pub fn region_mut(&mut self, id: RegionId) -> Option<&mut Region> {
        self.regions.get_mut(id.index()).and_then(Option::as_mut)
    }

    pub fn object(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(id.index()).and_then(Option::as_ref)
    }

    pub fn object_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(id.index()).and_then(Option::as_mut)
    }

    pub fn unit(&self, id: UnitId) -> Option<&Unit> {
        self.units.get(id.index()).and_then(Option::as_ref)
    }

    pub fn unit_mut(&mut self, id: UnitId) -> Option<&mut Unit> {
        self.units.get_mut(id.index()).and_then(Option::as_mut)
    }

    pub fn add_faction(&mut self, faction: Faction) -> FactionId {
        let id = FactionId::new(self.factions.len());
        self.factions.push(Some(faction));
        self.faction_ids.push(id);
        id
    }

    pub fn add_region(&mut self, region: Region) -> RegionId {
        let id = RegionId::new(self.regions.len());
        self.regions.push(Some(region));
        self.region_ids.push(id);
        id
    }

    pub fn add_object(
        &mut self,
        region_id: RegionId,
        mut object: Object,
    ) -> Result<ObjectId, ModelError> {
        if self.region(region_id).is_none() {
            return Err(ModelError::MissingRegion(region_id));
        }
        let id = ObjectId::new(self.objects.len());
        object.region = Some(region_id);
        self.objects.push(Some(object));
        self.region_mut(region_id)
            .expect("region was checked above")
            .add_object(id);
        Ok(id)
    }

    /// C++: `edit.cpp` removes the object from the region list without deleting the allocation.
    pub fn detach_region_object(
        &mut self,
        region_id: RegionId,
        object_id: ObjectId,
    ) -> Result<(), ModelError> {
        let region = self
            .region_mut(region_id)
            .ok_or(ModelError::MissingRegion(region_id))?;
        let index = region
            .object_ids
            .iter()
            .position(|id| *id == object_id)
            .ok_or(ModelError::MissingObject(object_id))?;
        region.object_ids.remove(index);
        Ok(())
    }

    /// C++: `economy.cpp` `ARegion::AddTown`; empties the lair's unit list without deleting units.
    pub fn empty_object_units(&mut self, object_id: ObjectId) -> Result<(), ModelError> {
        self.object_mut(object_id)
            .ok_or(ModelError::MissingObject(object_id))?
            .unit_ids
            .clear();
        Ok(())
    }

    pub fn remove_empty_object(&mut self, object_id: ObjectId) -> Result<Object, ModelError> {
        let object = self
            .object(object_id)
            .ok_or(ModelError::MissingObject(object_id))?;
        if !object.unit_ids.is_empty() {
            return Err(ModelError::ObjectContainsUnits(object_id));
        }
        let region_id = object
            .region
            .ok_or(ModelError::ObjectHasNoRegion(object_id))?;
        self.region_mut(region_id)
            .ok_or(ModelError::MissingRegion(region_id))?
            .object_ids
            .retain(|id| *id != object_id);
        Ok(self.objects[object_id.index()]
            .take()
            .expect("object was checked above"))
    }

    pub fn add_unit(&mut self, object_id: ObjectId, mut unit: Unit) -> Result<UnitId, ModelError> {
        if self.object(object_id).is_none() {
            return Err(ModelError::MissingObject(object_id));
        }
        let id = UnitId::new(self.units.len());
        unit.object = Some(object_id);
        self.units.push(Some(unit));
        self.object_mut(object_id)
            .expect("object was checked above")
            .add_unit(id);
        Ok(id)
    }

    pub fn add_hell_unit(
        &mut self,
        region_id: RegionId,
        mut unit: Unit,
    ) -> Result<UnitId, ModelError> {
        if self.region(region_id).is_none() {
            return Err(ModelError::MissingRegion(region_id));
        }
        let id = UnitId::new(self.units.len());
        unit.object = None;
        self.units.push(Some(unit));
        self.region_mut(region_id)
            .expect("region was checked above")
            .add_hell_unit(id);
        Ok(id)
    }

    /// C++: `unit.cpp` `Unit::MoveUnit`; removing and moving a unit appends it to the new list.
    pub fn attach_unit(&mut self, object_id: ObjectId, unit_id: UnitId) -> Result<(), ModelError> {
        if self.object(object_id).is_none() {
            return Err(ModelError::MissingObject(object_id));
        }
        let previous_object = self
            .unit(unit_id)
            .ok_or(ModelError::MissingUnit(unit_id))?
            .object;
        if let Some(previous_object) = previous_object {
            if let Some(object) = self.object_mut(previous_object) {
                object.unit_ids.retain(|id| *id != unit_id);
            }
        }
        self.unit_mut(unit_id)
            .expect("unit was checked above")
            .object = Some(object_id);
        self.object_mut(object_id)
            .expect("object was checked above")
            .add_unit(unit_id);
        Ok(())
    }

    pub fn add_region_array(&mut self, region_array: RegionArray) -> usize {
        let id = self.region_arrays.len();
        self.region_arrays.push(region_array);
        id
    }

    /// C++: `game.cpp` `Game::TurnNumber`.
    pub fn turn_number(&self) -> i32 {
        (self.year - 1) * 12 + self.month + 1
    }

    /// C++: `aregion.cpp` `ARegionList::GetRegion(int)`; the first matching region wins.
    pub fn get_region(&self, number: i32) -> Option<RegionId> {
        self.region_ids
            .iter()
            .copied()
            .find(|id| self.region(*id).is_some_and(|region| region.num == number))
    }

    /// C++: `aregion.cpp` `ARegionList::GetRegion(int, int, int)`.
    pub fn get_region_at(&self, x: i32, y: i32, z: i32) -> Option<RegionId> {
        let array = usize::try_from(z)
            .ok()
            .and_then(|index| self.region_arrays.get(index))?;
        array.get_region(x, y)
    }

    /// C++: `aregion.cpp` `ARegion::GetDummy`; returns the first dummy object in list order.
    pub fn get_dummy(&self, region_id: RegionId) -> Option<ObjectId> {
        self.region(region_id)?
            .object_ids()
            .iter()
            .copied()
            .find(|object_id| {
                self.object(*object_id).is_some_and(|object| {
                    object.object_type == crate::defs::objects::O_DUMMY as i32
                })
            })
    }

    /// C++: `game.cpp` `SetupUnitSeq`; scans regions, objects, then their unit lists.
    pub fn setup_unit_seq(&mut self) -> Result<u32, ModelError> {
        let max_number = self
            .ordered_unit_ids()
            .into_iter()
            .filter_map(|id| self.unit(id).map(|unit| unit.num))
            .filter(|number| *number > 0)
            .max()
            .unwrap_or(0);
        self.unit_seq = u32::try_from(max_number)
            .ok()
            .and_then(|number| number.checked_add(1))
            .ok_or(ModelError::UnitNumberOverflow)?;
        Ok(self.unit_seq)
    }

    /// C++: `game.cpp` `SetupUnitNums`; duplicate and nonpositive numbers are reassigned in list order.
    pub fn setup_unit_nums(&mut self) -> Result<Vec<UnitNumberingIssue>, ModelError> {
        self.setup_unit_seq()?;
        self.unit_numbers = vec![None; self.unit_seq as usize];
        let mut issues = Vec::new();

        for unit_id in self.ordered_unit_ids() {
            let number = self.unit(unit_id).expect("ordered unit exists").num;
            let index = usize::try_from(number).ok();
            if let Some(index) =
                index.filter(|index| *index > 0 && *index < self.unit_numbers.len())
            {
                if self.unit_numbers[index].is_none() {
                    self.unit_numbers[index] = Some(unit_id);
                    continue;
                }
                let reassigned = self.assign_next_unit_number(unit_id)?;
                issues.push(UnitNumberingIssue::Duplicate {
                    unit: unit_id,
                    duplicate_number: number,
                    reassigned_number: reassigned,
                });
            } else {
                let reassigned = self.assign_next_unit_number(unit_id)?;
                issues.push(UnitNumberingIssue::Invalid {
                    unit: unit_id,
                    invalid_number: number,
                    reassigned_number: reassigned,
                });
            }
        }
        self.next_free_unit_number = self.first_free_unit_number(1);
        Ok(issues)
    }

    /// C++: `game.cpp` `Game::GetNewUnit`; reuses the lowest free number and receives active-rules movement points.
    pub fn get_new_unit(
        &mut self,
        faction: Option<FactionId>,
        alias: i32,
        default_move_points: i32,
    ) -> Result<UnitId, ModelError> {
        let number = self.first_free_unit_number(self.next_free_unit_number.min(self.unit_seq));
        let reuses_number = number < self.unit_seq;
        let next_seq = if reuses_number {
            self.unit_seq
        } else {
            number
                .checked_add(1)
                .ok_or(ModelError::UnitNumberOverflow)?
        };
        let legacy_number = i32::try_from(number).map_err(|_| ModelError::UnitNumberOverflow)?;
        self.ensure_unit_number_capacity(number)?;
        let id = UnitId::new(self.units.len());
        self.units.push(Some(Unit::new(
            legacy_number,
            faction,
            alias,
            default_move_points,
        )));
        self.unit_numbers[number as usize] = Some(id);
        self.unit_seq = next_seq;
        self.next_free_unit_number = if reuses_number {
            self.first_free_unit_number(number + 1)
        } else {
            next_seq
        };
        Ok(id)
    }

    /// C++: `game.cpp` `Game::GetUnit`.
    pub fn get_unit_id(&self, number: i32) -> Option<UnitId> {
        let index = usize::try_from(number).ok()?;
        self.unit_numbers.get(index).copied().flatten()
    }

    pub fn get_unit(&self, number: i32) -> Option<&Unit> {
        self.unit(self.get_unit_id(number)?)
    }

    fn assign_next_unit_number(&mut self, unit_id: UnitId) -> Result<i32, ModelError> {
        let number = i32::try_from(self.unit_seq).map_err(|_| ModelError::UnitNumberOverflow)?;
        let next_seq = self
            .unit_seq
            .checked_add(1)
            .ok_or(ModelError::UnitNumberOverflow)?;
        self.ensure_unit_number_capacity(self.unit_seq)?;
        self.units[unit_id.index()]
            .as_mut()
            .expect("ordered unit exists")
            .num = number;
        self.unit_numbers[self.unit_seq as usize] = Some(unit_id);
        self.unit_seq = next_seq;
        Ok(number)
    }

    fn ensure_unit_number_capacity(&mut self, number: u32) -> Result<(), ModelError> {
        let index = usize::try_from(number).map_err(|_| ModelError::UnitNumberOverflow)?;
        if self.unit_numbers.len() <= index {
            self.unit_numbers.resize(
                index.checked_add(1).ok_or(ModelError::UnitNumberOverflow)?,
                None,
            );
        }
        Ok(())
    }

    fn first_free_unit_number(&self, start: u32) -> u32 {
        (start..self.unit_seq)
            .find(|number| {
                self.unit_numbers
                    .get(*number as usize)
                    .is_none_or(|id| id.is_none())
            })
            .unwrap_or(self.unit_seq)
    }

    fn ordered_unit_ids(&self) -> Vec<UnitId> {
        self.region_ids
            .iter()
            .filter_map(|region_id| self.region(*region_id))
            .flat_map(|region| region.object_ids().iter())
            .filter_map(|object_id| self.object(*object_id))
            .flat_map(|object| object.unit_ids().iter().copied())
            .collect()
    }
}
