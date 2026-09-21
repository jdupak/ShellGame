# Section 10 Specification – "Error Streams & Advanced Redirection"

> **Renumbered.** This section used to be number 9; wildcards were moved ahead of
> permissions and redirection so globbing is taught before it is used. Level IDs and
> `level-9/` workspace paths below still use the old `9.x` numbering, and the
> Extension/Optional level statuses were never implemented. Treat this file as the
> original design intent, not as a description of the shipped section.

## 1. Purpose & Scope
Section 9 introduces stderr (standard error) as a separate output stream and advanced redirection techniques. Building on Section 8's stdout redirection, students learn to:
- Distinguish between stdout (stream 1) and stderr (stream 2)
- Redirect stderr separately with `2>`
- Redirect both streams with `&>` or `2>&1`
- Capture errors while suppressing output
- Debug command failures by examining error messages
- Combine stream redirection with pipes

Deliberate exclusions:
- No process substitution or advanced file descriptor manipulation
- No exec or custom file descriptors beyond 0, 1, 2
- No signal handling
- No background processes

Allowed commands: All previous commands plus stream redirection operators: `2>`, `2>&1`, `&>`, `>/dev/null`, `2>/dev/null`
Estimated Time: 8–10 minutes (core Levels 9.1–9.5) + optional Level 9.6 (~2 minutes)
<!-- REVISED: -->
> Time Calibration: Target 7 minutes average; slow path 6 minutes (Levels 9.1–9.4). Level 9.5 becomes Extension (debug capture), 9.6 Optional (aggregation).

## 2. Learning Objectives
By the end of Section 9 the player will:
1. Explain the difference between stdout and stderr.
2. Redirect stderr to a file using `2>`.
3. Redirect both stdout and stderr to the same file using `&>` or `> file 2>&1`.
4. Discard output by redirecting to `/dev/null`.
5. Capture error messages from failing commands.
6. Combine stderr redirection with pipes.
7. Debug scripts by examining captured error output.
8. (Optional) Build complex redirection chains separating success and error logs.

## 3. Concept Tutorial (Displayed Before Level 9.1)
Key concepts:
- **Three standard streams**:
  - stdin (0): input
  - stdout (1): normal output
  - stderr (2): error messages
- **Why separate streams?**: Allows filtering errors independently from results
- **Redirection operators**:
  - `>` or `1>`: redirect stdout only
  - `2>`: redirect stderr only
  - `&>`: redirect both stdout and stderr to same destination
  - `2>&1`: redirect stderr to wherever stdout currently goes
  - Order matters: `> file 2>&1` (stdout to file, then stderr to stdout's location)
- **/dev/null**: A "black hole" device that discards all data written to it
- **Common patterns**:
  - `command 2> errors.log`: save errors, show output
  - `command > output.log 2>&1`: save both to same file
  - `command 2>/dev/null`: suppress errors only
  - `command &>/dev/null`: suppress everything

Visual example:
```
ls existing_file missing_file
  ↓
existing_file          (stdout)
ls: missing_file: No such file or directory    (stderr)

ls existing_file missing_file 2> errors.txt
  ↓
existing_file          (stdout to terminal)
(errors.txt contains: "ls: missing_file: No such file or directory")
```

Short prompt:
"Separate success from failure. Capture errors for debugging. Silence noise when needed. Control every stream."

## 4. Directory Layout (Initial for Section 9)
Base: `$WORKSPACE/level-9/`

Proposed structure:
```
level-9/
├── mixed/
│   ├── good1.txt
│   ├── good2.txt
│   └── good3.txt
├── commands/
│   ├── failing_script.sh        (executable: exits with error)
│   ├── noisy_process.sh         (executable: produces both stdout and stderr)
│   └── silent_worker.sh         (executable: only produces stderr on failure)
├── debug/
│   ├── broken_command.sh        (script with intentional error)
│   └── expected_error.log       (reference for comparison)
├── logs/
│   └── .placeholder
└── capture/
    └── .placeholder
```

Example script contents:
- `failing_script.sh`: `#!/bin/bash\necho "Starting..."\necho "Error: Something failed" >&2\nexit 1`
- `noisy_process.sh`: `#!/bin/bash\necho "Processing..."\necho "Warning: Low memory" >&2\necho "Done"`

## 5. Level Index
| ID   | Title                              | Focus                                  | Answer Type           |
|------|------------------------------------|----------------------------------------|-----------------------|
| 9.1  | Understanding Stream Separation    | Observe stdout vs stderr               | Single word           |
| 9.2  | Redirecting Stderr                 | 2> operator                            | File basename         |
| 9.3  | Redirecting Both Streams           | &> or 2>&1                             | Integer (byte count)  |
| 9.4  | Silencing Output                   | >/dev/null and 2>/dev/null             | Single word           |
| 9.5  | Capturing Command Errors           | Extension                      | Single word           |
| 9.6  | (Optional) Complex Stream Control  | Optional                       | Ordered list          |

## 6. Detailed Level Specifications

### Level 9.1 – Understanding Stream Separation
Start: `$WORKSPACE/level-9/mixed/`
Task: "Run: ls good1.txt missing.txt (where missing.txt doesn't exist). Which stream receives the error message? Submit either 'stdout' or 'stderr'."
Expected behavior:
- `good1.txt` appears (stdout)
- Error message about missing.txt appears (stderr)
Answer: `stderr`
Validation:
- Single word.
- Case-insensitive: 'stderr' or 'STDERR' accepted.
Hints:
1. "Try the command and observe where error messages go."
2. "Normal output goes to stdout; errors go to stderr."
3. "Answer: stderr"
Failure:
- 'stdout' submitted → "Error messages go to stderr, not stdout."

### Level 9.2 – Redirecting Stderr
Start: `$WORKSPACE/level-9/mixed/`
Task: "List both good1.txt and missing.txt, but redirect ONLY errors to errors.log using 2>. After running the command, submit the basename of the error log file without extension."
Command: `ls good1.txt missing.txt 2> errors.log`
Answer: `errors`
Validation:
- File errors.log exists in current directory.
- Contains error message about missing.txt.
- Extension stripped.
Hints:
1. "Use: ls good1.txt missing.txt 2> errors.log"
2. "The 2> redirects only stderr to the file."
3. "Answer: errors"
Failure:
- File missing → "Did you create errors.log with 2>?"
- Extension included → "Remove extension."

### Level 9.3 – Redirecting Both Streams
Start: `$WORKSPACE/level-9/commands/`
Task: "Run ./noisy_process.sh and redirect BOTH stdout and stderr to combined.log. Count the number of bytes in combined.log using wc -c. Submit that count."
Command: `./noisy_process.sh &> combined.log` or `./noisy_process.sh > combined.log 2>&1`
Content (approx 60 bytes including newlines):
```
Processing...
Warning: Low memory
Done
```
Answer: `60` (or exact byte count)
Validation:
- File combined.log exists.
- Contains both normal output and warning.
- Byte count matches.
Hints:
1. "Use: ./noisy_process.sh &> combined.log"
2. "Count bytes with: wc -c combined.log"
3. "Answer: 60" (adjust to actual)
Failure:
- Missing stderr content → "Both streams should be captured—use &> or 2>&1."

### Level 9.4 – Silencing Output
Start: `$WORKSPACE/level-9/commands/`
Task: "Run ./failing_script.sh and suppress ALL output (both stdout and stderr) using /dev/null. The script will still fail (exit code 1), but produce no visible output. After running it silently, what is the name of the device file you redirected to? Submit just the device name (no path)."
Command: `./failing_script.sh &>/dev/null` or `./failing_script.sh >/dev/null 2>&1`
Answer: `null`
Validation:
- Single word.
- Accepts 'null' or 'dev/null' → normalize to 'null'.
Hints:
1. "Use: ./failing_script.sh &>/dev/null"
2. "/dev/null is a special device that discards all data."
3. "Answer: null"
Failure:
- Wrong answer → "The device is /dev/null; submit 'null'."

### Level 9.5 – Capturing Command Errors
Start: `$WORKSPACE/level-9/debug/`
Task: "Run ./broken_command.sh and capture ONLY its error messages to debug.log using 2>. Read debug.log and find the first word of the error message. Submit that word."
Script produces: "ERROR: Configuration file not found"
First word: `ERROR:`
Answer: `ERROR` (with or without colon, normalize)
Validation:
- File debug.log exists.
- Contains error message.
- First word extracted (case-insensitive).
Hints:
1. "Use: ./broken_command.sh 2> debug.log"
2. "Read the file with: cat debug.log"
3. "First word is: ERROR"
Failure:
- Captured stdout instead → "Use 2> to capture stderr only."

### Level 9.6 – (Optional) Complex Stream Control
Start: `$WORKSPACE/level-9/commands/`
Task: "Run ./noisy_process.sh three times:
1. Redirect stdout to success.log, stderr to errors.log
2. Append stdout to success.log, stderr to errors.log (using >>)
3. Append stdout to success.log, stderr to errors.log (using >>)
Count total lines in success.log and errors.log combined. Submit the total."
Commands:
```
./noisy_process.sh > success.log 2> errors.log
./noisy_process.sh >> success.log 2>> errors.log
./noisy_process.sh >> success.log 2>> errors.log
```
Expected:
- success.log: 6 lines (2 normal outputs × 3 runs)
- errors.log: 3 lines (1 warning × 3 runs)
- Total: 9 lines
Answer: `9`
Validation:
- Both files exist.
- Line count matches.
Hints:
1. "First run with >, subsequent with >> to append."
2. "Count total: wc -l success.log errors.log"
3. "Answer: 9"
Optional metadata: `optional=true`

## 7. General Validation Rules (Section 9)
- Trim whitespace.
- File basename answers: strip extension.
- Single word answers: case-insensitive where appropriate.
- Integer answers: strict numeric parsing.
- Verify file existence and content.
- Execute commands in sandboxed environment to verify behavior.

## 8. Hint Strategy
Three hints per level:
1. Command syntax with operator explanation.
2. Conceptual clarification of stream behavior.
3. Explicit answer or verification command.
Track `attempts` and `hints_used`.

## 9. Telemetry / State Logging
Per completion:
```
"9.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 9.1–9.4 required. 9.5 Extension. 9.6 Optional.

## 10. Failure Message Templates
- Wrong stream: "You redirected the wrong stream—check stdout vs stderr."
- File missing: "Output file not created—verify redirection syntax."
- Extension included: "Remove the extension."
- Both streams not captured: "Both stdout and stderr should be in the file—use &>."
- Wrong device: "The discard device is /dev/null."
- Incomplete capture: "Error message not found in file—verify 2> usage."

## 11. Edge Cases & Robustness
- Scripts must be executable (chmod +x during setup).
- /dev/null always available on Unix systems.
- Order sensitivity: `2>&1 > file` vs `> file 2>&1` behave differently (teach correct order).
- Empty stderr: some commands succeed without errors (valid).
- Exit codes: can check but not required for submission (focus on streams).

## 12. Implementation Checklist
- Pre-create all scripts with exact error messages.
- Ensure scripts are executable (chmod +x).
- Provide helpers:
  - `file_exists(path)` → bool
  - `file_contains(path, pattern)` → bool
  - `count_lines(file)` → int
  - `count_bytes(file)` → int
  - `first_word(file)` → string
- Validate captured content matches expected stderr/stdout.
- Reset mechanism reconstructs Section 9 tree and removes created logs.

## 13. Advancement Criteria
After Level 9.4 success:
- `current_level = "10.1"`
Optional Level 9.6 accessible post-advancement.

## 14. Sample Instruction Screen (Level 9.3)
```
══════════════════════════════════════════════════════
LEVEL 9.3: Redirecting Both Streams

Some commands produce both normal output (stdout) and
error/warning messages (stderr). You can capture both
to the same file.

Task:
1. Navigate to level-9/commands/
2. Run: ./noisy_process.sh &> combined.log
3. This captures BOTH streams in one file
4. Count bytes: wc -c combined.log
5. Submit the byte count

Alternative syntax: ./noisy_process.sh > combined.log 2>&1

Submit with: shellgame submit <count>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- Stream separation enables precise control of output and errors.
- /dev/null teaches the concept of intentional data disposal.
- Append vs overwrite distinction critical for log aggregation.
- Error capture essential for debugging production systems.
- Order of redirection operators matters (2>&1 position sensitivity).
- Real-world pattern: separating application logs from error logs.

## 16. Future Cross-References
- Section 10 (wildcards): combined with redirection for batch logging.
- Section 11 (search): grep through error logs.
- Scripting contexts: proper error handling in automation.
- System administration: log rotation and error monitoring.

## 17. Summary (Instructor View)
Section 9 completes the redirection model by introducing stderr as a controllable stream. Students gain debugging literacy—essential for troubleshooting commands, scripts, and system processes. The ability to capture, examine, or suppress error output transforms them from passive command executors to active system diagnosticians.

Pacing Note: Prioritize stream distinction + redirection basics (9.1–9.4) for core timeline.

End of Section 9 Specification.