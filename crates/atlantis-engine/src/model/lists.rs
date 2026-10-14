use crate::defs::ItemId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item {
    pub item_type: ItemId,
    pub num: i32,
    pub selling: i32,
    pub checked: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ItemList {
    entries: Vec<Item>,
}

impl ItemList {
    pub fn entries(&self) -> &[Item] {
        &self.entries
    }

    pub fn get_num(&self, item_type: ItemId) -> i32 {
        self.entries
            .iter()
            .find(|item| item.item_type == item_type)
            .map_or(0, |item| item.num)
    }

    pub fn push(&mut self, item: Item) {
        self.entries.push(item);
    }

    /// C++: `items.cpp` `ItemList::SetNum`; zero removes the entry, so re-adding appends it.
    pub fn set_num(&mut self, item_type: ItemId, num: i32) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|item| item.item_type == item_type)
        {
            if num == 0 {
                self.entries.remove(index);
            } else {
                self.entries[index].num = num;
            }
        } else if num != 0 {
            self.push(Item {
                item_type,
                num,
                selling: 0,
                checked: false,
            });
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Skill {
    pub skill_type: usize,
    pub days: u32,
    pub exp: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillList {
    entries: Vec<Skill>,
}

impl SkillList {
    pub fn entries(&self) -> &[Skill] {
        &self.entries
    }

    pub fn get_days(&self, skill_type: usize) -> u32 {
        self.entries
            .iter()
            .find(|skill| skill.skill_type == skill_type)
            .map_or(0, |skill| skill.days)
    }

    pub fn push(&mut self, skill: Skill) {
        self.entries.push(skill);
    }

    pub fn set_days(&mut self, skill_type: usize, days: u32) {
        let exp = self
            .entries
            .iter()
            .find(|skill| skill.skill_type == skill_type)
            .map_or(0, |skill| skill.exp);
        self.set(skill_type, days, exp);
    }

    pub fn set(&mut self, skill_type: usize, days: u32, exp: u32) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|skill| skill.skill_type == skill_type)
        {
            if days == 0 && exp == 0 {
                self.entries.remove(index);
            } else {
                let skill = &mut self.entries[index];
                skill.days = days;
                skill.exp = exp;
            }
        } else if days != 0 || exp != 0 {
            self.entries.push(Skill {
                skill_type,
                days,
                exp,
            });
        }
    }

    pub fn remove(&mut self, skill_type: usize) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|skill| skill.skill_type == skill_type)
        {
            self.entries.remove(index);
        }
    }
}
