# Orders and parsing

Purpose: order verbs, order data, tokenizer semantics, and the order-file state machine, so the Rust
`orders` and `text` modules parse and validate exactly as the original, including `atlantis check`.

## Order verbs (`orders.h:58-126`, strings `orders.cpp:27-93`)

66 verbs (`NORDERS`), matched by `Parse1Order` as a straight scan of the string table:

`#atlantis #end UNIT ADDRESS ADVANCE ARMOR ASSASSINATE ATTACK AUTOTAX AVOID BEHIND BUILD BUY CAST
CLAIM COMBAT CONSUME DECLARE DESCRIBE DESTROY DISTRIBUTE end(ENDFORM) ENDTURN ENTER ENTERTAIN EVICT
EXCHANGE FACTION FIND FORGET FORM GIVE GUARD HOLD IDLE JOIN LEAVE MOVE NAME NOAID NOCROSS NOSPOILS
OPTION PASSWORD PILLAGE PREPARE PRODUCE PROMOTE QUIT RESTART REVEAL SAIL SELL SHARE SHOW SPOILS
STEAL STUDY TAKE TAX TEACH TRANSPORT TURN WEAPON WITHDRAW WORK`

Move modes `M_NONE/WALK/RIDE/FLY/SWIM/SAIL`. Special directions: `MOVE_PAUSE=97`, `MOVE_IN=98`,
`MOVE_OUT=99`, `MOVE_ENTER=100+objnum`.

## Order classes

| Class | Fields |
|---|---|
| `Order` | `type` (default NORDERS), `quiet` |
| `MoveOrder` | `advancing`, `dirs` |
| `WithdrawOrder` | `item`, `amount` |
| `GiveOrder` (also TAKE) | `item` (negative = item class), `amount` (-1 unit, -2 all), `except`, `unfinished`, `merge`, `target` |
| `StudyOrder` | `skill`, `days`, `level` (-1 none) |
| `TeachOrder` | target list |
| `ProduceOrder` (also WORK, ENTERTAIN) | `item`, `skill` (-1 = WORK), `productivity`, `target` |
| `BuyOrder` / `SellOrder` | `item`, `num` (-1 = ALL) |
| `AttackOrder`, `EvictOrder` | target lists |
| `BuildOrder` | `target`, `new_building`, `needtocomplete` |
| `SailOrder` | `dirs` |
| `FindOrder` | `find` |
| `StealOrder` | `target`, `item` |
| `AssassinateOrder` | `target` |
| `ForgetOrder` | `skill` |
| `ExchangeOrder` | give item/amount, expect item/amount, `exchangeStatus`, `target` |
| `TurnOrder` | `repeating`, `turnOrders` (raw lines) |
| `CastOrder` and subclasses | `spell`, `level`; `CastMindOrder(id)`, `CastRegionOrder(x,y,z)`, `TeleportOrder(gate, units)`, `CastIntOrder`, `CastUnitsOrder`, `CastTransmuteOrder` |
| `IdleOrder` | |
| `TransportOrder` (also DISTRIBUTE) | `item`, `amount`, `except`, `target` |
| `JoinOrder` | `target`, `overload`, `merge` |

Unit ids (`Game::ParseUnit`, `parseorders.cpp:81`) become `UnitId{unitnum, alias, faction}`: `0`
means unitnum -1; `new N` an alias; `faction F new N` another faction's alias; a number the unit.

## Tokenizer and string semantics (`astring.cpp`, `fileio.cpp`)

- Lines (`Aorders::GetLine`, `fileio.cpp:225`): skip whitespace and blank lines, `getline` up to
  1023 chars. A trailing `\r` is **not** stripped. A longer line sets the fail bit and ends reading.
- `gettoken` (`astring.cpp:181`): split on space/tab; `;` at token start or inside an unquoted token
  ends the line; `"..."` keeps spaces and `;`; an **unmatched quote returns null and wipes the rest
  of the line**; buffer 1024.
- `isEqual` (`astring.cpp:136`): case-insensitive, `_` equals space. Used for every keyword, item,
  skill, object, terrain name and for passwords.
- `value()`: leading digits only, 0 on overflow or non-numeric. `strict_value()`: -1 if any non-digit
  (used by DECLARE).
- `getat()`: turns a leading `@` into a space and returns 1.
- `getlegal()`: drops characters outside the legal set; null if only spaces remain.
- `Trunc(val, back=30)` (`astring.cpp:330`): see `reports-and-templates.md`.

## Order-file state machine (`Game::ParseOrders`, `parseorders.cpp:215-488`)

- `#atlantis N [pass]`: closes an open faction block with `No #END statement given.`; looks up faction
  **N from the line**, not from the file name; password must match unless stored password is `none`;
  monster and guard factions rejected; sets `lastorders` unless `LASTORDERS_MAINTAINED_BY_SCRIPTS`.
  Bad password: everything until the next `#atlantis` is silently ignored.
- `#end`: unwinds the FORM stack (`TURN: without ENDTURN`, `FORM: without END.`).
- `unit N`: unwinds the same way; foreign unit → `fac->Error("N is not your unit.")`; else
  `ClearOrders()`.
- `form A`: no nesting except inside TURN. `ProcessFormOrder` (`parseorders.cpp:2785`) creates the
  unit immediately (`CopyFlags`, `DefaultOrders`, placed in former's object). `@form` sets
  `former->format`, and every line of the block is copied into the **former's** `oldorders`.
- `end`: back to the former unit.
- `turn` / `endturn`: in a real run `ProcessTurnOrder` (`parseorders.cpp:2027`) swallows raw lines
  into a `TurnOrder` with nesting depth, possibly pushing back a `unit`, `#end` or `end` line. In check
  mode it sets `inTurnBlock` and parses lines normally, swapping the month order out and back.
- Other lines: a leading `@` copies the raw line into `oldorders`, then `ProcessOrder`
  (`parseorders.cpp:490`). Comment-only `@` lines are also kept.
- Errors (`ParseError`, `parseorders.cpp:207`): check file if checking, else `unit->Error` prefixed
  `"<unit name>: "`, else `faction->Error`. NPC factions drop errors. Lists capped at 1000 then
  `Too many errors!`.
- Check mode: dummy faction and unit; echoes each line indented by TURN/FORM depth; errors as
  `*** Error: ... ***`; ends with `No errors found.` or `N error(s) found!`.
- `DefaultWorkOrder` (`game.cpp:77`) runs after each faction file over all regions: units of
  non-NPC factions without a month order get WORK (or auto-tax).

## Per-order rules

- Month-long orders (MOVE, ADVANCE, SAIL, STUDY, TEACH, PRODUCE, BUILD, ENTERTAIN, WORK, IDLE, plus TAX
  and PILLAGE when `TAX_PILLAGE_MONTH_LONG`) replace an earlier one with
  `X: Overwriting previous [DELAYED ]month-long order.`. MOVE/ADVANCE/SAIL **append** directions to
  a same-type order; TEACH appends students.
- `ParseDir` (`parseorders.cpp:52`): full or abbreviated directions, `in`, `out`, `pause`/`p`, or a
  number meaning enter object. SAIL accepts only the six directions and `pause`.
- `ParseTF`: `true/false/t/f/on/off/yes/no/1/0`.
- GIVE/TAKE (`parseorders.cpp:2207`): TAKE needs `FROM`; amount `unit` (GIVE only), `all`, or N≥1;
  optional `unfinished` (ships); with `all`, an item class: normal, advanced, trade, man(men),
  monster(s), magic, weapon(s), armor, mount(s), battle, special, food, tool(s), item(s), ship(s);
  `except N` only with `all` plus a specific item.
- Feature-gated (else `X is not a valid order.`): PREPARE (`USE_PREPARE_COMMAND`), WEAPON/ARMOR
  (`USE_WEAPON_ARMOR_COMMAND`), WITHDRAW (`ALLOW_WITHDRAW`), FIND (`HAVE_EMAIL_SPECIAL_COMMANDS`),
  FACTION (faction types), NOCROSS (flying-over-water or swimming items exist).
- Instant at parse time: PASSWORD, OPTION (times/notimes/showattitudes/dontshowattitudes/template
  off|short|long|map), SHOW (over 100 per faction rejected), CLAIM, COMBAT, PREPARE, WEAPON, ARMOR,
  FACTION, DECLARE, NAME, DESCRIBE, ADDRESS, flag orders, QUIT, RESTART (creates the new faction at
  parse time). NAME of a settlement costs `(towntype+1)*CITY_RENAME_COST` and needs a large enough
  owned building.
- BUY accepts `peasant(s)`/`peas` as the region race. SELL/BUY compare `ALL` case-insensitively.
- CAST parsing: `spells.cpp:45`; skill must be castable magic, then per-spell parsers.

## Porting notes

Targets: `crates/atlantis-engine/src/text/` (tokenizer, comparison, legal names) and
`src/orders/` (order model, parser state machine, check mode). Keep the parser operating on a byte
string, since `\r` and 1023-char limits are observable. Check-mode output is verified against the
reference `check` command.
