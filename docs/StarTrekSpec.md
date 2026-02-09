# Star Trek (1971) — Complete Game Specification

This document is the authoritative specification for the classic Star Trek text game
created by Mike Mayfield in 1971. It describes the original game mechanics faithfully,
including known bugs and preserved typos. No programming language code is used —
all formulas are in plain math notation.

**Sources:**

- Mike Mayfield's original HP BASIC source (October 20, 1972 port)
- Michael Birken's 2008 analysis at meatfighter.com/startrek1971
- The `st.c` faithful C port by busfahrer (2024)

---

## 1. Overview

The player commands the starship Enterprise on a mission to destroy all Klingon battle
cruisers in the galaxy within a fixed number of stardates. The galaxy is an 8×8 grid of
quadrants, each containing an 8×8 grid of sectors. The game is turn-based: the player
issues commands, time passes, and Klingons fire back.

The player has 8 ship systems, energy reserves, photon torpedoes, and shields. Starbases
scattered through the galaxy provide resupply when docked. The game ends when all
Klingons are destroyed (victory), the Enterprise is destroyed, energy runs out, or
time expires (all losses).

---

## 2. Coordinate System

### 2.1 Galaxy Grid

The galaxy is an 8×8 grid of **quadrants**, with coordinates (X, Y) where:

- X ranges from 1 to 8, left to right
- Y ranges from 1 to 8, top to bottom

### 2.2 Sector Grid

Each quadrant contains an 8×8 grid of **sectors**, with coordinates (X, Y) using the
same convention (1–8 left-to-right, 1–8 top-to-bottom).

### 2.3 Course / Direction System

Courses are specified as a number from 1 (inclusive) to 9 (exclusive). The directions
form a circular arrangement:

```
        4    3    2
         `.  :  .'
           `.:.'
        5---<*>---1
           .':`.
         .'  :  `.
        6    7    8
```

A course of 9 is undefined, but values may approach 9. Fractional courses are
permitted (e.g., course 1.5 is halfway between direction 1 and direction 2).

The direction vectors for integer courses are:

| Course | Delta-X | Delta-Y |
|--------|---------|---------|
| 1      | +1      |  0      |
| 2      | +1      | -1      |
| 3      |  0      | -1      |
| 4      | -1      | -1      |
| 5      | -1      |  0      |
| 6      | -1      | +1      |
| 7      |  0      | +1      |
| 8      | +1      | +1      |
| 9      | +1      |  0      |

Course 9 has the same vector as course 1 (used for interpolation only).

**Fractional course interpolation:**

Given a course value B, let R = floor(B). The movement vector is:

    delta_x = C[R].x + (C[R+1].x − C[R].x) × (B − R)
    delta_y = C[R].y + (C[R+1].y − C[R].y) × (B − R)

This linearly interpolates between adjacent direction vectors.

### 2.4 Display Symbols

| Symbol | Entity     | Internal Code |
|--------|------------|---------------|
| `<*>`  | Enterprise | 1             |
| `+++`  | Klingon    | 2             |
| `>!<`  | Starbase   | 3             |
| ` * `  | Star       | 4             |
| `   `  | Empty      | 0             |

---

## 3. Game Initialization

### 3.1 Starting Resources

| Resource           | Initial Value |
|--------------------|---------------|
| Energy             | 3000          |
| Photon torpedoes   | 10            |
| Shields            | 0             |
| Klingon shields    | 200 each      |
| Mission time limit | 30 stardates  |

### 3.2 Starting Stardate

The starting stardate is calculated as:

    stardate = floor(random(0..1) × 20 + 20) × 100

This produces a value from 2000 to 3900 in increments of 100.

### 3.3 Starting Position

The Enterprise starts in a random quadrant (each coordinate 1–8) and a random sector
within that quadrant (each coordinate 1–8).

### 3.4 Galaxy Generation

For each of the 64 quadrants in the galaxy:

**Klingon count** — Roll a random number F from 0 to 1:

| Condition     | Klingons | Probability |
|---------------|----------|-------------|
| F > 0.98      | 3        | 2%          |
| F > 0.95      | 2        | 3%          |
| F > 0.80      | 1        | 15%         |
| Otherwise     | 0        | 80%         |

**Starbase count** — Roll a random number F from 0 to 1:

| Condition     | Starbases | Probability |
|---------------|-----------|-------------|
| F > 0.96      | 1         | 4%          |
| Otherwise     | 0         | 96%         |

**Star count** — Random integer from 1 to 8:

    stars = floor(random(0..1) × 8 + 1)

**Galactic record encoding:**

Each quadrant's data is stored as a three-digit number:

    encoded = klingons × 100 + starbases × 10 + stars

### 3.5 Regeneration Guard

After generating the galaxy, if the total number of Klingons is 0 **or** the total
number of starbases is 0, the entire galaxy is regenerated from scratch.

### 3.6 Mission Briefing

After galaxy generation, the game prints:

> `YOU MUST DESTROY` *n* `KINGONS IN` *t* `STARDATES WITH` *b* `STARBASE`[`S`]

Note: "KINGONS" is a typo preserved from the original. The plural "S" on "STARBASE"
is only appended when there are 2 or more starbases.

### 3.7 All Devices Start Undamaged

All 8 device damage values are initialized to 0 (fully operational).

---

## 4. Quadrant Entry

Every time the Enterprise enters a quadrant (including the starting quadrant), the
sector-level map is regenerated. The galactic record stores only counts (encoded as
a 3-digit number); exact sector positions are **not** preserved between visits.

### 4.1 Placement Order

1. The sector map is cleared (all sectors set to empty)
2. The Enterprise is placed at its sector coordinates
3. Klingons are placed at random empty sectors, each assigned shield strength of 200
4. Starbases are placed at random empty sectors
5. Stars are placed at random empty sectors

The random empty sector finder picks random coordinates (1–8, 1–8) repeatedly until
an empty sector is found.

### 4.2 Maximum Klingons Per Quadrant

A quadrant can have at most 3 Klingons. The Klingon data array has 3 slots (index 1–3),
each storing (sector-X, sector-Y, shield-strength).

### 4.3 Red Alert on Entry

When entering a quadrant that contains Klingons, if shields are 200 or less:

> `COMBAT AREA      CONDITION RED`
> `   SHIELDS DANGEROUSLY LOW`

---

## 5. Ship Systems

The Enterprise has 8 systems, indexed 1 through 8:

| Index | Device Name    | Affects                          |
|-------|----------------|----------------------------------|
| 1     | WARP ENGINES   | Navigation (Command 0)           |
| 2     | S.R. SENSORS   | Short Range Scan (Command 1)     |
| 3     | L.R. SENSORS   | Long Range Scan (Command 2)      |
| 4     | PHASER CNTRL   | Phasers (Command 3)              |
| 5     | PHOTON TUBES   | Torpedoes (Command 4)            |
| 6     | DAMAGE CNTRL   | Damage Report (Command 6)        |
| 7     | SHIELD CNTRL   | Shield Control (Command 5)       |
| 8     | COMPUTER       | Library Computer (Command 7)     |

### 5.1 Damage Values

- **0** — Fully operational
- **Negative** — Damaged (magnitude indicates severity)
- **Positive** — Operating above normal

A device is "damaged" (and its associated command blocked) when its value is < 0.

### 5.2 Automatic Repair

Every time a warp navigation move is executed, each damaged device is repaired by 1:

    For each device D[i] where D[i] < 0:
        D[i] = D[i] + 1

### 5.3 Random Damage/Repair Events

After automatic repair, on each navigation move there is a 20% chance of a random
event affecting one system:

1. Roll a random number. If > 0.2, nothing happens (80% chance).
2. Select a random device: index = floor(random(0..1) × 8 + 1)
3. Roll again. If random ≥ 0.5, the device is **improved**; otherwise it is **damaged**.

**Damage severity:** floor(random(0..1) × 5 + 1), producing a value from 1 to 5.

- **Damage:** D[device] = D[device] − severity
- **Improvement:** D[device] = D[device] + severity

Messages printed:

> `DAMAGE CONTROL REPORT:` *device-name* `DAMAGED`

> `DAMAGE CONTROL REPORT:` *device-name* `STATE OF REPAIR IMPROVED`

These messages are preceded and followed by blank lines.

---

## 6. Commands

The game prompts `COMMAND` and accepts a number 0–7. Any other input displays the
command list:

> ```
>    0 = SET COURSE
>    1 = SHORT RANGE SENSOR SCAN
>    2 = LONG RANGE SENSOR SCAN
>    3 = FIRE PHASERS
>    4 = FIRE PHOTON TORPEDOES
>    5 = SHIELD CONTROL
>    6 = DAMAGE CONTROL REPORT
>    7 = CALL ON LIBRARY COMPUTER
> ```

### 6.0 Command 0 — Warp Engine Control (Set Course)

**Input flow:**

1. Prompt: `COURSE (1-9)`
2. If input is 0, return to command prompt
3. If input < 1 or ≥ 9, re-prompt
4. Prompt: `WARP FACTOR (0-8)`
5. If warp factor < 0 or > 8, return to course prompt
6. If warp engines are damaged (D[1] < 0) and warp factor > 0.2:
   > `WARP ENGINES ARE DAMAGED, MAXIMUM SPEED = WARP .2`
   Return to course prompt

**Pre-move combat:**

If Klingons are present in the quadrant, they fire at the Enterprise **before** the
warp move occurs. If the Enterprise is destroyed, the game ends. If energy drops
to 0 or below (with no shields), the "dead in space" sequence triggers.

**Energy check (no Klingons path):**

If there are no Klingons, but energy ≤ 0 and shields < 1:
> `THE ENTERPRISE IS DEAD IN SPACE. IF YOU SURVIVE ALL IMPENDING`
> `ATTACK YOU WILL BE DEMOTED TO THE RANK OF PRIVATE`

If energy ≤ 0 but shields ≥ 1:
> `YOU HAVE` *e* `UNITS OF ENERGY`
> `SUGGEST YOU GET SOME FROM YOUR SHIELDS WHICH HAVE` *s*
> `UNITS LEFT`

**Automatic repair and random events** occur (see Section 5.2 and 5.3).

**Movement:**

The number of movement steps is:

    N = floor(warp_factor × 8)

The ship moves N steps along the course vector, one sector at a time:

    For each step:
        sector_x = sector_x + delta_x
        sector_y = sector_y + delta_y

At each step:

- If the new position is outside the quadrant (< 0.5 or ≥ 8.5 in either coordinate),
  the ship crosses a quadrant boundary (see below).
- If the sector is occupied (not empty), the ship stops one step before:
  > `WARP ENGINES SHUTDOWN AT SECTOR` *x*`,`*y* `DUE TO BAD NAVIGATION`

After movement completes, the sector coordinates are rounded to integers:

    sector_x = floor(sector_x + 0.5)
    sector_y = floor(sector_y + 0.5)

**Energy cost:**

    energy = energy − N + 5

(Cost is N − 5 energy units.)

**Time advancement:**

    If warp_factor ≥ 1: stardate = stardate + 1

Sub-warp movement (warp factor < 1) does **not** advance the stardate.

**Quadrant boundary crossing:**

When the ship exits the current quadrant, the new position is calculated in absolute
galactic coordinates:

    abs_x = quadrant_x × 8 + original_sector_x + delta_x × N
    abs_y = quadrant_y × 8 + original_sector_y + delta_y × N
    new_quadrant_x = floor(abs_x / 8)
    new_quadrant_y = floor(abs_y / 8)
    new_sector_x = floor(abs_x − new_quadrant_x × 8 + 0.5)
    new_sector_y = floor(abs_y − new_quadrant_y × 8 + 0.5)

If new_sector_x = 0: new_quadrant_x = new_quadrant_x − 1, new_sector_x = 8
If new_sector_y = 0: new_quadrant_y = new_quadrant_y − 1, new_sector_y = 8

Quadrant coordinates are clamped to 1–8 range upon quadrant entry.

The stardate always advances by 1 on a quadrant boundary crossing (regardless of warp
factor). The energy cost is the same: N − 5.

After crossing, the new quadrant is entered (see Section 4).

**Time limit check:**

After any time advancement, if the current stardate exceeds the starting stardate +
mission time limit, the game is lost:

> `IT IS STARDATE` *t*

Followed by the remaining-Klingons message (see Section 10).

### 6.1 Command 1 — Short Range Sensor Scan

Triggers the quadrant display (see Section 4 for docking check, which also occurs).

If short range sensors are damaged (D[2] < 0):

> `*** SHORT RANGE SENSORS ARE OUT ***`

Otherwise, displays the sector grid with status information:

```
-=--=--=--=--=--=--=--=-
[row 1 of sectors]        STARDATE  [t]
[row 2 of sectors]        CONDITION [condition]
[row 3 of sectors]        QUADRANT  [qx],[qy]
[row 4 of sectors]        SECTOR    [sx],[sy]
[row 5 of sectors]        ENERGY    [e]
[row 6 of sectors]        SHIELDS   [s]
[row 7 of sectors]        PHOTON TORPEDOES [p]
[row 8 of sectors]
-=--=--=--=--=--=--=--=-
```

Each row shows 8 sectors using the 3-character display symbols. The status information
is printed on specific rows (stardate on line 2, condition on line 3, etc.).

### 6.2 Command 2 — Long Range Sensor Scan

If long range sensors are damaged (D[3] < 0):

> `LONG RANGE SENSORS ARE INOPERABLE`

Otherwise:

> `LONG RANGE SENSOR SCAN FOR QUADRANT` *x*`,`*y*

Displays a 3×3 grid centered on the current quadrant:

```
-------------------
| xxx | xxx | xxx |
-------------------
| xxx | xxx | xxx |
-------------------
| xxx | xxx | xxx |
-------------------
```

Each cell shows the 3-digit encoded quadrant data (klingons × 100 + starbases × 10
+ stars). Quadrants outside the galaxy (coordinates < 1 or > 8) display as 000.

**Computer memory update:** If the computer is undamaged (D[7] ≥ 0), the scanned
quadrant data is recorded to the computer's galactic record. Note: this check uses
device index 7 (SHIELD CNTRL), not index 8 (COMPUTER) — this is a known bug
(see Section 12).

### 6.3 Command 3 — Fire Phasers

**Preconditions:**

If no Klingons in quadrant:
> `SHORT RANGE SENSORS REPORT NO KLINGONS IN THIS QUANDRANT`
(Note: "QUANDRANT" is a typo preserved from the original.)

If phaser control is damaged (D[4] < 0):
> `PHASER CONTROL IS DISABLED`

If computer is damaged (D[7] < 0):
> ` COMPUTER FAILURE HAMPERS ACCURACY`
(Note: leading space is present in the original.)

**Input:**
> `PHASERS LOCKED ON TARGET.  ENERGY AVAILABLE =` *e*
> `NUMBER OF UNITS TO FIRE`

If input ≤ 0, return to command prompt.
If energy − input < 0, re-prompt (back to "PHASERS LOCKED ON TARGET").

**Firing sequence:**

1. Deduct energy: energy = energy − X (where X is units to fire)
2. Klingons fire back at the Enterprise (see Section 8)
3. If computer is damaged (D[7] < 0): X = X × random(0..1) — reduces effectiveness
4. For each Klingon (index 1 to 3) with shields > 0:
   - Calculate hit damage (see Section 7.2)
   - Deduct from Klingon shields: klingon_shields = klingon_shields − hit
   - Print: *hit* `UNIT HIT ON KLINGON AT SECTOR` *x*`,`*y*
   - Print: `   (`*remaining*` LEFT)`
   - If klingon_shields ≤ 0, destroy the Klingon (see Section 7.5)
   - If all Klingons in galaxy destroyed: victory

If energy < 0 after combat, the Enterprise is destroyed.

### 6.4 Command 4 — Fire Photon Torpedoes

**Preconditions:**

If photon tubes are damaged (D[5] < 0):
> `PHOTON TUBES ARE NOT OPERATIONAL`

If torpedoes = 0:
> `ALL PHOTON TORPEDOES EXPENDED`

**Input:**
> `TORPEDO COURSE (1-9)`

If input is 0, return to command prompt.
If input < 1 or ≥ 9, re-prompt.

**Firing:**

1. Calculate direction vector using the same interpolation formula as navigation
2. Decrement torpedo count: torpedoes = torpedoes − 1
3. Print: `TORPEDO TRACK:`
4. Starting from the Enterprise's sector position, advance one step at a time
   along the direction vector:

For each step:
- x = x + delta_x
- y = y + delta_y
- If outside quadrant (< 0.5 or ≥ 8.5): print `TORPEDO MISSED` and stop
- Print the current position: *x*`,`*y*
- Check what occupies the sector at (round(x), round(y)):
  - **Empty (0):** continue to next step
  - **Klingon (2):** Print `*** KLINGON DESTROYED ***`, destroy the Klingon,
    update counts and galactic record. If all Klingons in galaxy destroyed: victory.
  - **Star (4):** Print `YOU CAN'T DESTROY STARS SILLY` and stop
  - **Starbase (3):** Print `*** STAR BASE DESTROYED ***  .......CONGRATULATIONS`,
    decrement local starbase count, clear sector, update galactic record, and stop

After torpedo resolution (hit or miss), surviving Klingons fire back at the Enterprise
(see Section 8).

**Note:** Torpedoes follow an exact trajectory with no random deviation. The torpedo
hits exactly what the course vector leads to.

### 6.5 Command 5 — Shield Control

If shield control is damaged (D[7] < 0):
> `SHIELD CONTROL IS NON-OPERATIONAL`

Note: This checks device 7 (SHIELD CNTRL), which is correct.

**Input:**
> `ENERGY AVAILABLE =` *(energy + shields)*
> `NUMBER OF UNITS TO SHIELDS`

If input ≤ 0, return to command prompt.
If energy + shields − input < 0, re-prompt.

**Effect:**

    energy = energy + shields − new_shield_value
    shields = new_shield_value

Total energy (energy + shields) is conserved.

### 6.6 Command 6 — Damage Control Report

If damage control is damaged (D[6] < 0):
> `DAMAGE CONTROL REPORT IS NOT AVAILABLE`

Otherwise, prints all 8 devices and their repair states:

> ```
> DEVICE        STATE OF REPAIR
> WARP ENGINES  [value]
> S.R. SENSORS  [value]
> L.R. SENSORS  [value]
> PHASER CNTRL  [value]
> PHOTON TUBES  [value]
> DAMAGE CNTRL  [value]
> SHIELD CNTRL  [value]
> COMPUTER      [value]
> ```

Values are printed as integers (truncated, not rounded).

### 6.7 Command 7 — Library Computer

If computer is damaged (D[8] < 0):
> `COMPUTER DISABLED`

Otherwise:
> `COMPUTER ACTIVE AND AWAITING COMMAND`

Accepts option 0, 1, or 2. Any other input displays:

> ```
> FUNCTIONS AVAILABLE FROM COMPUTER
>    0 = CUMULATIVE GALATIC RECORD
>    1 = STATUS REPORT
>    2 = PHOTON TORPEDO DATA
> ```

Note: "GALATIC" is a typo preserved from the original.

#### Option 0 — Cumulative Galactic Record

> `COMPUTER RECORD OF GALAXY FOR QUADRANT` *x*`,`*y*

Displays an 8×8 grid of all quadrants from the computer's memory:

```
-------------------------------------------------
| xxx | xxx | xxx | xxx | xxx | xxx | xxx | xxx |
-------------------------------------------------
| xxx | xxx | xxx | xxx | xxx | xxx | xxx | xxx |
-------------------------------------------------
[... 6 more rows ...]
-------------------------------------------------
```

Unscanned quadrants display as 000. Data is populated by long range sensor scans
(when the computer is undamaged — see Section 6.2 note about the D[7] bug).

#### Option 1 — Status Report

> ```
>    STATUS REPORT
>
> NUMBER OF KLINGONS LEFT  = [count]
> NUMBER OF STARDATES LEFT = [remaining]
> NUMBER OF STARBASES LEFT = [count]
> ```

Stardates remaining = (starting_stardate + mission_time_limit) − current_stardate.

**Falls through to Command 6** (Damage Control Report) — the status report always
displays the damage control report immediately after the status information.

#### Option 2 — Photon Torpedo Data

For each Klingon in the quadrant (up to 3) with shields > 0, calculates and displays
the direction and distance from the Enterprise to that Klingon:

> `DIRECTION =` *d.dd*
> `DISTANCE  =` *d.dd*

The direction calculation uses the algorithm described in Section 7.4.

After displaying torpedo data for all Klingons, the game offers:

> `ENTER 1 TO USE THE CALCULATOR`

If the player enters 1, the game prompts:

> `YOU ARE AT QUADRANT` *qx*`,`*qy* `SECTOR` *sx*`,`*sy*
> `SHIP'S & TARGET'S COORDINATES ARE`

The player inputs four comma-separated values (source_x, source_y, target_x, target_y)
and the game calculates direction and distance between them, additionally showing the
warp units:

> `   (`*n* `WARP UNIT`[`S`]`)`

The plural "S" is appended unless the warp units = 1. The warp unit count is the
larger of |delta_x| and |delta_y|.

---

## 7. Formulas

### 7.1 Distance Between Enterprise and Klingon

    distance = sqrt((klingon_x − enterprise_x)² + (klingon_y − enterprise_y)²)

Uses sector coordinates. This is standard Euclidean distance.

### 7.2 Phaser Hit Damage (Enterprise Firing)

    hit = (energy_fired / num_klingons_in_quadrant / distance) × (2 × random(0..1))

Where:
- energy_fired = total energy units the player chose to fire
  (if computer is damaged, this is further multiplied by random(0..1) before use)
- num_klingons_in_quadrant = number of Klingons at time of firing
- distance = Euclidean distance from Enterprise to this Klingon (Section 7.1)
- random(0..1) = uniform random number from 0 to 1

Energy is divided equally among all Klingons; damage increases with proximity
and has a random factor from 0 to 2.

### 7.3 Klingon Attack Damage (Klingon Firing)

    hit = (klingon_shields / distance) × (2 × random(0..1))

Where:
- klingon_shields = that Klingon's current remaining shield strength
- distance = Euclidean distance from that Klingon to the Enterprise (Section 7.1)
- random(0..1) = uniform random number from 0 to 1

Damage is deducted from the Enterprise's shield value. If shields drop below 0,
the Enterprise is destroyed.

### 7.4 Direction Calculation (Torpedo Data / Calculator)

Given source position (A, B) and target position (X, W):

    delta_x = floor(X − A)
    delta_y = floor(B − W)

Note: delta_y is **source minus target** (inverted), because the Y axis points
downward on screen but "up" on the course diagram.

The direction is then computed using the following rules:

**Case 1: delta_x ≥ 0 and delta_y ≥ 0** (target is right and/or up)
- If delta_x > 0 or delta_y > 0: base = 1
  - If |delta_y| ≤ |delta_x|: direction = 1 + |delta_y| / |delta_x|
  - Else: direction = 1 + (|delta_y| − |delta_x| + |delta_y|) / |delta_y|
- If both are 0: base = 5 (arbitrary; same position)

**Case 2: delta_x < 0 and delta_y > 0** (target is left and up)
- Base = 3
  - If |delta_y| ≥ |delta_x|: direction = 3 + |delta_x| / |delta_y|
  - Else: direction = 3 + (|delta_x| − |delta_y| + |delta_x|) / |delta_x|

**Case 3: delta_x ≥ 0 and delta_y < 0** (target is right and/or down)
- Base = 7
  - If |delta_y| ≥ |delta_x|: direction = 7 + |delta_x| / |delta_y|
  - Else: direction = 7 + (|delta_x| − |delta_y| + |delta_x|) / |delta_x|

**Case 4: delta_x < 0 and delta_y ≤ 0** (target is left and down)
- If delta_y = 0 and delta_x = 0: handled in Case 1
- Base = 5
  - If |delta_y| ≤ |delta_x|: direction = 5 + |delta_y| / |delta_x|
  - Else: direction = 5 + (|delta_y| − |delta_x| + |delta_y|) / |delta_y|

**Distance:**

    distance = sqrt(delta_x² + delta_y²)

**Important:** This algorithm uses ratio-based linear interpolation, not trigonometry.
It produces systematic inaccuracies for directions that are not exact multiples of 45°.

### 7.5 Klingon Destruction

When a Klingon's shields drop to 0 or below (from phasers):

> `*** KLINGON AT SECTOR` *x*`,`*y* `DESTROYED ***`

When a Klingon is hit by a torpedo:

> `*** KLINGON DESTROYED ***`

In both cases:
- Local Klingon count decremented
- Galaxy-wide Klingon count decremented
- Sector cleared to empty
- Galactic record updated: klingons × 100 + starbases × 10 + stars

### 7.6 Navigation Energy Cost

    cost = N − 5

Where N = floor(warp_factor × 8). The "−5" means very short movements are free or
even gain energy.

### 7.7 Efficiency Rating

    rating = (initial_total_klingons / elapsed_stardates) × 1000

Where elapsed_stardates = current_stardate − starting_stardate.

Displayed as an integer (truncated).

---

## 8. Combat

### 8.1 When Klingons Attack

Klingons in the current quadrant fire at the Enterprise in these situations:

1. **Before a warp move** (Command 0) — if Klingons are present, they fire before
   the ship moves
2. **After phasers fire** (Command 3) — Klingons fire back before phaser damage
   is applied to them
3. **After a torpedo is resolved** (Command 4) — surviving Klingons fire back

Each Klingon fires independently, applying the formula from Section 7.3.

### 8.2 Klingon Attack Display

For each Klingon with shields > 0:

> *hit* `UNIT HIT ON ENTERPRISE FROM SECTOR` *x*`,`*y*
> `   (`*remaining* `LEFT)`

Where *remaining* = max(0, shields_after_hit).

### 8.3 Docking Protection

If the Enterprise is docked (condition = DOCKED), Klingon attacks are blocked:

> `STAR BASE SHIELDS PROTECT THE ENTERPRISE`

### 8.4 Enterprise Destruction

If shields drop below 0 from a Klingon attack, the Enterprise is destroyed immediately
(see Section 10).

---

## 9. Docking

### 9.1 Adjacency Detection

Docking occurs when the Enterprise is adjacent to (or in the same sector as) a starbase.
The check examines all sectors in the 3×3 area centered on the Enterprise:

    For x from enterprise_x − 1 to enterprise_x + 1:
      For y from enterprise_y − 1 to enterprise_y + 1:
        If within bounds (1–8) and sector contains starbase: DOCKED

### 9.2 Docking Effects

When docked:

| Resource   | Set To |
|------------|--------|
| Energy     | 3000   |
| Torpedoes  | 10     |
| Shields    | 0      |
| Condition  | DOCKED |

Message printed:
> `SHIELDS DROPPED FOR DOCKING PURPOSES`

### 9.3 What Docking Does NOT Do

Docking does **not** repair damaged devices. Devices only repair via the automatic
+1 repair on navigation moves and random repair events.

### 9.4 Condition Codes

Condition is evaluated each time the short range scan is performed:

| Code   | Condition                                              |
|--------|--------------------------------------------------------|
| GREEN  | No Klingons in quadrant AND energy ≥ 300               |
| YELLOW | No Klingons in quadrant AND energy < 300               |
| RED    | Klingons present in quadrant                           |
| DOCKED | Adjacent to a starbase (overrides all others)          |

The energy threshold for YELLOW is initial_energy × 0.1 = 3000 × 0.1 = 300.

---

## 10. Win/Lose Conditions

### 10.1 Victory — All Klingons Destroyed

When the galaxy-wide Klingon count reaches 0:

> ```
>
> THE LAST KLIGON BATTLE CRUISER IN THE GALAXY HAS BEEN DESTROYED
> THE FEDERATION HAS BEEN SAVED !!!
>
> YOUR EFFICIENCY RATING = [rating]
> ```

Note: "KLIGON" is a typo preserved from the original.

The efficiency rating formula is given in Section 7.7.

### 10.2 Loss — Enterprise Destroyed

When shields drop below 0 from Klingon fire:

> ```
>
> THE ENTERPRISE HAS BEEN DESTROYED. THE FEDERATION WILL BE CONQUERED
> THERE ARE STILL [n] KLINGON BATTLE CRUISERS
> ```

### 10.3 Loss — Time Expired

When the stardate exceeds starting_stardate + mission_time_limit:

> ```
>
> IT IS STARDATE [t]
> THERE ARE STILL [n] KLINGON BATTLE CRUISERS
> ```

### 10.4 Loss — Dead in Space

When energy ≤ 0 and shields < 1:

> ```
> THE ENTERPRISE IS DEAD IN SPACE. IF YOU SURVIVE ALL IMPENDING
> ATTACK YOU WILL BE DEMOTED TO THE RANK OF PRIVATE
> ```

Then all remaining Klingons in the quadrant fire repeatedly until either:
- All their shots miss (Enterprise survives, demoted to private) → falls through
  to "THERE ARE STILL *n* KLINGON BATTLE CRUISERS"
- The Enterprise is destroyed → "THE ENTERPRISE HAS BEEN DESTROYED..." message

After any loss, the game immediately restarts a new game.

---

## 11. Complete Message Reference

### 11.1 Game Start

| Context              | Message |
|----------------------|---------|
| Title                | `STAR TREK` (centered) |
| Instruction prompt   | `ENTER 1 OR 2 FOR INSTRUCTIONS (ENTER 2 TO PAGE)` |
| Seed prompt          | `ENTER SEED NUMBER` |
| Initializing         | `INITIALIZING...` |
| Mission briefing     | `YOU MUST DESTROY` *n* `KINGONS IN` *t* `STARDATES WITH` *b* `STARBASE`[`S`] |

### 11.2 Navigation (Command 0)

| Context              | Message |
|----------------------|---------|
| Course prompt        | `COURSE (1-9)` |
| Warp prompt          | `WARP FACTOR (0-8)` |
| Engines damaged      | `WARP ENGINES ARE DAMAGED, MAXIMUM SPEED = WARP .2` |
| Blocked by object    | `WARP ENGINES SHUTDOWN AT SECTOR` *x*`,`*y* `DUE TO BAD NAVIGATION` |
| Low energy hint      | `YOU HAVE` *e* `UNITS OF ENERGY` |
| Shield suggestion    | `SUGGEST YOU GET SOME FROM YOUR SHIELDS WHICH HAVE` *s* |
|                      | `UNITS LEFT` |

### 11.3 Short Range Scan (Command 1)

| Context              | Message |
|----------------------|---------|
| Sensors damaged      | `*** SHORT RANGE SENSORS ARE OUT ***` |
| Border               | `-=--=--=--=--=--=--=--=-` |
| Stardate label       | `STARDATE` |
| Condition label      | `CONDITION` |
| Condition values     | `GREEN` / `YELLOW` / `RED` / `DOCKED` |
| Quadrant label       | `QUADRANT` |
| Sector label         | `SECTOR` |
| Energy label         | `ENERGY` |
| Shields label        | `SHIELDS` |
| Torpedoes label      | `PHOTON TORPEDOES` |

### 11.4 Long Range Scan (Command 2)

| Context              | Message |
|----------------------|---------|
| Sensors damaged      | `LONG RANGE SENSORS ARE INOPERABLE` |
| Header               | `LONG RANGE SENSOR SCAN FOR QUADRANT` *x*`,`*y* |
| Grid border          | `-------------------` |

### 11.5 Phasers (Command 3)

| Context              | Message |
|----------------------|---------|
| No targets           | `SHORT RANGE SENSORS REPORT NO KLINGONS IN THIS QUANDRANT` |
| Phasers damaged      | `PHASER CONTROL IS DISABLED` |
| Computer hampered    | ` COMPUTER FAILURE HAMPERS ACCURACY` |
| Energy prompt        | `PHASERS LOCKED ON TARGET.  ENERGY AVAILABLE =` *e* |
| Fire prompt          | `NUMBER OF UNITS TO FIRE` |
| Hit on Klingon       | *hit* `UNIT HIT ON KLINGON AT SECTOR` *x*`,`*y* |
| Klingon remaining    | `   (`*shields* `LEFT)` |
| Klingon destroyed    | `*** KLINGON AT SECTOR` *x*`,`*y* `DESTROYED ***` |

### 11.6 Torpedoes (Command 4)

| Context              | Message |
|----------------------|---------|
| Tubes damaged        | `PHOTON TUBES ARE NOT OPERATIONAL` |
| No torpedoes         | `ALL PHOTON TORPEDOES EXPENDED` |
| Course prompt        | `TORPEDO COURSE (1-9)` |
| Track header         | `TORPEDO TRACK:` |
| Hit Klingon          | `*** KLINGON DESTROYED ***` |
| Hit star             | `YOU CAN'T DESTROY STARS SILLY` |
| Hit starbase         | `*** STAR BASE DESTROYED ***  .......CONGRATULATIONS` |
| Missed               | `TORPEDO MISSED` |

### 11.7 Shields (Command 5)

| Context              | Message |
|----------------------|---------|
| Control damaged      | `SHIELD CONTROL IS NON-OPERATIONAL` |
| Energy prompt        | `ENERGY AVAILABLE =` *(energy + shields)* |
| Input prompt         | `NUMBER OF UNITS TO SHIELDS` |

### 11.8 Damage Report (Command 6)

| Context              | Message |
|----------------------|---------|
| Report unavailable   | `DAMAGE CONTROL REPORT IS NOT AVAILABLE` |
| Header               | `DEVICE        STATE OF REPAIR` |
| Device names         | `WARP ENGINES`, `S.R. SENSORS`, `L.R. SENSORS`, `PHASER CNTRL`, `PHOTON TUBES`, `DAMAGE CNTRL`, `SHIELD CNTRL`, `COMPUTER` |

### 11.9 Library Computer (Command 7)

| Context              | Message |
|----------------------|---------|
| Computer damaged     | `COMPUTER DISABLED` |
| Prompt               | `COMPUTER ACTIVE AND AWAITING COMMAND` |
| Invalid option       | `FUNCTIONS AVAILABLE FROM COMPUTER` |
|                      | `   0 = CUMULATIVE GALATIC RECORD` |
|                      | `   1 = STATUS REPORT` |
|                      | `   2 = PHOTON TORPEDO DATA` |
| Option 0 header      | `COMPUTER RECORD OF GALAXY FOR QUADRANT` *x*`,`*y* |
| Option 0 border      | `-------------------------------------------------` |
| Option 1 header      | `   STATUS REPORT` |
| Option 1 Klingons    | `NUMBER OF KLINGONS LEFT  =` *n* |
| Option 1 stardates   | `NUMBER OF STARDATES LEFT =` *n* |
| Option 1 starbases   | `NUMBER OF STARBASES LEFT =` *n* |
| Option 2 direction   | `DIRECTION =` *d.dd* |
| Option 2 distance    | `DISTANCE  =` *d.dd* |
| Calculator prompt    | `ENTER 1 TO USE THE CALCULATOR` |
| Calculator position  | `YOU ARE AT QUADRANT` *qx*`,`*qy* `SECTOR` *sx*`,`*sy* |
| Calculator input     | `SHIP'S & TARGET'S COORDINATES ARE` |
| Warp units           | `   (`*n* `WARP UNIT`[`S`]`)` |

### 11.10 Combat

| Context              | Message |
|----------------------|---------|
| Hit on Enterprise    | *hit* `UNIT HIT ON ENTERPRISE FROM SECTOR` *x*`,`*y* |
| Enterprise shields   | `   (`*shields* `LEFT)` |
| Docked protection    | `STAR BASE SHIELDS PROTECT THE ENTERPRISE` |
| Docking message      | `SHIELDS DROPPED FOR DOCKING PURPOSES` |

### 11.11 Alerts and Status

| Context              | Message |
|----------------------|---------|
| Red alert            | `COMBAT AREA      CONDITION RED` |
| Low shields          | `   SHIELDS DANGEROUSLY LOW` |

### 11.12 Random Events

| Context              | Message |
|----------------------|---------|
| Device damaged       | `DAMAGE CONTROL REPORT:` *device* `DAMAGED` |
| Device improved      | `DAMAGE CONTROL REPORT:` *device* `STATE OF REPAIR IMPROVED` |

### 11.13 Game Over

| Context              | Message |
|----------------------|---------|
| Victory              | `THE LAST KLIGON BATTLE CRUISER IN THE GALAXY HAS BEEN DESTROYED` |
|                      | `THE FEDERATION HAS BEEN SAVED !!!` |
|                      | `YOUR EFFICIENCY RATING =` *rating* |
| Enterprise destroyed | `THE ENTERPRISE HAS BEEN DESTROYED. THE FEDERATION WILL BE CONQUERED` |
| Remaining Klingons   | `THERE ARE STILL` *n* `KLINGON BATTLE CRUISERS` |
| Time expired         | `IT IS STARDATE` *t* |
| Dead in space        | `THE ENTERPRISE IS DEAD IN SPACE. IF YOU SURVIVE ALL IMPENDING` |
|                      | `ATTACK YOU WILL BE DEMOTED TO THE RANK OF PRIVATE` |

### 11.14 Command Menu

| Message |
|---------|
| `   0 = SET COURSE` |
| `   1 = SHORT RANGE SENSOR SCAN` |
| `   2 = LONG RANGE SENSOR SCAN` |
| `   3 = FIRE PHASERS` |
| `   4 = FIRE PHOTON TORPEDOES` |
| `   5 = SHIELD CONTROL` |
| `   6 = DAMAGE CONTROL REPORT` |
| `   7 = CALL ON LIBRARY COMPUTER` |

---

## 12. Notable Original Bugs and Quirks

### 12.1 Shield Control / Computer Damage Index Mixup

The game has a confusing relationship between device indices 7 and 8:

- **D[7] = SHIELD CNTRL** — Controls: shield command (correct), phaser accuracy
  reduction (should be D[8]), and long range scan recording to computer memory
  (should be D[8])
- **D[8] = COMPUTER** — Controls: library computer command only

This means:
- If SHIELD CNTRL is damaged, shields are unavailable **and** phasers lose accuracy
  **and** long range scans don't update the computer's galactic record
- If COMPUTER is damaged, only the library computer command is blocked — phasers
  and LRS recording still work fine

### 12.2 Preserved Typos

| Location               | Typo        | Correct     |
|------------------------|-------------|-------------|
| Mission briefing       | `KINGONS`   | KLINGONS    |
| Victory message        | `KLIGON`    | KLINGON     |
| No Klingons in quadrant| `QUANDRANT` | QUADRANT    |
| Computer option 0      | `GALATIC`   | GALACTIC    |
| Help text              | `INTERGER`  | INTEGER     |
| Help text              | `PERTINATE` | PERTINENT   |
| Help text              | `TEMPORARALY` | TEMPORARILY |

### 12.3 Sector Positions Not Preserved

Leaving and re-entering a quadrant regenerates all entity positions randomly.
The galactic record only stores counts, not positions. This means a Klingon you
were fighting may appear in a completely different sector if you leave and return.

### 12.4 FND Function Parameter Unused

The distance function takes a parameter D but ignores it entirely, instead using
global state (the current Klingon index and Enterprise position).

### 12.5 Energy Can Go Negative Briefly

The energy cost formula (N − 5) can result in negative costs for very short
movements, effectively granting free energy. A warp factor of 0.5 gives N = 4,
so the cost is 4 − 5 = −1 (gaining 1 energy).

### 12.6 No Input Validation on Seed

The seed value accepts any number. Negative numbers are converted to positive
via absolute value, then truncated to integer.

### 12.7 Dead-in-Space Loop

When the Enterprise is dead in space, Klingons fire repeatedly in a loop. Each
Klingon attack involves the random damage formula, so it's theoretically possible
(though unlikely with large numbers of Klingons) for the Enterprise to survive
all attacks if every shot happens to roll 0 damage.

### 12.8 Status Report Falls Through to Damage Report

Computer Option 1 (Status Report) unconditionally falls through to Command 6
(Damage Control Report), displaying device states after the status information.
This occurs even if the damage control system (D[6]) would normally block the
damage report — the fall-through bypasses the damage control check.

### 12.9 Navigation Delta Rounding

In the direction calculation (Section 7.4), the deltas use floor() which can
introduce off-by-one targeting for certain positions, compounding the systematic
inaccuracy from the ratio-based approach.
