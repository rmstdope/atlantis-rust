# Movement and sailing

Purpose: the phased movement algorithm, move points and fleet sailing, so the Rust movement phase moves
units into the same regions in the same order.

## `RunMovementOrders` (`monthorders.cpp:38`)

For `phase = 0 .. MAX_SPEED-1` (8 in both rulesets):
1. `DoMoveEnter` for all units (`monthorders.cpp:1637`).
2. Fleets whose captain has `phase < current` run `Do1SailOrder` (`monthorders.cpp:217`).
3. Units run `DoAMoveOrder` (`monthorders.cpp:1722`) for MOVE and ADVANCE.
4. `DoMovementAttacks` (can start battles).

Afterwards one more Enter pass; unfinished moves are re-queued into `oldorders` with
`insufficient movement points` and stored in `savedmovement` / `savedmovedir`.

The loops iterate region → object → unit while units move between regions; units that arrive in a
region later in the list are guarded by the `u->phase` check. See `forlist` semantics in
`data-model.md`.

## Move points

- Each call adds `CalcMovePoints` (speed up to `MAX_SPEED`; +`FLEET_WIND_BOOST` when flying with the
  wind skill). A full turn gives speed × `MAX_SPEED` points.
- One hex costs `MoveCost * MAX_SPEED`. Start value `movepoints = PHASED_MOVE_OFFSET % MAX_SPEED`.
- `ARegion::MoveCost` (`aregion.cpp:1744`): weather ×2 (blizzard 10), roads halve walk/ride cost,
  minimum 1.
- Saved movement carries over when moving in the same direction as last turn (`monthorders.cpp:1906`).
- `Unit::MoveType` picks walk/ride/fly/swim/sail from capacity vs weight (`Weight`,
  `WalkingCapacity`, `RidingCapacity`, `FlyingCapacity`, `SwimmingCapacity`).

## Gateways (Nexus exits when `START_CITIES_EXIST == 0`)

Moving through a gateway (`monthorders.cpp:1746`) chooses a random town of matching terrain. Bug: at
candidate level 4 the destination is never reassigned.

## Sailing (`Do1SailOrder`, `monthorders.cpp:217`)

Checks crew skill against `GetFleetSize` (ship weight / 50), capacity, inland sailing, `SailThroughCheck`
(`PREVENT_SAIL_THROUGH`, `ALLOW_TRIVIAL_PORTAGE`), and guards. Fleet speed and capacity:
`object.cpp:560-800`. `FleetSailingSkill`, `FLEET_CREW_BOOST`, `FLEET_LOAD_BOOST`. After movement:
`SinkUncrewedFleets`, `DrownUnits`.

## Porting notes

Target: `crates/atlantis-engine/src/turn/movement.rs`. This is the area most exposed to the
list-mutation semantics; write down, per loop, which units are visited when units move, and test with
fixtures that contain multiple moving units in one region.
