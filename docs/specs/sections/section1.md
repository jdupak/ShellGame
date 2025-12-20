# Section 1 Specification – "Where Am I?"

## 1. Purpose & Scope
Section 1 now focuses even more intensely on raw navigation fluency:
- Repeated use of `pwd` after movement.
- Conscious directory listing with `ls`.
- Relative movement (`cd name`, `cd ..`, multi-level ascent).
- Maze following + instruction parsing.
- Absolute path construction from root (before any home-specific tasks).
- Manual root→home walk (stepwise, verified).
- Hidden entries only near the end.
- Optional structural visualization with `tree` as last level.

Deliberate exclusions in this section:
- No file content reading via `cat`, `head`, etc. (Only names, counts, and path reasoning.)
- No assumptions about home being `/home/<user>`; we rely on `$HOME`.
- No symlinks, permissions, creation, deletion.

Estimated Time: 12–15 minutes (core Levels 1.1–1.10) + optional Level 1.11 (~2–3 minutes).

<!-- REVISED TIME & PEDAGOGY -->
> Time Calibration: Target 10 minutes average; slow path 8 minutes using Levels 1.1–1.7 (core). Levels 1.8–1.10 become “Extension” and 1.11 remains “Optional Visualization”.
> Core Levels (advancement-critical): 1.1–1.7
> Extension: 1.8–1.10
> Optional: 1.11
> Hint Model (all sections unified): 1) Orient (command recall) 2) Strategy (how / pattern) 3) Reveal (answer or near-answer).

## 2. Learning Objectives
By the end of Section 1 the player will:
1. Define absolute vs relative path (rooted at `/` vs anchored at current directory).
2. Use `pwd` reflexively for confirmation.
3. Apply `ls` to enumerate and select targets.
4. Use `cd ..` and multi-up patterns (`cd ../../..`) confidently (introduced gradually).
5. Traverse a directive-driven maze requiring up, across, and ignoring stale directions.
6. Jump directly to a deep target using an absolute path.
7. Perform a manual root→home reconstruction using only relative steps.
8. Identify their actual home path without assuming conventional prefix.
9. Interpret file extensions conceptually (e.g., `.txt`) and submit basenames correctly.
10. (Optional) Visualize structure using `tree` to reinforce mental model.

## 3. Concept Tutorial (Displayed Before Level 1.1)
Core concepts:
- Filesystem as a tree with `/` at the top.
- Working directory (CWD) defines relative path origin.
- Commands in play: `pwd`, `ls`, `cd`, `cd ..`, `cd ~`, and optionally `tree` (only final level).
- `$HOME` is authoritative for your home; do not guess its parent path.
- Extensions: a filename may have a suffix after a dot (`inside.txt` has basename `inside` and extension `txt`). We only strip the final extension part; no nested extension complexity here.

Short Prompt:
"Master moving deliberately. Confirm with pwd; inspect with ls; move with cd (including cd ..). Absolute paths start at `/`. Your home is reached via `cd ~` or `cd $HOME`—do not assume its parent directory name."

## 4. Directory Layout (Initial for Section 1)
Workspace subset: `$WORKSPACE/level-1/`

```
level-1/
├── alpha/
│   └── inside.txt            (basename: inside)
├── delta/
│   └── single.dat            (basename: single)
├── gamma/
├── patterns/
│   ├── data.txt
│   ├── dog.md
│   ├── drama.log
│   ├── zebra.txt
│   └── omega/
├── absolute-target/
│   └── PLACEHOLDER.answer    (contains no read task; answer is directory basename later)
└── maze/
    ├── 00/
    │   ├── _GO_TO_DIR_01
    │   ├── 02/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   ├── 03/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   └── 04/
    │       └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    ├── 01/
    │   ├── _GO_TO_DIR_deep
    │   ├── shallow/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   ├── surface/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   └── deep/
    │       ├── _GO_TO_DIR_a
    │       ├── c/
    │       │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │       ├── d/
    │       │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │       └── a/
    │           ├── _GO_TO_DIR_b
    │           ├── x/
    │           │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │           ├── z/
    │           │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │           └── b/
    │               ├── _GO_UP_2_THEN_GO_TO_02
    │               ├── decoy.txt
    │               └── note.md
    ├── 02/
    │   ├── _GO_UP_1_THEN_GO_TO_03
    │   ├── stray.txt
    │   └── readme.md
    ├── 03/
    │   ├── _GO_TO_DIR_x
    │   ├── w/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   ├── z/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   └── x/
    │       ├── _GO_TO_DIR_y
    │       ├── q/
    │       │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │       ├── r/
    │       │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │       └── y/
    │           ├── _GO_UP_1_THEN_GO_TO_04
    │           └── marker.txt
    ├── 04/
    │   ├── _GO_TO_DIR_final
    │   ├── finish/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   ├── end/
    │   │   └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    │   └── done/
    │       └── YOU_ARE_NOT_SUPPOSED_TO_BE_HERE
    └── final/
        └── VICTORY.marker


```

Maze semantics:
- Every directory contains exactly one instruction file starting with underscore that guides the next move.
- Instruction files are either `_GO_TO_DIR_<NAME>` (descend into child dir <NAME>) or `_GO_UP_<N>_THEN_GO_TO_<NAME>` (ascend N levels, then enter named sibling <NAME>).
- Sequence: 00 → 01 → deep → a → b → (up 2 to 02) → 02 → (up 1 to 03) → 03 → x → y → (up 1 to 04) → 04 → final.
- Depth points: 01/deep/a/b and 03/x/y provide ≥4-level depth enforcing multi-level mental modeling.
- Answer = basename of the final directory reached (`final`).

Home (`$HOME`) environment is external; we do not assume location pattern. Hidden directory count appears late (Level 1.10).

## 5. Level Index (Expanded)
| ID    | Title                                   | Focus                                | Answer Type              |
|-------|-----------------------------------------|--------------------------------------|--------------------------|
| 1.1   | Current Location                        | pwd                                   | Directory basename       |
| 1.2   | Listing & Pattern Recognition           | ls pattern picking                    | Directory basename       |
| 1.3   | Enter & Report (Extension Concept)      | cd + basename (strip extension)       | File basename            |
| 1.4   | Return to Base                          | cd .. single ascend                   | Directory basename       |
| 1.5   | Multi-Level Ascent                      | cd ../../.. style                     | Directory basename       |
| 1.6   | Maze Navigation                         | Follow instruction chain + ups        | Directory basename       |
| 1.7   | Absolute Path Jump                      | Construct & use absolute path         | Directory basename       |
| 1.8   | Root to Home Walk (Manual)              | Extension (skip for slow core)         | Ordered list (comma)     |
| 1.9   | Home Confirmation                       | Extension                              | Directory basename       |
| 1.10  | (Optional) Visualizing Structure        | Optional (moved)                       | Ordered list             |

<!-- NOTE: Renamed classification: 1.8–1.9 Extension; 1.10 Optional -->

## 6. Detailed Level Specs

### Level 1.1 – Current Location
Instruction: "Report the basename of your current directory."
Actions: `pwd`, parse last segment.
Answer: `level-1`
Hints:
1. "Use pwd."
2. "Last part after final slash."
3. "Answer: level-1"
Failure: Full path supplied → ask for basename only; case mismatch → emphasize sensitivity.

### Level 1.2 – Listing & Pattern Recognition
Instruction: "List entries. Find directory starting with 'd' and ending with 'a'."
Answer: `delta`
Validation ensures directory exists and matches pattern.
Failure messages differentiate file vs directory.
Hints escalate from using `ls` to pattern explanation to explicit answer.

### Level 1.3 – Enter & Report (Extension Concept)
Instruction: "Enter 'alpha' and submit the name of the single file inside without its extension."
Directory: `alpha/inside.txt`
Extension Teaching:
- Explain: Everything after the last '.' is extension.
- Basename is before that part.
Special Failure Mode:
- If user submits `inside.txt`: respond with "You included the extension. 'inside.txt' has extension 'txt'. Submit only 'inside'."
Answer: `inside`
Hints:
1. "cd alpha"
2. "List contents; strip '.txt'."
3. "Answer: inside"
No `cat` usage required—listing is enough.

Post-Completion Requirement:
Force manual navigation back to `level-1` (transition to Level 1.4). Validation for 1.4 checks current directory before accepting answer.

### Level 1.4 – Return to Base
Start: Inside `alpha/`
Instruction: "Move up to the parent directory and submit its basename."
Action: `cd ..`
Answer: `level-1`
Validation: Must actually be in parent; can compare `pwd` ending.
Failure: If still in `alpha`, instruct to use `cd ..`.
Hints:
1. "Use cd .. to go up one level."
2. "After moving, run pwd to confirm."
3. "Parent is level-1"

### Level 1.5 – Multi-Level Ascent
Setup: Player is placed (or directed) into a deeper chain `delta/single.dat` not entered (we will create subdirs for ascent demo):
Add temporary path for this level:
```
level-1/gamma/deep/a/b/c/
```
Start: `level-1/gamma/deep/a/b/c/`
Instruction: "Go up 3 levels in a single command and submit the basename of the directory you land in."
Action: `cd ../../..`
Target: `deep`
Answer: `deep`
Validation: Confirm `pwd` ends with `/deep`
Hints ladder introduces chaining `..` before giving explicit command.
Failure: Using multiple sequential single ascents acceptable but emphasize efficiency.

### Level 1.6 – Maze Navigation
Start: `level-1/maze/00/`
Goal: Follow `_GO_` instructions; handle multi-up directives; disregard stray files.
Answer: `final` (basename of ending directory containing `VICTORY.marker`)
Validation: Confirm current directory ends with `/final`
Hints:
1. "Only follow filenames starting with _GO_."
2. "For multi-up, count how many ../ segments you need."
3. "Final directory basename is 'final'"
Failure: If answer given without being in `final`: "You are not at the final directory; verify with pwd."

### Level 1.7 – Absolute Path Jump
Instruction: "From anywhere now, use a single absolute cd to reach `level-1/absolute-target/` and submit its basename."
Answer: `absolute-target`
Note: No file reading. Directory basename is the answer.
Validation: Must be located at that path.
Hints:
1. "Absolute path starts with /"
2. "Include /tmp/.../level-1/absolute-target in one cd"
3. "Answer: absolute-target"

### Level 1.8 – Root to Home Walk (Manual)
Instruction: "Walk to root '/', then step by step to your home ($HOME) using only relative moves. Submit comma-separated list of each basename you entered AFTER leaving root (excluding the empty root)."
Process:
1. `cd /`
2. Identify first segment of `$HOME` path by comparing `$HOME` and `pwd`.
3. Enter segments one at a time: `cd <segment>`
Answer: Ordered list of path segments constructing home (e.g. if `$HOME=/net/students/username` → `net,students,username`)
Validation:
- Must match actual `$HOME` expansion.
Hints:
1. "Find $HOME with echo $HOME."
2. "Break it into components after leading slash."
3. "Comma list of each component in order."

### Level 1.9 – Home Confirmation
Instruction: "Navigate (or remain) at $HOME and submit the basename of your home directory."
Answer: Last component of `$HOME` path (`username`).
Validation: Verify `pwd == $HOME`.
Failure: Not at home → instruct to `cd $HOME`.
Hints escalate from using `$HOME` to final component reveal.

### Level 1.10 – (Optional) Visualizing Structure
Instruction: "In `level-1/` list (or visualize using tree) top-level directory names alphabetically, submit comma-separated list."
Directories: `absolute-target, alpha, delta, gamma, maze, patterns`
Answer: `absolute-target,alpha,delta,gamma,maze,patterns`
Hints escalate same pattern; final gives sequence.
Optional metadata: `optional=true`.

## 7. General Validation Rules
- Trim whitespace.
- Basename answers: no `/`.
- File basename: remove single trailing extension (portion after last dot).
- Ordered list: split by commas; ignore surrounding spaces; order must match expected.
- Integer: numeric only.
- Presence checks rely on current `pwd` for location-sensitive levels (1.4, 1.5, 1.6, 1.7, 1.9, 1.10, 1.11).

## 8. Hint System Mechanics
Three hints per level:
1. Recall / orientation.
2. Strategy / partial path or method.
3. Explicit near-answer or full answer (allowed in early formative tasks).
Track `hints_used` per level.

## 9. Telemetry / State Logging
Per completion:
```
"1.x": {
  "time_sec": <int>,
  "hints": <int>,
  "attempts": <int>
}
```
Optional level 1.11 recorded only if attempted; otherwise can store `"1.11": {"optional_skipped": true}` or omit.

Section completion condition: Levels 1.1–1.7 correct (1.8–1.10 Extension, 1.11 Optional).

## 10. Failure Message Templates
- Extension included: "You included the extension. Submit only the part before the final dot."
- Multi-word where single expected: "Expected a single word; multiple tokens received."
- Not at required directory: "You are currently at a different path—use pwd and adjust."
- Directory vs file mismatch: "Type mismatch: expected a directory."
- Case mismatch: "Case-sensitive mismatch—verify spelling."
- Integer parse: "Expected an integer count."
- Order mismatch in lists: "List must be alphabetically ordered."

## 11. Edge Cases & Robustness
- Home path unusual (network mount): always derive from `$HOME`; never pattern-match `/home/`.
- If player creates extra hidden directories before Level 1.10, count naturally includes them (teaches environment integrity awareness).
- Maze resilience: ignore unexpected files; only `_GO_` prefixed names instruct.
- Missing `tree`: fallback to `ls` sorted output; instruct: "tree not available; simulate by listing and sorting."

## 12. Implementation Checklist
- Generate directory structure before starting Section 1.
- Maze instruction files named predictably; ensure multi-up instructions exist.
- Provide helper to compare current path for ascent validation.
- Implement extension strip: `basename.split('.')[:-1].join('.')` (simple; assume only one dot for teaching clarity).
- Home walk segmentation: `segments = HOME_PATH.strip('/').split('/')`
- (Hidden directory logic deferred to Section 3; not implemented here.)

## 13. Advancement Criteria
Upon finishing Level 1.9:
- `current_level = "2.1"`
- If Level 1.11 later attempted, do not modify advancement; treat as post-completion enrichment.

## 14. Sample Instruction Screen (Level 1.7)
```
══════════════════════════════════════════════════════
LEVEL 1.7: Absolute Path Jump

Goal:
Reach the directory 'absolute-target' inside level-1 using ONE
absolute cd command starting from wherever you are.

Remember:
Absolute paths start with /. You can find your current full path
with: pwd

Task:
1. Determine full path to .../level-1/absolute-target
2. cd /tmp/.../level-1/absolute-target  (adjust ... to match your workspace)
3. Submit the basename of that directory.

Submit with: shellgame submit -f absolute-target
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- Habit loop: move → pwd → interpret → next move.
- Multi-up navigation accelerates path correction.
- Maze compels careful reading of names, not guessing.
- Absolute paths free students from local uncertainty.
- Root→home walk consolidates path parsing and relative construction.
- Hidden directories deferred entirely to Section 3.
- Visualization last to transform abstract learning into structural insight.

## 16. Future Cross-References
- Section 2 deepens relative chains (`../..` plus siblings).
- Section 3 formalizes hidden files & `ls -a` beyond counting.
- Later sections add creation, manipulation, and permissions—kept out here for focus.

## 17. Summary (Instructor View)
Section 1 now delivers sustained navigation drills: baseline identification, pattern selection, controlled descent/ascent, maze interpretation, absolute path synthesis, and environment awareness without assuming conventional home layout. Students exit with high-confidence directional skills ready for relative chain mastery.

Addendum (Pacing): Encourage students to advance after 1.7 if time-constrained; revisit extension/optional tasks post-progress without blocking flow.