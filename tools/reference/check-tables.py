#!/usr/bin/env python3

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


TABLE_FIELDS = {
    "items": {
        "name", "plural", "abbreviation", "flags", "production_skill",
        "production_level", "production_months", "production_output",
        "production_inputs", "magic_skill", "magic_level", "magic_output",
        "magic_inputs", "weight", "type", "base_price", "combat", "walk",
        "ride", "fly", "swim", "speed", "hitch_item", "hitch_walk",
        "mult_item", "mult_value", "max_inventory", "escape", "escape_skill",
        "escape_value", "grant_skill", "grant_from", "min_grant", "max_grant",
    },
    "races": {
        "abbreviation", "terrain", "special_level", "default_level", "skills",
        "ethnicity",
    },
    "monsters": {
        "attack_level", "defense", "number_of_attacks", "hits", "regeneration",
        "tactics", "stealth", "observation", "special", "special_level",
        "silver", "spoil_type", "hostile", "number", "name", "abbreviation",
        "hit_damage", "preferred_terrain", "forbidden_terrain",
    },
    "weapons": {
        "abbreviation", "flags", "base_skill", "or_skill", "weapon_class",
        "attack_type", "number_of_attacks", "attack_bonus", "defense_bonus",
        "mount_bonus", "hit_damage", "bonus_malus",
    },
    "armor": {"abbreviation", "flags", "from", "saves"},
    "mounts": {
        "abbreviation", "skill", "minimum_bonus", "maximum_bonus",
        "maximum_hampered_bonus", "special", "special_level",
    },
    "battle_items": {
        "abbreviation", "flags", "special", "skill_level", "hit_damage",
    },
    "objects": {
        "name", "flags", "protection", "capacity", "sailors", "maximum_mages",
        "item", "cost", "skill", "level", "maximum_maintenance",
        "maximum_monthly_decay", "maintenance_factor", "monster",
        "production_aided", "defense",
    },
    "skills": {
        "name", "abbreviation", "cost", "flags", "special", "range",
        "dependencies",
    },
    "terrains": {
        "name", "plural", "type", "marker", "similar_type", "flags",
        "population", "wages", "economy", "movement_points", "products",
        "races", "coastal_races", "wandering_monster_frequency",
        "small_monster", "big_monster", "humanoid", "lair_chance", "lairs",
    },
    "healing": {"number", "rate"},
    "magic_healing": {"number", "rate"},
    "specials": {
        "key", "name", "target_flags", "buildings", "targets", "effects",
        "effect_flags", "shields", "defenses", "shield_description", "damage",
        "spell_description", "secondary_spell_description", "spell_target",
    },
    "effects": {
        "name", "attack_value", "defense_modifiers", "cancel_effect", "flags",
    },
    "ranges": {
        "key", "flags", "range_class", "range_multiplier",
        "cross_level_penalty",
    },
    "attribute_modifiers": {"key", "flags", "modifiers"},
}

ARRAY_FIELDS = {
    "items": {"production_inputs", "magic_inputs", "grant_from"},
    "races": {"skills"},
    "monsters": {"defense", "preferred_terrain", "forbidden_terrain"},
    "weapons": {"bonus_malus"},
    "armor": {"saves"},
    "objects": {"defense"},
    "skills": {"dependencies"},
    "terrains": {"products", "races", "coastal_races", "lairs"},
    "specials": {"buildings", "targets", "effects", "shields", "defenses", "damage"},
    "effects": {"defense_modifiers"},
    "attribute_modifiers": {"modifiers"},
}

OBJECT_ARRAY_FIELDS = {
    "items": {
        "production_inputs": {"item", "amount"},
        "magic_inputs": {"item", "amount"},
    },
    "weapons": {
        "bonus_malus": {"weapon", "attack_modifier", "defense_modifier"},
    },
    "skills": {"dependencies": {"skill", "level"}},
    "terrains": {"products": {"item", "chance", "amount"}},
    "specials": {
        "defenses": {"type", "value"},
        "damage": {
            "type", "minimum", "value", "flags", "class", "effect", "hit_damage",
        },
    },
    "effects": {"defense_modifiers": {"type", "value"}},
    "attribute_modifiers": {
        "modifiers": {"flags", "identifier", "type", "value"},
    },
}

SKILL_FAMILY_TABLES = {
    "skills",
    "healing",
    "magic_healing",
    "specials",
    "effects",
    "ranges",
    "attribute_modifiers",
}


def fail(message):
    print(message, file=sys.stderr)
    raise SystemExit(1)


def consumer_root():
    root = Path(__file__).resolve().parents[2]
    if root.parent.name == "worktrees" and root.parent.parent.name == ".cerebro":
        return root.parent.parent.parent
    return root


def expected_game_defs(source):
    header = source.read_text(encoding="utf-8")
    body = header.split("class GameDefs {", 1)[1].split(
        "extern GameDefs *Globals;", 1
    )[0]
    pattern = (
        r"^\s*(?:char const \*\s*|int\s+|ATL_VER\s+|"
        r"FactionActivityRules\s+|DestroyBehavior\s+|BattleLogLevel\s+)"
        r"([A-Z][A-Z0-9_]*)\s*;"
    )
    return set(re.findall(pattern, body, re.MULTILINE))


def run_dump(binary, ruleset):
    if not binary.is_file() or not os.access(binary, os.X_OK):
        fail(f"{ruleset}: reference binary is not executable: {binary}")

    outputs = []
    for _ in range(2):
        with tempfile.TemporaryDirectory(prefix=f"atlantis-check-{ruleset}-") as cwd:
            result = subprocess.run(
                [str(binary), "dump-tables"],
                check=False,
                capture_output=True,
                cwd=cwd,
                text=True,
            )
        if result.returncode != 0:
            fail(
                f"{ruleset}: dump-tables exited {result.returncode}: "
                f"{result.stderr.strip()}"
            )
        outputs.append(result.stdout)

    if outputs[0] != outputs[1]:
        fail(f"{ruleset}: dump-tables output is not deterministic")
    if "Atlantis Engine Version:" in outputs[0]:
        fail(f"{ruleset}: dump-tables output contains the engine banner")

    rows = []
    for line_number, line in enumerate(outputs[0].splitlines(), 1):
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            fail(f"{ruleset}: invalid JSON on line {line_number}: {error}")
        if not isinstance(row, dict) or not isinstance(row.get("table"), str):
            fail(f"{ruleset}: line {line_number} is not a table record")
        rows.append(row)

    if not rows or rows[0].get("table") != "metadata":
        fail(f"{ruleset}: metadata record is missing or not first")
    metadata = [row for row in rows if row["table"] == "metadata"]
    if len(metadata) != 1 or metadata[0].get("format_version") != 1:
        fail(f"{ruleset}: unsupported or duplicate dump metadata")

    counts_rows = [row for row in rows if row["table"] == "counts"]
    counts = {row.get("name"): row.get("value") for row in counts_rows}
    if len(counts_rows) != len(TABLE_FIELDS) or set(counts) != set(TABLE_FIELDS):
        fail(f"{ruleset}: table counts do not match the expected table set")
    if any(not isinstance(value, int) or value < 0 for value in counts.values()):
        fail(f"{ruleset}: a table count is not a non-negative integer")

    global_rows = [row for row in rows if row["table"] == "globals"]
    actual_globals = [row.get("name") for row in global_rows]
    if len(actual_globals) != len(set(actual_globals)):
        fail(f"{ruleset}: duplicate GameDefs member in globals")

    expected_tables = set(TABLE_FIELDS) | {"metadata", "globals", "counts"}
    actual_tables = {row["table"] for row in rows}
    if actual_tables != expected_tables:
        fail(f"{ruleset}: unexpected or missing records: {actual_tables ^ expected_tables}")

    expected_record_fields = {
        "metadata": {"table", "format_version"},
        "globals": {"table", "name", "value"},
        "counts": {"table", "name", "value"},
    }
    for table, fields in TABLE_FIELDS.items():
        expected_record_fields[table] = {"table", "index"} | fields

    for row in rows:
        table = row["table"]
        if set(row) != expected_record_fields[table]:
            fail(
                f"{ruleset}: unexpected fields in {table} record: "
                f"{set(row) ^ expected_record_fields[table]}"
            )
        for field in ARRAY_FIELDS.get(table, set()):
            if not isinstance(row[field], list):
                fail(f"{ruleset}: {table}.{field} is not an array")
        for field, object_fields in OBJECT_ARRAY_FIELDS.get(table, {}).items():
            for index, value in enumerate(row[field]):
                if not isinstance(value, dict) or set(value) != object_fields:
                    fail(
                        f"{ruleset}: {table}.{field}[{index}] has an "
                        "unexpected object shape"
                    )
    for table, count in counts.items():
        indices = [row.get("index") for row in rows if row["table"] == table]
        if indices != list(range(count)):
            fail(f"{ruleset}: {table} indices do not match its declared count")

    return rows, global_rows


def check_pre_modify_skill_dump(binary, ruleset, project):
    if not binary.is_file() or not os.access(binary, os.X_OK):
        fail(f"{ruleset}: reference binary is not executable: {binary}")

    outputs = []
    for _ in range(2):
        with tempfile.TemporaryDirectory(prefix=f"atlantis-skills-{ruleset}-") as cwd:
            result = subprocess.run(
                [str(binary), "dump-skill-tables"],
                check=False,
                capture_output=True,
                cwd=cwd,
                text=True,
            )
        if result.returncode != 0:
            fail(
                f"{ruleset}: dump-skill-tables exited {result.returncode}: "
                f"{result.stderr.strip()}"
            )
        outputs.append(result.stdout)

    if outputs[0] != outputs[1]:
        fail(f"{ruleset}: pre-modify skill-family dump is not deterministic")

    fixture = (
        project
        / "fixtures"
        / "tables"
        / "pre-modify-skill-families"
        / f"{ruleset}.jsonl"
    )
    if not fixture.is_file():
        fail(f"{ruleset}: pre-modify skill-family fixture is missing: {fixture}")
    if outputs[0].encode() != fixture.read_bytes():
        fail(f"{ruleset}: pre-modify skill-family dump differs from {fixture}")

    rows = []
    for line_number, line in enumerate(outputs[0].splitlines(), 1):
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            fail(f"{ruleset}: invalid skill-family JSON on line {line_number}: {error}")
        if not isinstance(row, dict) or not isinstance(row.get("table"), str):
            fail(f"{ruleset}: skill-family line {line_number} is not a table record")
        rows.append(row)

    counts_rows = [row for row in rows if row["table"] == "counts"]
    counts = {row.get("name"): row.get("value") for row in counts_rows}
    if len(counts_rows) != len(SKILL_FAMILY_TABLES) or set(counts) != SKILL_FAMILY_TABLES:
        fail(f"{ruleset}: pre-modify skill-family counts are incomplete")
    expected_tables = SKILL_FAMILY_TABLES | {"counts"}
    if {row["table"] for row in rows} != expected_tables:
        fail(f"{ruleset}: pre-modify skill-family tables are incomplete")

    for row in rows:
        table = row["table"]
        if table == "counts":
            expected_fields = {"table", "name", "value"}
        else:
            expected_fields = {"table", "index"} | TABLE_FIELDS[table]
        if set(row) != expected_fields:
            fail(f"{ruleset}: unexpected fields in {table} record")
        for field in ARRAY_FIELDS.get(table, set()):
            if not isinstance(row[field], list):
                fail(f"{ruleset}: {table}.{field} is not an array")
        for field, object_fields in OBJECT_ARRAY_FIELDS.get(table, {}).items():
            for index, value in enumerate(row[field]):
                if not isinstance(value, dict) or set(value) != object_fields:
                    fail(
                        f"{ruleset}: {table}.{field}[{index}] has an "
                        "unexpected object shape"
                    )

    for table, count in counts.items():
        indices = [row.get("index") for row in rows if row["table"] == table]
        if not isinstance(count, int) or indices != list(range(count)):
            fail(f"{ruleset}: pre-modify {table} indices do not match its count")
    print(f"{ruleset}: pre-modify skill-family JSONL matches its reference snapshot")


def item_flags(rows, abbreviation, ruleset):
    matches = [
        row
        for row in rows
        if row["table"] == "items" and row.get("abbreviation") == abbreviation
    ]
    if len(matches) != 1:
        fail(f"{ruleset}: expected exactly one {abbreviation} item")
    return matches[0].get("flags")


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {Path(sys.argv[0]).name} <reference-build-root>", file=sys.stderr)
        return 2

    build_root = Path(sys.argv[1]).expanduser().resolve() / "Atlantis"
    project = Path(__file__).resolve().parents[2]
    consumer = consumer_root()
    source = consumer.parent / "Atlantis" / "gamedefs.h"
    if not source.is_file():
        fail(f"C++ GameDefs header not found: {source}")
    expected_globals = expected_game_defs(source)
    if not expected_globals:
        fail(f"No GameDefs members found in {source}")

    dumps = {}
    for ruleset in ("standard", "neworigins"):
        binary = build_root / ruleset / ruleset
        rows, global_rows = run_dump(
            binary,
            ruleset,
        )
        check_pre_modify_skill_dump(binary, ruleset, project)
        actual_globals = {row.get("name") for row in global_rows}
        if actual_globals != expected_globals:
            fail(
                f"{ruleset}: GameDefs members differ; "
                f"missing={sorted(expected_globals - actual_globals)}, "
                f"unexpected={sorted(actual_globals - expected_globals)}"
            )
        dumps[ruleset] = rows
        print(f"{ruleset}: deterministic JSONL, {len(rows)} records, tables verified")

    standard_flags = item_flags(dumps["standard"], "SBOW", "standard")
    neworigins_flags = item_flags(dumps["neworigins"], "SBOW", "neworigins")
    if standard_flags == neworigins_flags:
        fail("SBOW flags did not reflect the standard/neworigins table mutation")
    print("SBOW flags differ between standard and neworigins as expected")
    return 0


if __name__ == "__main__":
    sys.exit(main())
