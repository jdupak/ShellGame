# Section 6 Specification – "Copying & Moving Files"

## 1. Purpose & Scope
Section 6 introduces file and directory manipulation through copying and moving operations. Building on inspection skills from Section 5, students learn to:
- Copy files with `cp`
- Copy directories recursively with `cp -r`
- Move/rename files and directories with `mv`
- Understand the difference between copy (duplicate) and move (relocate/rename)
- Recognize when operations overwrite existing files
- Plan multi-step file organization workflows

Deliberate exclusions:
- No deletion (`rm`, `rmdir`) as that was covered in Section 4.
- No permission preservation flags (`-p`).
- No interactive prompts (`-i`).
- No wildcards yet (deferred to Section 9).
- No symbolic links.

Allowed commands: `pwd`, `ls`, `ls -l`, `cd`, `cp`, `cp -r`, `mv`, `cat`, `file`
Estimated Time: 10 minutes (core Levels 6.1–6.5) + optional Level 6.6 (~2 minutes)

> Time Calibration: Target 8 minutes average; slow path 6 minutes (Levels 6.1–6.4). Level 6.5 becomes Extension (directory rename), 6.6 Optional (multi-step workflow).

## 2. Learning Objectives
By the end of Section 6 the player will:
1. Copy a single file to a new location with `cp source dest`.
2. Understand that copying creates a duplicate (the original remains).
3. Copy an entire directory tree using `cp -r`.
4. Move or rename a file with `mv source dest`.
5. Recognize that moving removes the original.
6. Rename a directory using `mv`.
7. Organize files into directories using planned copy/move sequences.
8. Count files after operations to verify correct outcomes.
9. (Optional) Execute a multi-step reorganization workflow.

## 3. Concept Tutorial (Displayed Before Level 6.1)
Key concepts:
- `cp source destination` → creates a copy of `source` at `destination`. Both files exist after.
- `cp -r source_dir dest_dir` → recursively copies a directory and its contents.
- `mv source destination` → moves (or renames) `source` to `destination`. The source is gone after.
- `mv` works for both files and directories without a `-r` flag.
- If `source` and `destination` are in the same directory, `mv` renames the item.
- Overwriting: If a destination file already exists, it gets replaced without warning.

Visual example:
```
Before: workspace/file.txt
After `cp file.txt backup.txt`:
  workspace/file.txt (original)
  workspace/backup.txt (copy)

After `mv file.txt archive/`:
  archive/file.txt (moved)
  workspace/ (file.txt is gone)
```

Short prompt:
"Copy duplicates. Move relocates. Plan your operations, then verify with `ls`."

## 4. Directory Layout (Initial for Section 6)
Base: `$WORKSPACE/level-6/`

Proposed structure:
```
level-6/
├── source/
│   ├── document.txt
│   ├── report.md
│   ├── data.csv
│   └── notes/
│       ├── todo.txt
│       └── ideas.md
├── backup/
│   └── .placeholder
├── archive/
│   └── old.log
├── organize/
│   ├── photo1.jpg
│   ├── photo2.jpg
│   ├── script.sh
│   ├── script.py
│   ├── readme.txt
│   └── manual.pdf
└── workspace/
    └── .placeholder
```

## 5. Level Index
| ID   | Title                              | Focus                                  | Answer Type           |
|------|------------------------------------|----------------------------------------|-----------------------|
| 6.1  | Simple File Copy                   | `cp` single file                       | Integer (count)       |
| 6.2  | Directory Copy                     | `cp -r` directory tree                 | Integer (count)       |
| 6.3  | File Rename with `mv`              | `mv` in same directory (rename)        | File basename         |
| 6.4  | File Move                          | `mv` to different directory            | Integer (count)       |
| 6.5  | Directory Rename                    | Extension                        | Directory basename    |
| 6.6  | (Optional) Multi-Step Organization  | Optional                         | Ordered list          |

## 6. Detailed Level Specifications

### Level 6.1 – Simple File Copy
Start: `$WORKSPACE/level-6/source/`
Task: "Copy `document.txt` to the `backup/` directory. After copying, count the total number of `.txt` files that exist across BOTH `source/` and `backup/`. Submit the count."
Answer: `2`
Hints:
1. "Use: `cp document.txt ../backup/`"
2. "After copying, the original still exists. Check both directories."
3. "The total count should be 2."

### Level 6.2 – Directory Copy
Start: `$WORKSPACE/level-6/source/`
Task: "Copy the entire `notes/` directory to `workspace/` using `cp -r`. Count the total number of files inside `workspace/notes/` after copying. Submit the count."
Answer: `2`
Hints:
1. "Use: `cp -r notes ../workspace/`"
2. "The `-r` flag is required to copy directories recursively."
3. "Count the files inside the new `workspace/notes/` directory. There should be 2."

### Level 6.3 – File Rename with `mv`
Start: `$WORKSPACE/level-6/source/`
Task: "Rename `report.md` to `final_report.md`. Submit the new basename without the extension."
Answer: `final_report`
Hints:
1. "Use: `mv report.md final_report.md`"
2. "Using `mv` on two files in the same directory results in a rename."
3. "The answer is `final_report`."

### Level 6.4 – File Move
Start: `$WORKSPACE/level-6/source/`
Task: "Move `data.csv` to the `archive/` directory. After moving, count how many `.csv` files remain in `source/`. Submit the count."
Answer: `0`
Hints:
1. "Use: `mv data.csv ../archive/`"
2. "After a move, the file is gone from the original location."
3. "The count of `.csv` files remaining in `source/` is 0."

### Level 6.5 – Directory Rename
Start: `$WORKSPACE/level-6/`
Task: "Rename the `workspace/` directory to `projects/`. Submit the new directory basename."
Answer: `projects`
Hints:
1. "Use: `mv workspace projects`"
2. "`mv` works on directories without needing a `-r` flag."
3. "The answer is `projects`."

### Level 6.6 – (Optional) Multi-Step Organization
Start: `$WORKSPACE/level-6/organize/`
Task: "Organize files by type. First, create directories: `images/`, `scripts/`, and `docs/`. Then, move `photo1.jpg` and `photo2.jpg` to `images/`, `script.sh` and `script.py` to `scripts/`, and `readme.txt` and `manual.pdf` to `docs/`. After organizing, submit a comma-separated list of the new directory names that contain exactly 2 files (alphabetically ordered)."
Answer: `docs,images,scripts`
Hints:
1. "Create the directories first with `mkdir`."
2. "Use `mv` to move each file into its correct new directory."
3. "All three directories should have 2 files. The answer is `docs,images,scripts`."

## 7. General Validation Rules
- Trim whitespace from answers.
- File/directory basename answers should not include extensions where specified.
- Existence checks: verify source is removed for `mv` and preserved for `cp`.

## 8. Hint Strategy
Three hints per level:
1. Command syntax reminder.
2. Conceptual clarification (copy vs. move).
3. Explicit answer or verification command.

## 9. Telemetry / State Logging
Per completion:
```
"6.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 6.1–6.4 required. 6.5 Extension. 6.6 Optional.

## 10. Failure Message Templates
- "You copied instead of moved—the original should be gone after `mv`."
- "Destination not found—verify the path."
- "The file count does not match—list files to verify."
- "The directory was copied, not moved/renamed."

## 11. Implementation Checklist
- Pre-create all source files and directories.
- Implement helper functions for file/dir existence and counting.
- Validation logic must track both source and destination states.
- Implement a reset mechanism for the Section 6 tree.

## 12. Advancement Criteria
After Level 6.4 success:
- `current_level = "7.1"`
Optional Level 6.6 is accessible post-advancement.

## 13. Sample Instruction Screen (Level 6.4)
```
══════════════════════════════════════════════════════
LEVEL 6.4: File Move

`mv` relocates a file—it disappears from the original
location and appears at the destination.

Task:
1. Navigate to `level-6/source/`
2. Move `data.csv` to `../archive/`
3. Verify `data.csv` is GONE from `source/`
4. Count remaining `.csv` files in `source/`
5. Submit that count.

Command: `mv data.csv ../archive/`

Submit with: shellgame submit <count>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 14. Pedagogical Reinforcement Points
- The copy vs. move distinction is fundamental to file management.
- Recursive copy (`-r`) introduces tree manipulation.
- Renaming as a same-directory move clarifies the mental model.
- Counting exercises verify the success of operations.
- Multi-step organization simulates real-world workflow planning.

## 15. Future Cross-References
- **Section 7 (Permissions)**: Recognize how permissions are handled during copy operations.
- **Section 8 (Content & Redirection)**: Combine file creation with immediate organization.
- **Section 9 (Wildcards)**: Will enable batch operations like `cp *.txt backup/`.

## 16. Summary (Instructor View)
Section 6 empowers students to reshape their workspace intentionally. The copy/move duality solidifies their understanding of filesystem state transitions (duplication vs. relocation). By the end, students can confidently reorganize files and verify outcomes—a critical skill for project management and system maintenance.

Pacing Note: Essentials = copy vs move semantics (6.1–6.4); rename and workflow after advancement.

End of Section 6 Specification.