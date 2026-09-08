# Section 3 Specification – "Hidden vs Visible"

## 1. Purpose & Scope
Section 3 introduces hidden entries in the filesystem (names beginning with a dot `.`) and contrasts them with visible (non-dot-prefixed) entries. Building on navigation and simple file interaction from Sections 1–2, this section focuses on:
- Recognizing hidden files and directories with `ls -a`
- Counting hidden directories accurately (excluding `.` and `..`)
- Reading content from hidden files safely
- Navigating into hidden directories
- Interpreting multi-part filenames (extension / suffix awareness reinforcing earlier concepts)

Deliberate exclusions:
- No creation or deletion commands here (`mkdir`, `rm`, etc.) — deferred to Section 4
- No permission changes
- No pattern globbing (`*`, `?`) yet
- No size or metadata inspection (`ls -l`)

Allowed commands: `pwd`, `ls`, `ls -a`, `cd`, `cat` (introduced in Section 2; reused here)
Estimated Time: 5–8 minutes for core levels (3.1–3.3) + optional Level 3.4 (~2 minutes)
> Time Calibration: Target 5 minutes average; slow path 4 minutes (Levels 3.1–3.2 only). Level 3.3 now Extension for very slow students (can defer), 3.4 remains Optional.

## 2. Learning Objectives
By the end of Section 3 the player will:
1. Explain what makes a file or directory “hidden” (leading dot).
2. Use `ls -a` to reveal hidden entries reliably.
3. Distinguish hidden files from hidden directories via inspection (`ls -a` then `cd` attempt).
4. Count only hidden directories excluding the pseudo entries `.` and `..`.
5. Read the first word of a hidden file’s content.
6. Navigate inside a hidden directory and count its real files (excluding `.` and `..` if listed with `ls -a`).
7. (Optional) Extract the basename or final suffix segment from a multi-dot hidden filename.

## 3. Concept Tutorial (Displayed Before Level 3.1)
Highlights:
- Hidden entries start with `.` (examples: `.config`, `.cache`, `.env`, `.notes.txt`)
- `ls` does NOT list hidden entries; use: `ls -a` to show all (including `.` and `..`)
- `.` = current directory; `..` = parent directory — not real files for counting purposes
- Hidden status is a naming convention, not a permission change
- Reading hidden files uses the same `cat` command
- Basename vs extension refresher: In `.backup.archive.log` the basename concept we use is everything before the first dot for hidden naming exercises OR we may request the last segment after the final dot (specify per level)

Mini Example:
```
(hidden-play/)
├── visible.txt
├── .secret/
│   └── clue.md          (content: "vault key")
├── .config
├── .archive.old         (content: "snapshot legacy")
└── .notes.txt           (content: "daily summary")
```

## 4. Directory Layout (Initial for Section 3)
Base path: `$WORKSPACE/level-3/`

Proposed structure:
```
level-3/
├── hub/                       (starting directory for Level 3.1)
│   ├── .alpha
│   ├── .beta/
│   │   └── readme.txt         (content: "hidden journey")
│   ├── .gamma/
│   │   └── info.md            (content: "stealth mode")
│   ├── visible/
│   │   └── marker.txt         (content: "standard entry")
│   ├── .delta.log             (content: "log initialized")
│   └── .epsilon.archive.old   (content: "historic bundle")
├── hidden-read/               (Level 3.2 focus)
│   └── .schedule.txt          (content: "Tuesday events")
├── deep/
│   └── .vault/
│       ├── door.txt           (content: "open sesame")
│       ├── key.md             (content: "silver token")
│       └── .inner-secret      (empty file)
└── suffix-lab/
    └── .multi.segment.name.final.txt  (content: "multi hidden example")
```

Notes:
- `.alpha` is a hidden file (empty or minimal content not required).
- `.beta` and `.gamma` are hidden directories.
- Counting tasks will differentiate directories vs files.
- Optional suffix extraction uses multi-dot hidden filename in `suffix-lab/`.

## 5. Level Index
| ID   | Title                               | Focus                                   | Answer Type            |
|------|-------------------------------------|-----------------------------------------|------------------------|
| 3.1  | Hidden Directory Count              | Use ls -a, exclude . and ..             | Integer count          |
| 3.2  | Reading a Hidden File               | cat on hidden file                      | Single word (content)  |
| 3.3  | Inside a Hidden Directory           | Extension (can defer)          | Integer count          |
| 3.4  | (Optional) Hidden Filename Suffix   | Optional                       | Single word (suffix)   |

## 6. Detailed Level Specifications

### Level 3.1 – Hidden Directory Count
Start Location: `$WORKSPACE/level-3/hub/`
Task: "List all entries including hidden. Count hidden directories (names starting with '.') excluding '.' and '..'. Submit the count."
Hidden Directories Present: `.beta`, `.gamma`
Hidden Non-Directory Files: `.alpha`, `.delta.log`, `.epsilon.archive.old`
Answer: `2`
Validation:
- Must be integer.
- Confirm candidate names actually directories (stat check in implementation).
Failure Modes:
- Counting hidden files → "You included hidden files; count only directories."
- Non-integer input → "Expected an integer."
Hints:
1. "Use ls -a to reveal hidden entries."
2. "Filter for names starting with '.' that are directories."
3. "There are 2 hidden directories."

### Level 3.2 – Reading a Hidden File
Start Location: `$WORKSPACE/level-3/hidden-read/`
Target File: `.schedule.txt` with content: `Tuesday events`
Task: "Read the hidden file `.schedule.txt` and submit its first word."
Answer: `Tuesday`
Rules:
- First word only.
- Case-insensitive acceptance (consistent with prior content word rules).
Failure:
- Submitting both words → "Expected a single word."
- Submitting filename → "Submit the first word from the file content."
Hints:
1. "Use ls -a to see the file."
2. "Use cat .schedule.txt; take the first word only."
3. "Answer: Tuesday"

### Level 3.3 – Inside a Hidden Directory
Start Location: `$WORKSPACE/level-3/deep/`
Goal: Enter hidden directory `.vault/` then count regular files inside (exclude `.` and `..`; none of its entries are directories except possibly `.inner-secret` which is a hidden file).
Contents inside `.vault/`: `door.txt`, `key.md`, `.inner-secret`
Task: "Navigate into .vault and count the number of files present (include hidden file `.inner-secret` in the count; exclude '.' and '..')."
Answer: `3`
Validation:
- Verify current directory ends with `.vault`
- Count all entries except `.` and `..` regardless of leading dot
Failure:
- Not inside `.vault` → "You are not in .vault—check with pwd."
- Wrong count (e.g., 2) → "Did you include the hidden file starting with a dot?"
Hints:
1. "cd .vault after listing with ls -a."
2. "List all entries; ignore . and ..."
3. "There are 3 files."

### Level 3.4 – (Optional) Hidden Filename Suffix
Start Location: `$WORKSPACE/level-3/suffix-lab/`
Target Filename: `.multi.segment.name.final.txt`
Task: "Submit the final suffix (the word before the last dot extension) from `.multi.segment.name.final.txt`. Ignore the leading dot and the final extension."
Explanation:
- Split on dots: ['', 'multi', 'segment', 'name', 'final', 'txt']
- Leading empty segment (due to starting dot) ignored.
- Last segment 'txt' is the extension.
- Desired suffix: `final`
Answer: `final`
Failure Modes:
- `final.txt` submitted → "Remove the extension."
- `multi` / other segment → "That is not the final suffix before extension."
Hints:
1. "Split the name by dots; ignore the leading empty part."
2. "Drop the last segment (the extension)."
3. "Answer: final"
Optional metadata: `optional=true`

## 7. General Validation Rules (Section 3)
- Trim whitespace.
- Integer answers: strict digit parse.
- Content word answers: single word; case-insensitive compare.
- Suffix extraction: case-sensitive (filename semantics).
- Location-dependent tasks (3.3, 3.4): verify `pwd` ends with expected directory.
- Hidden directory count excludes `.` and `..`.
- For multi-dot filename parsing, define: segments = name.split('.'); discard empty first; extension = last; suffix = second-to-last.

## 8. Hint System
Three hints per level:
1. Command recall / orientation (`ls -a`, `cd .vault`)
2. Strategy refinement (filter, splitting, counting logic)
3. Near-answer or full answer for reinforcement (especially early hidden concepts)
Record `hints_used` and `attempts`.

## 9. Telemetry / State Logging
Per completion:
```
"3.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement condition: Complete Levels 3.1–3.2 (3.3 Extension, 3.4 Optional).

## 10. Failure Message Templates
- Hidden count includes files: "Count only hidden directories; you included hidden files."
- Not inside target directory: "You are at a different path—use pwd to confirm."
- Multiple words: "Expected a single word."
- Wrong suffix: "Not the final segment before the extension—inspect the filename again."
- Extension included: "Remove the extension; submit only the suffix."
- Non-integer: "Expected an integer count."

## 11. Edge Cases & Robustness
- Additional user-created hidden entries: Accept natural variation but instruct reset if counts differ from canonical. (Implementation may snapshot expected directory for Level 3.1.)
- Empty hidden file: If user tries to read a hidden file with no content (not in current design), answer must be clarified as "File appears empty—reset level."
- Unusual home path irrelevant here (using workspace only).
- If user renames files: Validation fails; instruct reset.

## 12. Implementation Checklist
- Create directory tree and files before level start.
- Provide helper:
  - `is_hidden(name)` → name.startswith('.')
  - `hidden_dir_count(path)` filtering directories excluding `.` and `..`
  - `extract_suffix(filename)` logic as specified.
- Validation ensures current path for location-sensitive tasks.
- Reset routine recreates minimal structure for section.
- Hints stored as per-level arrays of length 3.

## 13. Advancement Criteria
After successful completion of Level 3.2:
- Set `current_level = "4.1"`
- Preserve ability to attempt 3.4 later without regression.

## 14. Sample Instruction Screen (Level 3.1)
```
══════════════════════════════════════════════════════
LEVEL 3.1: Hidden Directory Count

Hidden entries start with a dot. Use ls -a to see them.
Count ONLY hidden directories here (exclude . and ..).
Submit the count as a single integer.

Commands: ls -a, pwd

Submit with: shellgame submit <number>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- Visual differentiation: `ls` vs `ls -a` encourages conceptual shift.
- Counting task anchors definition of hidden directories vs files.
- Reading hidden file extends prior skill—hidden does not mean special access.
- Directory traversal into hidden directory normalizes hidden navigation.
- Optional suffix extraction consolidates multi-dot parsing and extension awareness.

## 16. Future Cross-References
- Section 4: creation & cleanup (mkdir, rm) will include making hidden directories.
- Later: pattern matching may target hidden names explicitly.
- Permissions section (later) clarifies hidden ≠ protected.

## 17. Summary (Instructor View)
Section 3 isolates hidden visibility mechanics, ensuring learners confidently list, classify, and interact with hidden entries before adding creation/deletion complexity. Focus remains narrow to solidify conceptual clarity and avoid cognitive overload prior to structural modifications in Section 4.

Pacing Note: Permit advancement after 3.2 when compressing schedule; revisit directory traversal (3.3) later.