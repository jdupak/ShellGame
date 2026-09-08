# Section 7 Specification – "Permissions Matter"

## 1. Purpose & Scope
Section 7 introduces file permissions, a fundamental concept in Unix-like systems. Building on file inspection skills from Section 5, students will learn to:
- Read the permission string in `ls -l` output.
- Understand the meaning of read (`r`), write (`w`), and execute (`x`) for owner, group, and others.
- Modify file permissions using `chmod`.
- Make a script executable.
- Protect a file by removing write permissions.

Deliberate exclusions:
- No advanced permission topics like `setuid`, `setgid`, or sticky bits.
- No `umask` or default permissions.
- No file ownership changes (`chown`, `chgrp`).

Allowed commands: `pwd`, `ls`, `ls -l`, `cd`, `chmod`, `cat`
Estimated Time: 7 minutes (core Levels 7.1–7.4)
> Time Calibration: Target 6 minutes average; slow path 5 minutes focusing on 7.1, 7.2, 7.4. Level 7.3 (protect & error) becomes Extension (can revisit after numeric mode).

## 2. Learning Objectives
By the end of Section 7, the player will be able to:
1.  Identify the owner, group, and other permissions for a file.
2.  Recognize which files are executable by looking at their permissions.
3.  Add execute permissions to a file using `chmod +x`.
4.  Remove write permissions using `chmod -w`.
5.  Apply permissions using numeric modes (e.g., `chmod 755`).
6.  Understand the consequence of trying to write to a file without write permission.

## 3. Concept Tutorial (Displayed Before Level 7.1)
Key concepts:
-   **Permissions**: Every file and directory has permissions that control who can read, write, or execute it.
-   **`ls -l` Output**: The first 10 characters show the permissions (e.g., `-rwxr-xr--`).
    -   The first character is the file type (`-` for file, `d` for directory).
    -   The next 3 are for the **owner** (`rwx` = read, write, execute).
    -   The next 3 are for the **group** (`r-x` = read, execute).
    -   The final 3 are for **others** (`r--` = read only).
-   **`chmod`**: The command to "change mode" (change permissions).
    -   **Symbolic mode**: `chmod +x file` (adds execute), `chmod -w file` (removes write). You can specify `u` (user/owner), `g` (group), `o` (other), e.g., `chmod u+x file`.
    -   **Numeric (octal) mode**: Each permission has a value: `r`=4, `w`=2, `x`=1. Sum them up for each category. `rwx` = 4+2+1=7. `r-x` = 4+0+1=5. `r--` = 4+0+0=4. So, `rwxr-xr--` is `754`.

Short prompt:
"Permissions control who can do what. Use `ls -l` to see them and `chmod` to change them. Make your scripts runnable and your data safe."

## 4. Directory Layout (Initial for Section 7)
Base: `$WORKSPACE/level-7/`

```
level-7/
├── scripts/
│   ├── run_me.sh         (permissions: 755, rwxr-xr-x)
│   └── needs_fixing.sh   (permissions: 644, rw-r--r--)
├── data/
│   ├── report.txt        (permissions: 666, rw-rw-rw-)
│   └── protected.dat     (permissions: 444, r--r--r--)
└── numeric/
    └── target.sh         (permissions: 644, rw-r--r--)
```

## 5. Level Index
| ID   | Title                          | Focus                               | Answer Type          |
|------|--------------------------------|-------------------------------------|----------------------|
| 7.1  | Find the Executable            | Reading `x` permission bit          | File basename        |
| 7.2  | Make a Script Executable       | `chmod +x`                          | Permission string    |
| 7.3  | Protect a File                 | Extension                         | Error keyword        |
| 7.4  | Apply Numeric Mode             | `chmod 755`                         | Permission string    |

## 6. Detailed Level Specifications

### Level 7.1 – Find the Executable
Start: `$WORKSPACE/level-7/scripts/`
Task: "One of the scripts in this directory is already executable. Use `ls -l` to find it. Submit its basename without the extension."
Target: `run_me.sh`
Answer: `run_me`
Validation:
-   The submitted name must correspond to the file with execute (`x`) permissions.
Hints:
1.  "Use `ls -l` to view the permissions for all files."
2.  "Look for an `x` in the permission string (e.g., `-rwxr-xr-x`)."
3.  "The executable file is `run_me.sh`."

### Level 7.2 – Make a Script Executable
Start: `$WORKSPACE/level-7/scripts/`
Task: "The script `needs_fixing.sh` is not executable. Add execute permission for the owner (`u`), group (`g`), and others (`o`). After you run the command, submit the new permission string for the owner (the first three letters after the initial dash)."
Action: `chmod +x needs_fixing.sh` or `chmod 755 needs_fixing.sh`.
Initial permissions: `rw-r--r--`. Final permissions: `rwxr-xr-x`.
Owner's permission trio: `rwx`.
Answer: `rwx`
Validation:
-   The file `needs_fixing.sh` must have execute permissions for all.
-   The submitted answer must be the owner's permission string.
Hints:
1.  "Use `chmod +x <filename>` to add execute permission for everyone."
2.  "After running `chmod`, use `ls -l` again to see the new permissions."
3.  "The owner's permissions will be `rwx`."

### Level 7.3 – Protect a File
Start: `$WORKSPACE/level-7/data/`
Task: "The file `report.txt` can be written to by anyone. Remove the write permission (`w`) for the 'other' users. Then, try to append text to it with `echo 'test' >> report.txt`. The command will fail. Submit the key word from the error message."
Action: `chmod o-w report.txt`. Then `echo 'test' >> report.txt`.
Error message: `bash: report.txt: Permission denied`
Answer: `denied`
Validation:
-   The file `report.txt` must have `o-w` permissions.
-   The submitted word must be `denied` (case-insensitive).
Hints:
1.  "Use `chmod o-w report.txt` to remove write permission for 'others'."
2.  "After changing the permission, try to append to the file: `echo 'test' >> report.txt`."
3.  "The error message contains the word `denied`."

### Level 7.4 – Apply Numeric Mode
Start: `$WORKSPACE/level-7/numeric/`
Task: "Use the numeric mode to set the permissions of `target.sh` to `755` (owner can read/write/execute, group and others can read/execute). After setting it, submit the new permission string for 'other' users (the last three characters)."
Action: `chmod 755 target.sh`.
Final permissions: `rwxr-xr-x`.
"Other" permissions: `r-x`.
Answer: `r-x`
Validation:
-   The file `target.sh` must have `755` permissions.
-   The submitted answer must be the "other" permission string.
Hints:
1.  "Use the command `chmod 755 target.sh`."
2.  "Remember, `755` translates to `rwxr-xr-x`."
3.  "The last three characters of the permission string are `r-x`."

## 7. General Validation Rules
-   Trim whitespace from answers.
-   Permission string answers are case-sensitive.
-   Error message keywords are case-insensitive.
-   Validation logic will use `stat` or `ls -l` parsing to check the actual file modes on the filesystem.

## 8. Hint Strategy
1.  High-level concept reminder (e.g., "Use `chmod`...").
2.  Specific syntax suggestion (e.g., "`chmod +x ...`").
3.  Explicit answer or verification command.

## 9. Telemetry / State Logging
Per completion:
```
"7.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Core levels 7.1, 7.2, 7.4 required. 7.3 Extension.

## 10. Failure Message Templates
-   "That's not the right permission string. Use `ls -l` to double-check."
-   "The file does not have the correct permissions yet. Try the `chmod` command again."
-   "That's not the keyword from the error message. Make sure you are running the correct command after changing permissions."

## 11. Implementation Checklist
-   Create the initial directory structure and files.
-   Use `chmod` in the setup script to set the initial permissions for each level.
-   Implement validation helpers that can read and parse file permission strings.
-   Implement a reset mechanism for the Section 7 tree.

## 12. Advancement Criteria
After Level 7.4 success:
-   `current_level = "8.1"`

## 13. Sample Instruction Screen (Level 7.2)
```
══════════════════════════════════════════════════════
LEVEL 7.2: Make a Script Executable

The script `needs_fixing.sh` can't be run because it's
missing the execute permission.

Task:
1. Use `ls -l` to see the current permissions.
2. Use `chmod +x needs_fixing.sh` to add execute permission.
3. Use `ls -l` again to see the change.
4. Submit the owner's new permission string (e.g., `rwx`).

Submit with: shellgame submit <permission-string>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 14. Pedagogical Reinforcement Points
-   Connects the abstract concept of permissions to the practical ability to run a script.
-   Demonstrates how permissions provide a layer of safety (preventing accidental writes).
-   Introduces both symbolic and numeric modes, as both are common in the real world.

## 15. Future Cross-References
-   **Section 11 (Search & Discovery)**: Extend with `find . -perm 755` queries.

## 16. Summary (Instructor View)
Section 7 covers the critical topic of file permissions. Students learn not just the theory but the practical application: making scripts runnable and protecting files. This knowledge is essential for any user in a multi-user environment and is a prerequisite for writing and deploying simple shell scripts.

Time Note: Teach execute + numeric mode first (7.1, 7.2, 7.4). File protection scenario enriches after basics.

End of Section 7 Specification.