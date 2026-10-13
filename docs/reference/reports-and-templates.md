# Reports, templates, Times and stdout

Purpose: the structure and formatting of every text the engine writes, so the Rust `report` and
`events` modules produce byte-identical output.

## Output files

| File | Writer |
|---|---|
| `report.<n>` | `Game::WriteReport` → `Faction::WriteReport` (`faction.cpp:363-657`) |
| `template.<n>` | `Game::WriteTemplates` → `Faction::WriteTemplate` (`faction.cpp:660`), `ARegion::WriteTemplate` (`aregion.cpp:1504`), `template.cpp` |
| `times.<k>` | `WriteTimesArticle` (`game.cpp:2130`), `Events::Write` (`events.cpp:345`) |
| stdout | `Awrite`, `Adot` (`gameio.cpp`); captured as `engine-output.txt` in fixtures |

## Line formatting

- `Areport::PutStr` (`fileio.cpp:278-296`): prefix two spaces per tab level, then `Trunc(70)`.
  Continuation lines get two extra spaces plus the tab indent, and are truncated again. With the
  comment flag every physical line starts with `;`.
- `AString::Trunc(val, back=30)` (`astring.cpp:330-349`): returns null if length ≤ val. Otherwise
  breaks at the first `\n` or `\r` within the first `val` chars; else at the last space scanning from
  index `val` down to `val-back+1`; else hard-cuts at `val`. It writes `\0` without updating `len`.
- `Arules` (rules HTML and Times) wraps at 78/70.

## Faction report sections (`faction.cpp:363-657`)

1. `;Treasury:` table if `FACTION_STATISTICS` (rank, max, total per item across factions).
2. `Atlantis Report For:`, name with faction type string (e.g. `(War 1, Trade 1, Magic 1)`), then
   `<Month>, Year <n>`.
3. Engine version and ruleset version lines.
4. Notes: Times not sent, password `REMINDER`, inactivity `WARNING` within 3 turns of
   `MAX_INACTIVE_TURNS`, eliminated/won/restarted/game-over.
5. `Faction Status:`: martial-merged prints `Regions: x (y)`; otherwise `Tax Regions`,
   `Trade Regions`; then Quartermasters, Tacticians, Mages, Apprentices as applicable.
6. `Errors during turn:`, `Battles during turn:` (`Battle::Report`), `Events during turn:`.
7. `Skill reports:` (`ShowSkill::Report`, `skillshows.cpp:87`), `Item reports:`, `Object reports:`.
8. `Declared Attitudes (default X):` then one line per attitude, then `Unclaimed silver: N.`
9. Each region in `present_regions` (`MakeFactionReportLists`, `game.cpp:1163`): regions with own
   units, farsight, or units passing through.

## Region block (`ARegion::WriteReport`, `aregion.cpp:1295`)

1. Header `terrain (x,y[,z <level>]) in Name[, contains Town [city]], N peasants (race), $wealth.`
   then a dash line. The z format depends on `EASIER_UNDERWORLD` (`ShortPrint`, `aregion.cpp:684`).
2. Weather line (only if `WEATHER_EXISTS`).
3. Nexus description text.
4. `Wages:`, `Wanted:`, `For Sale:`, `Entertainment available:`, `Products:`.
5. `Exits:`.
6. Gate line (if the faction has a unit with Gate Lore).
7. Objects (`Object::Report`, `object.cpp:283`) with units listed under each. Buildings:
   `+ Name [n] : type, needs N / about to decay / ...; describe.` Fleets: load, sailors, speed,
   sail directions.

Passing units see what `TRANSIT_REPORT` allows. `present = Present(fac) || fac->IsNPC()`.

## Unit lines (`unit.cpp:429`, battle form `unit.cpp:643`)

- Own units start `* `; others `- `, or with `showunitattitudes` an attitude sign (`= : - % !`).
- Visibility compares viewer observation with unit stealth, considering reveal and guard.
- Own units show flags, items, `Weight`, `Capacity f/r/w/s`, `Skills`, combat spell, ready items,
  `Can Study:`, `Has visited ...`.

## GM report

If faction 1 (Guardsmen) is NPC and `GM_REPORT` is on (both rulesets), or on turn 1, `report.1` holds
every skill at levels 1-5, every item and object description, and every region at observation 10.

## Templates (`template.cpp`)

- Only if `temformat != OFF`. Starts `#atlantis N "pass"`, ends `#end`.
- Per region `;*** region ***`. Map format adds a 13-line ASCII hex map with Tax/Ente/Wages/Gate
  (`WriteTemplateHeader`, `template.cpp:299`); `ter_fill` is hard-coded by terrain enum order.
- Per unit: `unit N`; in long or map format a `;`-commented `TemplateReport()`; then `oldorders`
  (indented inside FORM/TURN); then stored TurnOrders. If the unit has no repeating month order, the
  first TURN block's lines are emitted **unwrapped** (they become next turn's orders); `@TURN` blocks
  are re-emitted. The engine creates TurnOrders too, e.g. PRODUCE with an unfinished target re-queues
  `PRODUCE n ITEM` (`monthorders.cpp:1194`).

## Battle text

`Battle::AddLine` accumulates: attack line, `Attackers:`/`Defenders:` rosters, rounds,
routed/destroyed, statistics (if `BATTLE_LOG_LEVEL >= DETAILED`), casualties, spoils. Attached to each
involved faction (`battle.cpp:1163-1171`). See `combat.md` for text quirks.

## Times / world events (`events.h`, `events.cpp`, `events-battle.cpp`, `events-assassination.cpp`)

- World news, not faction events. `AddBattleFact`, `AddAssassinationFact` (`battle.cpp:426,487`)
  record facts with landmark locations, only when `WORLD_EVENTS`.
- `Events::Write` (`events.cpp:345`): per category (battle, city capture, monster hunt, monster
  aggression, assassination) sort by score, keep top 10, pick `getrandom(3)+3` at random, assemble
  text with random `oneOf` word choices, wrap at 65 inside a 77-column ASCII scroll frame.
- `BattleFact::GetEvents`: defender faction 1 = city capture; defender faction 2 = monster hunt;
  attacker faction 2 = monster aggression; else PvP.
- `WriteTimesArticle` (`game.cpp:2130`) names the file `times.<getrandom(10000)>`, redrawing while
  the name exists. Other writers: NewOrigins `CheckVictory`, Blasphemous Ritual (`spells.cpp:2094`).

## stdout

Banner lines, load lines (`Saved Game Engine Version: ...`, `Reading the regions...`, `Setting up the
neighbors...`, `Setting Up Turn...`, `Reading the Gamemaster File...`, `Reading the Orders File...`,
`QUITting Inactive Factions...`, `Running the Turn...`, ...), `Adot()` progress, and ruleset printf
output such as quest reward lines. Compare against `engine-output.txt`.

## Porting notes

Targets: `crates/atlantis-engine/src/report/` (faction, region, unit, object, templates, GM report,
shows), `src/events/` (Times), `src/text/` (`ReportWriter` with tab and `Trunc` semantics). Copy every
literal verbatim. `TurnOutput` carries reports and templates keyed by faction number, Times articles
keyed by file name, and the stdout text.
