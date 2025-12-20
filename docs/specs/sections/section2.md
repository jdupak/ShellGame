# Section 2 Specification – "Moving With Purpose: Files & Navigation"

## 1. Purpose & Scope
Section 2 deepens navigation fluency while introducing minimal file interaction:
- Reinforces relative path reasoning (including multi-up sequences and sibling traversal).
- Integrates reading existing file contents using `cat`.
- Introduces file creation using `touch`.
- Strengthens mental modeling of directory layout without visual aids.
- Avoids hidden files (deferred to Section 3), permissions, deletion, or editing.

Allowed commands in this section: `pwd`, `ls`, `cd`, `cat`, `touch`
(No `ls -l`, no `echo >`, no `tree`, no wildcards, no `rm` yet.)

Estimated Time: 8–12 minutes (core Levels 2.1–2.4) + optional Level 2.5 (~2 minutes).
> Time Calibration: Target 7 minutes average. Core = 2.1–2.4 (navigation + reading). 2.5 becomes Extension (clue chain). 2.6 remains Optional (creation). Slow path may skip 2.5–2.6.

## 2. Learning Objectives
By the end of Section 2 the player can:
1. Traverse multi-level and sibling paths confidently using only relative movements.
2. Use `cd -` to toggle between the current and previous working directory.
3. Use `cat` to extract a single word (first word) from a file without altering the file.
4. Create an empty file with `touch` in a target directory after deliberate navigation.
5. Follow a chain of directory clues encoded inside files to reach a final destination.
6. Avoid over-reliance on starting position by reorienting with `pwd` and planning movements.
7. Recognize that an empty file (after `touch`) has no content when displayed with `cat`.

## 3. Concept Tutorial (Shown Before Level 2.1)
Concept Focus:
- Relative navigation review: `.` (current), `..` (parent), sibling movement by name.
- Toggling directories with `cd -` to quickly jump back.
- Sequential navigation planning: break longer moves into deliberate steps.
- Using `cat <file>` to display file contents; reading only, not editing.
- Using `touch <file>` to create an empty file if it does not exist (or update its timestamp if it exists).
- Empty file behavior: `cat` prints nothing (just returns).

Key Principle:
“Navigate with intent. Confirm location with pwd. Read to discover; create only when instructed.”

Mini ASCII Example (dynamic in implementation):
```
level-2/
├── reading/
│   ├── poem.txt            (content: "autumn leaves")
│   └── clue.txt            (content: "travel")
├── travel/
│   └── next.txt            (content: "chain")
├── chain/
│   └── final.dat           (content: "arrival complete")
├── create-zone/
│   └── placeholder.md
└── multi/
    └── a/b/c/target.txt    (content: "summit reached")
```

## 4. Directory Layout (Initial for Section 2)
Base path: `$WORKSPACE/level-2/`

Proposed structure (deterministic to simplify validation):
```
level-2/
├── start/                          (starting directory for some levels)
│   ├── alpha/
│   │   └── note.txt                (content: "first step")
│   ├── beta/
│   │   └── word.md                 (content: "momentum gained")
│   └── gamma/
│       └── hold.txt                (content: "pause briefly")
├── multi/
│   └── a/
│       └── b/
│           └── c/
│               └── target.txt      (content: "summit reached")
├── reading/
│   ├── poem.txt                    (content: "autumn leaves")
│   └── clue.txt                    (content: "travel")
├── travel/
│   └── next.txt                    (content: "chain")
├── chain/
│   └── final.dat                   (content: "arrival complete")
├── create-zone/
│   └── placeholder.md
├── toggle-zone/
│   ├── one/
│   │   └── file.a
│   └── two/
│       └── file.b
└── sandbox/                        (empty; may be used for optional task)
```

Clue Chain Intended Flow (Level 2.4):
1. `reading/clue.txt` → contains: `travel`
2. `travel/next.txt` → contains: `chain`
3. `chain/final.dat` → final content first word: `arrival`

## 5. Level Index
| ID   | Title                            | Focus                                   | Answer Type          |
|------|----------------------------------|-----------------------------------------|----------------------|
| 2.1  | Sibling Navigation Drill         | Relative movement across siblings       | File basename (no ext) |
| 2.2  | Previous Directory Toggle        | Using `cd -` to jump back              | Directory basename   |
| 2.3  | Deep Relative Ascent & Descent   | Multi-hop navigation (../ chains)       | Single word          |
| 2.4  | Reading a File’s First Word      | Using cat to extract first word         | Single word          |
| 2.5  | Chained Clue Traversal           | Extension (do after 2.4 if time)        | Single word          |
| 2.6  | (Optional) Create & Confirm      | Optional                                 | File basename        |

## 6. Detailed Level Specifications

### Level 2.1 – Sibling Navigation Drill
Start: `$WORKSPACE/level-2/start/alpha/`
Task: Move to siblings `beta` then `gamma` (via parent directory) and finally submit the basename of the file inside `gamma` without its extension.
Target File: `gamma/hold.txt` (basename: `hold`)
Answer: `hold`
Required Steps:
1. `pwd` (confirm in alpha)
2. `cd ..` (to start)
3. `cd beta`, inspect with `ls`, optional `cat word.md`
4. `cd ..`, then `cd gamma`
5. `ls` to find `hold.txt`
6. Submit `hold`
Validation:
- Must not include `.txt`
Failure Modes:
- Submit `hold.txt` → message clarifying extension removal.
- Submit while still in `beta` → location warning.
Hints:
1. "Move to parent with cd .. to reach siblings."
2. "Visit gamma and list files; drop the extension."
3. "Answer is hold"

### Level 2.2 – Previous Directory Toggle
Start: `$WORKSPACE/level-2/toggle-zone/one/`
Task: "You are in the `one` directory. Navigate to the `two` directory. Then, use `cd -` to jump back to your previous location (`one`). Submit the basename of the directory you land in."
Steps:
1. `pwd` (confirms you are in `.../one`)
2. `cd ../two` (navigate to `two`)
3. `pwd` (confirms you are in `.../two`)
4. `cd -` (toggles back to `one`)
5. `pwd` (confirms you are back in `.../one`)
Answer: `one`
Validation:
- The final directory must be `one`.
- The answer must be the directory basename.
Failure Modes:
- Submitting `two` -> "You submitted your current directory, not the one you toggled back to."
Hints:
1. "After moving to `../two`, use `cd -` to return."
2. "The `cd -` command takes you to the directory you were in before your last `cd`."
3. "The answer is `one`."

### Level 2.3 – Deep Relative Ascent & Descent
Setup: Player is moved to `$WORKSPACE/level-2/multi/a/b/c/`
Task: Ascend two levels in one command, then descend into `b/c/` again using separate steps, finally read `target.txt` first word.
Steps Example:
- From `.../a/b/c/` run `cd ../..` → now in `a`
- Then `cd b`, `cd c`
- `cat target.txt` → prints: `summit reached`
Answer: `summit` (first word only)
Validation:
- Must be exactly first token.
- Require that final `pwd` ends with `/c` before accept (ensures descent done).
Failure:
- User submits both words → instruct single word.
- Wrong ascent path → encourage checking with `pwd`.
Hints:
1. "Use cd ../.. to go up two levels."
2. "Re-enter b then c one directory at a time."
3. "First word is summit"

### Level 2.4 – Reading a File’s First Word
Start: `$WORKSPACE/level-2/reading/`
Task: Read `poem.txt` and submit its first word.
Content: `autumn leaves`
Answer: `autumn`
Rules:
- Ignore second word.
- No modification of file.
Validation:
- Case-insensitive for content word.
- Reject multiple words.
Hints:
1. "List files; identify poem.txt."
2. "Use cat poem.txt and take only the first word."
3. "Submit: autumn"
Failure Examples:
- Input: `autumn leaves` → "Expected a single word."
- Input: `poem` → "That is not the file content; read the file."

### Level 2.5 – Chained Clue Traversal
Start: `$WORKSPACE/level-2/reading/`
Instruction File Sequence:
- `reading/clue.txt` → contains `travel`
- `travel/next.txt` → contains `chain`
- `chain/final.dat` → contains `arrival complete`
Task:
1. Read `clue.txt` → navigate into `travel/`
2. Read `next.txt` → navigate into `chain/`
3. Read `final.dat` → submit first word
Answer: `arrival`
Validation:
- Ensure final directory is `chain` when answering.
- Must be first word only.
Hints:
1. "Each file’s first word is the next directory name (except final)."
2. "clue.txt → travel; next.txt → chain."
3. "Final first word: arrival"
Failure:
- If user guesses chain prematurely → "Navigate fully to final.dat and read its content."
Edge Case:
- If user jumps directly without reading: allow navigation but still requires correct word; hints encourage reading for authenticity.

### Level 2.6 – (Optional) Create & Confirm
Start: `$WORKSPACE/level-2/create-zone/`
Task: Create an empty file named `checkpoint.flag` using `touch` and submit its basename without extension.
Answer: `checkpoint`
Instructions:
1. Confirm current directory with `pwd`
2. Run `touch checkpoint.flag`
3. Use `ls` to verify presence
4. Submit `checkpoint`
Validation:
- Must exist as a file.
- Extension stripped.
Failure:
- File missing → "File not found—did you create it with touch?"
- Submitted with extension → extension guidance.
Hints:
1. "Use touch to create the file."
2. "List files; remove extension when submitting."
3. "Answer: checkpoint"
Optional metadata: `optional=true`

## 7. General Validation Rules (Section 2)
- Trim whitespace.
- File basename answers: strip single trailing extension (`name.ext` → `name`).
- Content word answers: single token only.
- For levels requiring navigation state (2.1, 2.2, 2.3, 2.5, 2.6), verify `pwd` matches expected target path prefix.
- Case sensitivity: basenames are case-sensitive; content words (from files) case-insensitive as per global spec philosophy.
- Creation check (2.6): confirm file existence; treat size as irrelevant (no size validation yet).

## 8. Hint Design & Pedagogy
Three-tier hint ladder:
1. Command recall / orientation (e.g., "Use cd ..", "List with ls").
2. Strategy articulation (multi-up pattern, identify next directory name).
3. Near-answer or explicit answer for early formative reinforcement.
Record `hints_used` and `attempts` per level for telemetry.

## 9. Telemetry / State Logging
Per level completion:
```
"2.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement condition: Levels 2.1–2.4 completed. Level 2.5 Extension; 2.6 Optional.