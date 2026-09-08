# Section 4 Specification – "Creating & Cleaning Your World"

## 1. Purpose & Scope
Section 4 introduces controlled creation and cleanup of filesystem elements inside the isolated game workspace. Students move from passive navigation (Sections 1–3) to intentional environment shaping:
- Creating files and directories (`touch`, `mkdir`, `mkdir -p`)
- Understanding when a directory is considered “empty”
- Removing files (`rm`) and empty directories (`rmdir`)
- Handling non-empty directory removal attempts (failure feedback)
- Building small multi-level structures consciously
- Practicing safe removal patterns without risking system data

No editing tools (nano/vim), no permissions (`chmod`), no wildcard-heavy batch operations (those come later in Section 9). Wildcards may be referenced ONCE as an optional enhancement but not required for progression.

Estimated Time: 12–15 minutes (core Levels 4.1–4.6) + optional Level 4.7 (~3 minutes).

> Time Calibration: Target 9 minutes average; slow path 7 minutes (Levels 4.1–4.5). Level 4.6 becomes Extension (scaffold building), 4.7 Optional. Emphasize separation of create vs clean before complex scaffold.

## 2. Learning Objectives
By completion of Section 4 the player can:
1. Create an empty file and verify it exists.
2. Create a single directory and confirm its basename.
3. Use `mkdir -p` to create nested directory chains in one command.
4. Distinguish between removing a file (`rm file`) and removing an empty directory (`rmdir dir`).
5. Discover why `rmdir` fails on non-empty directories and identify the remaining item.
6. Manually remove a subset of files based on an extension.
7. Construct a mini “project scaffold” and count directories created.
8. (Optional) Use a safe pattern removal (introducing cautious wildcard thinking).

## 3. Concept Tutorial (Displayed Before Level 4.1)
Key points:
- `touch name` creates an empty file.
- `mkdir dir` creates a single directory; fails if it already exists.
- `mkdir -p a/b/c` builds nested structure; intermediate directories created automatically.
- Removing:
  - `rm file` deletes a file.
  - `rmdir dir` only succeeds if `dir` is empty.
- Non-empty removal: `rmdir` error reinforces inspection (use `ls`) before deletion.
- Workspace safety: operations limited to `/tmp/...` game sandbox.
- Terminology:
  - “Empty directory” = contains no entries besides implicit `.` and `..`.
  - “Scaffold” = planned multi-directory hierarchy.

Short prompt:
“Create carefully, confirm with ls/pwd, then clean what you created. Understand empty vs non-empty and intentional structure building.”

## 4. Directory Layout (Initial for Section 4)
Base: `$WORKSPACE/level-4/`

Initial structure before any tasks:
```
level-4/
├── staging/
├── cleanup/
│   ├── keep.txt              (content: "persist")
│   ├── old.tmp               (empty)
│   ├── stray.tmp             (empty)
│   └── data.txt              (content: "value stream")
├── scaffold/
└── obstacle/
    └── nested/
        └── token.md          (content: "depth")
```

Notes:
- `staging/` starts empty for file/directory creation exercises.
- `cleanup/` contains two `.tmp` files targeted for manual removal task, plus survivors.
- `obstacle/nested/` holds a file to enforce non-empty directory removal failure.
- `scaffold/` starts empty for multi-level construction.

## 5. Level Index
| ID   | Title                                | Focus                                   | Answer Type                |
|------|--------------------------------------|-----------------------------------------|----------------------------|
| 4.1  | Empty File Creation                  | touch + existence check                 | File basename              |
| 4.2  | Single Directory Creation            | mkdir + basename confirmation           | Directory basename         |
| 4.3  | Nested Creation With -p              | mkdir -p chain                          | Integer (count created)    |
| 4.4  | Manual File Cleanup                  | rm selected extensions                  | Integer (remaining .txt)   |
| 4.5  | Non-Empty Directory Removal Attempt  | rmdir failure reasoning                 | File basename (survivor)   |
| 4.6  | Project Scaffold Build               | Extension                      | Integer (directories made) |
| 4.7  | (Optional) Pattern Preview Cleanup   | Optional                       | Ordered list               |

## 6. Detailed Level Specifications

### Level 4.1 – Empty File Creation
Start: `$WORKSPACE/level-4/staging/`
Task: Create a file `checkpoint.log` using `touch`. Submit its basename WITHOUT extension.
Answer: `checkpoint`
Validation:
- File exists at `staging/checkpoint.log`.
- Extension removed.
Failures:
- Submits `checkpoint.log` → extension reminder.
- File missing → instruct to run `touch checkpoint.log`.
Hints:
1. "Use touch checkpoint.log"
2. "List with ls to confirm creation."
3. "Submit basename: checkpoint"

### Level 4.2 – Single Directory Creation
Start: `$WORKSPACE/level-4/staging/`
Task: Create directory `reports`. Submit its basename.
Answer: `reports`
Validation:
- Directory exists and is a directory.
Failures:
- Submits full path → ask for basename only.
Hints:
1. "Use the `mkdir` command to create a directory."
2. "After creating it, submit its name: `reports`."

### Level 4.3 – Nested Creation With -p
Start: `$WORKSPACE/level-4/staging/`
Task: Create nested path `archive/2025/nov/exports/` in one command using `mkdir -p`. Count how many NEW directories were created.
Answer: `4`
Validation:
- Each directory exists.
- Count matches new creations (assume none existed initially).
Failure:
- Uses multiple mkdir calls → acceptable; still count new directories.
- Wrong count → instruct to run `ls -R archive`.
Hints:
1. "Use mkdir -p archive/2025/nov/exports"
2. "List intermediate directories to count them."
3. "Answer: 4"

### Level 4.4 – Manual File Cleanup
Start: `$WORKSPACE/level-4/cleanup/`
Task: Remove all `.tmp` files manually. After removal, count remaining `.txt` files.
Initial files: `keep.txt`, `old.tmp`, `stray.tmp`, `data.txt`
Action: `rm old.tmp stray.tmp`
Answer: `2` (remaining `.txt` files: keep.txt, data.txt)
Validation:
- `.tmp` files absent.
- Count of `.txt` files is correct.
Failure:
- Leaves a `.tmp` file → "You did not remove all .tmp files."
- Removes a `.txt` file → "You removed a required survivor."
Hints:
1. "List .tmp files before removing."
2. "Remove each with `rm name.tmp`."
3. "Two .txt files should remain."

### Level 4.5 – Non-Empty Directory Removal Attempt
Start: `$WORKSPACE/level-4/obstacle/`
Task: Attempt to remove `nested/` with `rmdir nested` (it will fail). Identify the basename of the file inside that made it non-empty.
File inside: `token.md`
Answer: `token`
Validation:
- Confirm `nested/` still exists.
- Confirm `token.md` exists.
Failure:
- Removes directory via forceful method (not taught) → Instruct reset.
- Submits `token.md` → extension reminder.
Hints:
1. "Try `rmdir nested` and observe the error."
2. "List the contents of `nested` to see what's inside."
3. "The file's basename is `token`."

### Level 4.6 – Project Scaffold Build
Start: `$WORKSPACE/level-4/scaffold/`
Task: Build a mini structure: `app/`, with `src/`, `tests/`, and `docs/` inside it. Submit the total number of DIRECTORIES created.
Answer: `4`
Validation:
- All directories exist (`app`, `app/src`, `app/tests`, `app/docs`).
- Count includes the root `app`.
Failure:
- Missing one subdirectory → instruct to list contents.
Hints:
1. "Create the root `app` directory first, then `cd` into it."
2. "Use `mkdir` to create `src`, `tests`, and `docs`."
3. "The total number of directories created is 4."

### Level 4.7 – (Optional) Pattern Preview Cleanup
Start: `$WORKSPACE/level-4/scaffold/`
Setup (game engine adds before level begins):
```
logs/
├── a.log
├── b.log
├── c.tmp
└── readme.md
```
Task: Remove only `.log` files. Submit an ordered comma-separated list of remaining basenames (alphabetic).
Remaining files after correct removal: `c.tmp`, `readme.md`
Answer: `c,readme`
Validation:
- Log files absent.
- Survivors present in correct order (alphabetic by basename).
Failure:
- Removes `c.tmp` or `readme.md` → "Removed required survivor."
- Leaves a `.log` → "A .log file still remains."
Hints:
1. "Identify .log files with `ls`."
2. "Remove them one by one with `rm`."
3. "The remaining basenames, sorted alphabetically, are `c,readme`."

## 7. General Validation Rules
- Trim whitespace.
- File basename answers: remove final extension segment after last dot.
- Directory basename answers: no slashes.
- Integer answers: strict digit parsing.
- Ordered list answers: split by commas, trim spaces, enforce expected sequence.
- Location-sensitive tasks: verify current `pwd` starts with `$WORKSPACE/level-4/...`.
- For creation counts: engine records pre-state to ensure accurate "new" count.

## 8. Hint Strategy
Three hints per level:
1. Command reminder / orientation.
2. Method refinement (-p usage, manual enumeration).
3. Near-answer or explicit value for reinforcement.
Track `attempts` and `hints_used` for telemetry.

## 9. Telemetry / State Logging
Per completion:
```
"4.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 4.1–4.5 required. 4.6 Extension. 4.7 Optional.

## 10. Failure Message Templates
- Extension included: "Submit basename only (remove extension)."
- Missing file after touch: "File not found—did you create it with `touch`?"
- Directory missing: "Directory not found—verify with `ls`."
- Non-empty removal confusion: "`rmdir` only works on empty directories—inspect contents."
- Incorrect count: "Count does not match existing items—list them again."
- Survivor removed: "You removed a required file—rebuild or reset."

## 11. Edge Cases & Robustness
- If a student force-deletes a non-empty directory (e.g., using `rm -r` prematurely), instruct to reset and clarify that recursive removal is out of scope.
- Accidental re-running of `mkdir -p` does not inflate count (engine compares existing vs newly created).
- Pattern preview (optional) must not require glob knowledge; hints mention manual removal first.

## 12. Implementation Checklist
- Provide helper functions for file/dir existence, listing, and counting.
- Validate current directory when location is critical.
- Flag optional level in metadata.
- Implement a reset mechanism to reconstruct the full Section 4 tree.

## 13. Advancement Criteria
After Level 4.5 success:
- `current_level = "5.1"`
Optional Level 4.7 accessible post-advancement.

## 14. Sample Instruction Screen (Level 4.3)
```
══════════════════════════════════════════════════════
LEVEL 4.3: Nested Creation With -p

Goal:
Create a nested directory path archive/2025/nov/exports
in ONE command.

Remember:
`mkdir -p` will create all missing parent directories.

Task:
1. Run `mkdir -p archive/2025/nov/exports`
2. Verify structure with `ls -R archive`
3. Count how many directories you just created
4. Submit that number

Submit with: shellgame submit <number>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- Emphasizes intentional environment shaping over random creation.
- Introduces “failure by design” (non-empty directory removal) as a learning pattern.
- Encourages enumeration and structural thinking (scaffold building).
- Prepares learner for more complex manipulations in later sections.

## 16. Future Cross-References
- Section 5: File Inspection (`ls -l`, `file`).
- Section 6: Copying & Moving (`cp`, `mv`).
- Section 9: Wildcards—will generalize pattern-based cleanup beyond manual enumeration.
- Recursive removal (`rm -r`) is intentionally delayed for focus and safety.

## 17. Summary (Instructor View)
Section 4 transitions learners from consumers of a prepared filesystem to active shapers. The exercises calibrate risk-free creation and deliberate cleanup, strengthening internalization of directory semantics ("empty" vs "non-empty") while reinforcing navigation fluency. By the end, students possess practical confidence to build and tidy structures—a critical foundation for subsequent wildcard operations and advanced manipulations.

Pacing Note: Defer scaffold (4.6) if time-constrained; essentials are creation/removal fundamentals (4.1–4.5).

End of Section 4 Specification.