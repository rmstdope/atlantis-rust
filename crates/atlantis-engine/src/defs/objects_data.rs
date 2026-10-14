// C++: gamedata.h object enum and gamedata.cpp ObjectDefs.
pub const O_DUMMY: ObjectId = 0;
pub const O_FLEET: ObjectId = 1;
pub const O_LONGBOAT: ObjectId = 2;
pub const O_LONGSHIP: ObjectId = 3;
pub const O_RAFT: ObjectId = 4;
pub const O_COG: ObjectId = 5;
pub const O_CLIPPER: ObjectId = 6;
pub const O_GALLEON: ObjectId = 7;
pub const O_AGALLEON: ObjectId = 8;
pub const O_GALLEY: ObjectId = 9;
pub const O_CORSAIR: ObjectId = 10;
pub const O_BALLOON: ObjectId = 11;
pub const O_AIRSHIP: ObjectId = 12;
pub const O_CLOUDSHIP: ObjectId = 13;
pub const O_TOWER: ObjectId = 14;
pub const O_FORT: ObjectId = 15;
pub const O_CASTLE: ObjectId = 16;
pub const O_CITADEL: ObjectId = 17;
pub const O_SHAFT: ObjectId = 18;
pub const O_LAIR: ObjectId = 19;
pub const O_RUIN: ObjectId = 20;
pub const O_CAVE: ObjectId = 21;
pub const O_DEMONPIT: ObjectId = 22;
pub const O_CRYPT: ObjectId = 23;
pub const O_MTOWER: ObjectId = 24;
pub const O_MFORTRESS: ObjectId = 25;
pub const O_MCASTLE: ObjectId = 26;
pub const O_MCITADEL: ObjectId = 27;
pub const O_MINE: ObjectId = 28;
pub const O_FARM: ObjectId = 29;
pub const O_RANCH: ObjectId = 30;
pub const O_TIMBERYARD: ObjectId = 31;
pub const O_INN: ObjectId = 32;
pub const O_QUARRY: ObjectId = 33;
pub const O_ISLE: ObjectId = 34;
pub const O_DERELICT: ObjectId = 35;
pub const O_OCAVE: ObjectId = 36;
pub const O_WHIRL: ObjectId = 37;
pub const O_ROADN: ObjectId = 38;
pub const O_ROADNW: ObjectId = 39;
pub const O_ROADNE: ObjectId = 40;
pub const O_ROADSW: ObjectId = 41;
pub const O_ROADSE: ObjectId = 42;
pub const O_ROADS: ObjectId = 43;
pub const O_TEMPLE: ObjectId = 44;
pub const O_MQUARRY: ObjectId = 45;
pub const O_AMINE: ObjectId = 46;
pub const O_PRESERVE: ObjectId = 47;
pub const O_SACGROVE: ObjectId = 48;
pub const O_BKEEP: ObjectId = 49;
pub const O_PALACE: ObjectId = 50;
pub const O_DCLIFFS: ObjectId = 51;
pub const O_HUT: ObjectId = 52;
pub const O_STOCKADE: ObjectId = 53;
pub const O_CPALACE: ObjectId = 54;
pub const O_NGUILD: ObjectId = 55;
pub const O_AGUILD: ObjectId = 56;
pub const O_ATEMPLE: ObjectId = 57;
pub const O_HTOWER: ObjectId = 58;
pub const O_MAGETOWER: ObjectId = 59;
pub const O_DARKTOWER: ObjectId = 60;
pub const O_GIANTCASTLE: ObjectId = 61;
pub const O_ILAIR: ObjectId = 62;
pub const O_ICECAVE: ObjectId = 63;
pub const O_BOG: ObjectId = 64;
pub const O_TRAPPINGHUT: ObjectId = 65;
pub const O_STABLE: ObjectId = 66;
pub const O_MSTABLE: ObjectId = 67;
pub const O_TRAPPINGLODGE: ObjectId = 68;
pub const O_FAERIERING: ObjectId = 69;
pub const O_ALCHEMISTLAB: ObjectId = 70;
pub const O_OASIS: ObjectId = 71;
pub const O_GEMAPPRAISER: ObjectId = 72;
pub const O_HPTOWER: ObjectId = 73;
pub const O_CARAVANSERAI: ObjectId = 74;
pub const O_GATEWAY: ObjectId = 75;

pub static OBJECT_DEFS: [ObjectDef; NOBJECTS] = [
    ObjectDef { name: "None", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Fleet", flags: 168, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Longboat", flags: 45, protection: 0, capacity: 200, sailors: 5, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Longship", flags: 45, protection: 0, capacity: 150, sailors: 4, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Raft", flags: 45, protection: 0, capacity: 450, sailors: 2, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Cog", flags: 45, protection: 0, capacity: 750, sailors: 6, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Clipper", flags: 45, protection: 0, capacity: 800, sailors: 10, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Galleon", flags: 45, protection: 0, capacity: 2700, sailors: 15, maximum_mages: 1, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Armored Galleon", flags: 45, protection: 200, capacity: 2700, sailors: 15, maximum_mages: 15, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [2, 2, 2, 2, 2, 2] },
    ObjectDef { name: "Galley", flags: 45, protection: 120, capacity: 1200, sailors: 12, maximum_mages: 2, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [2, 2, 2, 2, 2, 2] },
    ObjectDef { name: "Corsair", flags: 45, protection: 0, capacity: 1500, sailors: 15, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Balloon", flags: 45, protection: 0, capacity: 200, sailors: 3, maximum_mages: 1, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Airship", flags: 45, protection: 0, capacity: 800, sailors: 10, maximum_mages: 2, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Cloudship", flags: 45, protection: 140, capacity: 1400, sailors: 15, maximum_mages: 5, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [3, 3, 3, 3, 3, 3] },
    ObjectDef { name: "Tower", flags: 40, protection: 10, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 10, skill: Some("BUIL"), level: 1, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: -1, defense: [2, 2, 2, 2, 2, 2] },
    ObjectDef { name: "Fort", flags: 40, protection: 50, capacity: 0, sailors: 0, maximum_mages: 1, item: 24, cost: 40, skill: Some("BUIL"), level: 1, maximum_maintenance: 20, maximum_monthly_decay: 4, maintenance_factor: 3, monster: -1, production_aided: -1, defense: [2, 2, 2, 2, 2, 2] },
    ObjectDef { name: "Castle", flags: 40, protection: 300, capacity: 0, sailors: 0, maximum_mages: 2, item: 24, cost: 300, skill: Some("BUIL"), level: 2, maximum_maintenance: 80, maximum_monthly_decay: 6, maintenance_factor: 3, monster: -1, production_aided: -1, defense: [3, 2, 2, 2, 3, 3] },
    ObjectDef { name: "Citadel", flags: 40, protection: 1000, capacity: 0, sailors: 0, maximum_mages: 3, item: 24, cost: 800, skill: Some("BUIL"), level: 3, maximum_maintenance: 400, maximum_monthly_decay: 10, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [3, 2, 2, 2, 3, 3] },
    ObjectDef { name: "Shaft", flags: 12, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Lair", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 86, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Ruin", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 93, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Cave", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 92, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Demon Pit", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 106, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Crypt", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 103, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Magical Tower", flags: 41, protection: 10, capacity: 0, sailors: 0, maximum_mages: 3, item: 51, cost: 10, skill: Some("BUIL"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: -1, defense: [3, 3, 3, 3, 3, 3] },
    ObjectDef { name: "Magical Fort", flags: 41, protection: 50, capacity: 0, sailors: 0, maximum_mages: 10, item: 51, cost: 40, skill: Some("BUIL"), level: 3, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [3, 3, 3, 3, 3, 3] },
    ObjectDef { name: "Magical Fortress", flags: 40, protection: 300, capacity: 0, sailors: 0, maximum_mages: 30, item: 51, cost: 300, skill: Some("BUIL"), level: 4, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [4, 4, 4, 4, 4, 4] },
    ObjectDef { name: "Magical Citadel", flags: 41, protection: 1000, capacity: 0, sailors: 0, maximum_mages: 50, item: 51, cost: 800, skill: Some("BUIL"), level: 5, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [4, 4, 4, 4, 4, 4] },
    ObjectDef { name: "Mine", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("MINI"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 22, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Farm", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("FARM"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 20, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Ranch", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("FARM"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 21, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Timber Yard", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("LUMB"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 23, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Inn", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("BUIL"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 19, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Quarry", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("QUAR"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 24, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Island", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 145, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Derelict Ship", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 146, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Cavern", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 147, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Whirlpool", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 148, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road N", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road NW", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road NE", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road SW", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road SE", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Road S", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 75, skill: Some("BUIL"), level: 3, maximum_maintenance: 24, maximum_monthly_decay: 6, maintenance_factor: 4, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Temple", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 24, cost: 10, skill: Some("BUIL"), level: 3, maximum_maintenance: 5, maximum_monthly_decay: 1, maintenance_factor: 2, monster: -1, production_aided: 27, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Mystic Quarry", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 51, cost: 10, skill: Some("QUAR"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 51, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Arcane Mine", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 47, cost: 10, skill: Some("MINI"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 47, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Forest Preserve", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 48, cost: 10, skill: Some("LUMB"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 48, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Sacred Grove", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 52, cost: 10, skill: Some("LUMB"), level: 5, maximum_maintenance: 15, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 52, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Black Keep", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 51, cost: 1000, skill: Some("BUIL"), level: 5, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Palace", flags: 44, protection: 1250, capacity: 0, sailors: 0, maximum_mages: 200, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Dragon Cliffs", flags: 41, protection: 1, capacity: 0, sailors: 0, maximum_mages: 0, item: 51, cost: 100, skill: Some("DRAG"), level: 3, maximum_maintenance: 50, maximum_monthly_decay: 5, maintenance_factor: 2, monster: 92, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Hermits hut", flags: 41, protection: 1, capacity: 0, sailors: 0, maximum_mages: 1, item: 48, cost: 5, skill: Some("BUIL"), level: 3, maximum_maintenance: 1, maximum_monthly_decay: 1, maintenance_factor: 1, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Stockade", flags: 40, protection: 50, capacity: 0, sailors: 0, maximum_mages: 1, item: 23, cost: 60, skill: Some("BUIL"), level: 1, maximum_maintenance: 30, maximum_monthly_decay: 3, maintenance_factor: 2, monster: -1, production_aided: -1, defense: [1, 1, 1, 1, 1, 1] },
    ObjectDef { name: "Cloud Palace", flags: 13, protection: 200, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Necromancers Guild", flags: 13, protection: 10, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Artificers Guild", flags: 13, protection: 10, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Ancient Temple", flags: 13, protection: 8, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 103, production_aided: 27, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Hexagonal Tower", flags: 45, protection: 7, capacity: 0, sailors: 0, maximum_mages: 2, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [2, 2, 2, 2, 2, 2] },
    ObjectDef { name: "Magician's Tower", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 185, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Dark Mage Tower", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 186, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Giant's Castle", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 181, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Lair", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 183, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Ice Cave", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 188, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Bog", flags: 4, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: 180, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Trapping Hut", flags: 40, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("HUNT"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 25, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Stables", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("HORS"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 28, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Mythic Stables", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 48, cost: 10, skill: Some("HORS"), level: 5, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 49, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Trapping Lodge", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 48, cost: 10, skill: Some("HUNT"), level: 5, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 50, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Faerie Ring", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 164, cost: 10, skill: Some("HERB"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 164, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Alchemist Lab", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: 170, cost: 10, skill: Some("MINI"), level: 5, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 170, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Oasis", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("HORS"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 177, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Gem Appraiser", flags: 41, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 10, skill: Some("MINI"), level: 3, maximum_maintenance: 10, maximum_monthly_decay: 2, maintenance_factor: 2, monster: -1, production_aided: 190, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Hexagonal Portal Tower", flags: 41, protection: 7, capacity: 0, sailors: 0, maximum_mages: 1, item: 48, cost: 25, skill: None, level: 0, maximum_maintenance: 25, maximum_monthly_decay: 2, maintenance_factor: 3, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Caravanserai", flags: 105, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -2, cost: 20, skill: Some("BUIL"), level: 2, maximum_maintenance: 15, maximum_monthly_decay: 3, maintenance_factor: 3, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
    ObjectDef { name: "Gateway", flags: 12, protection: 0, capacity: 0, sailors: 0, maximum_mages: 0, item: -1, cost: 0, skill: None, level: 0, maximum_maintenance: 0, maximum_monthly_decay: 0, maintenance_factor: 0, monster: -1, production_aided: -1, defense: [0, 0, 0, 0, 0, 0] },
];
