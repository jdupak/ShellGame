# Section 11 Specification – "Search & Discovery"

## 1. Purpose & Scope
Section 11 introduces powerful search tools for locating files and content across directory trees. Building on all prior sections, students learn to:
- Use `find` to search for files by name, type, and attributes
- Use `grep -r` for recursive content searching
- Combine `find` with other commands using `-exec`
- Count files and content matches with `wc`
- Navigate large directory structures efficiently
- Build complex search queries with multiple criteria

Deliberate exclusions:
- No advanced find predicates (`-mtime`, `-user`, `-perm` beyond basics)
- No xargs (introduce `-exec` instead)
- No locate/updatedb
- No advanced grep options (PCRE, context lines)
- No ack/ag/ripgrep alternatives

Allowed commands: All previous commands plus `find`, `grep -r`, `grep -i`, `wc`
Estimated Time: 12–15 minutes (core Levels 11.1–11.6) + optional Level 11.7 (~3 minutes)
> Time Calibration: Target 9 minutes average; slow path 7 minutes (Levels 11.1–11.4). Levels 11.5–11.6 become Extension (size + case-insensitive), 11.7 Optional.

## 2. Learning Objectives
By the end of Section 11 the player will:
1. Use `find` to locate files by name pattern.
2. Use `find` to search by file type (file vs directory).
3. Use `find` with size predicates.
4. Use `grep -r` to search for text patterns recursively.
5. Combine `grep -r` with `wc -l` to count matches across files.
6. Use `find` with `-exec` to perform actions on found files.
7. Build multi-criteria searches (name AND type AND size).
8. Search case-insensitively with `grep -i`.
9. (Optional) Construct complex find/grep pipelines for data extraction.

## 3. Concept Tutorial (Displayed Before Level 11.1)
Key concepts:
- **`find`**: Searches directory trees for files matching criteria
  - Syntax: `find <path> <predicates>`
  - Common predicates:
    - `-name "pattern"`: match filename (glob pattern, quoted)
    - `-type f`: files only
    - `-type d`: directories only
    - `-size +1M`: larger than 1 megabyte
    - `-size -100c`: smaller than 100 bytes
    - `-exec command {} \;`: execute command on each match
  - Example: `find . -name "*.txt"` finds all .txt files in current tree
- **`grep -r`**: Searches file contents recursively
  - Syntax: `grep -r "pattern" <path>`
  - Returns: filename:matching_line
  - Example: `grep -r "ERROR" logs/` finds all ERROR mentions in logs/
- **`grep -i`**: Case-insensitive search
  - `grep -i "error"` matches ERROR, Error, error
- **Combining tools**:
  - `find . -name "*.log" -exec grep "ERROR" {} \;`
  - `grep -r "pattern" | wc -l` counts matching lines
  - `find . -type f | wc -l` counts all files
- **Search efficiency**: narrow search path to speed results

Visual example:
```
project/
├── src/
│   ├── main.py
│   └── utils.py
├── tests/
│   └── test_main.py
└── docs/
    └── readme.txt

find . -name "*.py"
→ ./src/main.py
→ ./src/utils.py
→ ./tests/test_main.py

grep -r "def main"
→ ./src/main.py:def main():
→ ./tests/test_main.py:def main_test():
```

Short prompt:
"Find files anywhere. Search content everywhere. Narrow by criteria. Extract precisely what you seek."

## 4. Directory Layout (Initial for Section 11)
Base: `$WORKSPACE/level-11/`

Proposed structure:
```
level-11/
├── docs/
│   ├── report.txt          (content: "Annual summary 2025")
│   ├── notes.txt           (content: "Meeting notes")
│   ├── draft.md            (content: "Draft proposal")
│   └── archive/
│       ├── old_report.txt  (content: "Legacy data")
│       └── backup.txt      (content: "Backup copy")
├── logs/
│   ├── app.log             (content: 50 lines, 5 contain "ERROR")
│   ├── system.log          (content: 80 lines, 3 contain "ERROR")
│   ├── debug.log           (content: 30 lines, 0 contain "ERROR")
│   └── archive/
│       ├── old_app.log     (content: 100 lines, 8 contain "ERROR")
│       └── old_system.log  (content: 60 lines, 2 contain "ERROR")
├── code/
│   ├── main.py             (content: Python code with "def main")
│   ├── utils.py            (content: Python code with "def helper")
│   ├── config.json         (content: JSON configuration)
│   └── lib/
│       ├── parser.py       (content: Python code with "def parse")
│       └── validator.py    (content: Python code with "def validate")
├── mixed/
│   ├── small.txt           (50 bytes)
│   ├── medium.dat          (500 bytes)
│   ├── large.bin           (5000 bytes)
│   ├── tiny.log            (10 bytes)
│   └── data/
│       ├── file1.txt       (200 bytes)
│       ├── file2.txt       (300 bytes)
│       └── file3.dat       (400 bytes)
└── search-target/
    ├── password.txt        (content: "SECRET: admin123")
    ├── config.yaml         (content: "password: changeme")
    └── deep/
        └── nested/
            └── credentials.conf  (content: "PASSWORD=test123")
```

## 5. Level Index
| ID    | Title                              | Focus                                  | Answer Type           |
|-------|------------------------------------|----------------------------------------|-----------------------|
| 11.1  | Finding by Name                    | find -name pattern                     | Integer (count)       |
| 11.2  | Finding by Type                    | find -type f/d                         | Integer (count)       |
| 11.3  | Recursive Content Search           | grep -r pattern                        | File basename         |
| 11.4  | Count Matching Lines               | grep -r \| wc -l                        | Integer (count)       |
| 11.5  | Finding by Size                    | Extension                          | File basename         |
| 11.6  | Case-Insensitive Search            | Extension                          | Integer (count)       |
| 11.7  | (Optional) Complex Search Combo    | Optional                           | Ordered list          |

## 6. Detailed Level Specifications

### Level 11.1 – Finding by Name
Start: `$WORKSPACE/level-11/`
Task: "Find all files named '*.txt' (anywhere in level-11 tree) using find. Count them with: find . -name '*.txt' | wc -l. Submit the count."
Files matching: report.txt, notes.txt, old_report.txt, backup.txt, small.txt, file1.txt, file2.txt, password.txt (8 files)
Answer: `8`
Validation:
- Integer.
- Matches actual count of .txt files recursively.
Hints:
1. "Use: find . -name '*.txt' | wc -l"
2. "Quote the pattern to prevent shell expansion."
3. "Answer: 8"
Failure:
- Wrong count → "Verify: find . -name '*.txt'"

### Level 11.2 – Finding by Type
Start: `$WORKSPACE/level-11/code/`
Task: "Count only directories (not files) under current directory using find . -type d. Submit the count (excluding . itself)."
Directories: lib/ (1 subdirectory)
Answer: `1`
Validation:
- Integer.
- Excludes starting directory (.).
Hints:
1. "Use: find . -type d"
2. "-type d finds directories only."
3. "Exclude current dir (.); answer: 1"
Failure:
- Includes current dir → "Exclude the starting directory (.)."

### Level 11.3 – Recursive Content Search
Start: `$WORKSPACE/level-11/docs/`
Task: "Search recursively for files containing the word 'Legacy' using grep -r 'Legacy' . Find the basename (without extension) of the file containing it."
File containing "Legacy": archive/old_report.txt
Answer: `old_report`
Validation:
- Single file basename.
- Extension stripped.
Hints:
1. "Use: grep -r 'Legacy' ."
2. "Output shows filename:line with match."
3. "Answer: old_report"
Failure:
- Extension included → "Remove extension."
- Full path → "Submit basename only."

### Level 11.4 – Count Matching Lines
Start: `$WORKSPACE/level-11/logs/`
Task: "Count total lines containing 'ERROR' across all .log files recursively. Use: grep -r 'ERROR' . | wc -l. Submit the count."
Matches: app.log (5) + system.log (3) + old_app.log (8) + old_system.log (2) = 18 lines
Answer: `18`
Validation:
- Integer.
- Matches total across all log files.
Hints:
1. "Use: grep -r 'ERROR' . | wc -l"
2. "This counts matching lines across all files."
3. "Answer: 18"
Failure:
- Wrong count → "Ensure searching all .log files recursively."

### Level 11.5 – Finding by Size
Start: `$WORKSPACE/level-11/mixed/`
Task: "Find files larger than 1000 bytes using find . -size +1000c. Submit the basename (without extension) of the largest file found."
Files >1000 bytes: large.bin (5000 bytes)
Answer: `large`
Validation:
- File basename.
- Extension stripped.
- Matches largest file.
Hints:
1. "Use: find . -size +1000c"
2. "+1000c means more than 1000 bytes."
3. "Answer: large"
Failure:
- Wrong file → "Find files larger than 1000 bytes."
- Extension included → "Remove extension."

### Level 11.6 – Case-Insensitive Search
Start: `$WORKSPACE/level-11/search-target/`
Task: "Count files containing 'password' or 'PASSWORD' (case-insensitive) using grep -ri 'password' . | wc -l. Submit the count of matching lines."
Matches:
- password.txt: 1 line ("SECRET: admin123" contains no password word) - WAIT, file is named password.txt but we search content
- config.yaml: 1 line ("password: changeme")
- credentials.conf: 1 line ("PASSWORD=test123")
Total: 2 lines (config.yaml + credentials.conf)
Answer: `2`
Validation:
- Integer.
- Case-insensitive matches.
Hints:
1. "Use: grep -ri 'password' . | wc -l"
2. "-i makes search case-insensitive."
3. "Answer: 2"
Failure:
- Case-sensitive count → "Use -i flag for case-insensitive."

### Level 11.7 – (Optional) Complex Search Combo
Start: `$WORKSPACE/level-11/code/`
Task: "Find all .py files containing the word 'def', then count how many unique files (not lines) contain it. Use: find . -name '*.py' -exec grep -l 'def' {} \; | wc -l. Submit the count."
Python files with 'def': main.py, utils.py, parser.py, validator.py (4 files)
Answer: `4`
Validation:
- Integer.
- Counts unique files, not lines.
- -l flag lists filenames only.
Hints:
1. "Use: find . -name '*.py' -exec grep -l 'def' {} \\;"
2. "grep -l lists filenames only (not lines)."
3. "Answer: 4"
Optional metadata: `optional=true`

## 7. General Validation Rules (Section 11)
- Trim whitespace.
- Integer answers: strict numeric parsing.
- File basename answers: strip extension.
- Ordered lists: comma-separated, alphabetically sorted.
- Find output: one result per line.
- Grep output format: filename:line_content.
- Case sensitivity: respect unless -i flag used.

## 8. Hint Strategy
Three hints per level:
1. Command syntax with flags.
2. Explanation of predicate or option behavior.
3. Explicit answer or verification command.
Track `attempts` and `hints_used`.

## 9. Telemetry / State Logging
Per completion:
```
"11.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 11.1–11.4 required. 11.5–11.6 Extension. 11.7 Optional.

## 10. Failure Message Templates
- Wrong count: "Count does not match—verify search criteria."
- Extension included: "Remove the extension."
- Full path submitted: "Submit basename only."
- Case sensitivity error: "Use -i for case-insensitive search."
- Wrong predicate: "Check find predicate syntax."
- File vs directory confusion: "Use -type f for files, -type d for directories."

## 11. Edge Cases & Robustness
- Empty search results: valid (count = 0).
- Symbolic links: follow or not (find default: follow).
- Permission denied: may appear in output (filter with 2>/dev/null if needed).
- Quote protection: patterns must be quoted to prevent shell expansion.
- Large trees: search may be slow (acceptable in game environment).

## 12. Implementation Checklist
- Pre-create all files with exact content and sizes.
- Provide helpers:
  - `find_files(path, name_pattern)` → list
  - `grep_recursive(path, pattern, case_insensitive)` → list
  - `count_matches(results)` → int
  - `strip_extension(filename)` → string
- Validate search results match expected counts.
- Reset mechanism reconstructs Section 11 tree.
- Ensure file sizes exact for size-based searches.

## 13. Advancement Criteria
After Level 11.4 success:
- Game complete (core). Offer Extension levels then Optional 11.7.
- Show completion summary with:
  - Total time
  - Total hints used
  - All sections completed
  - Optional challenges completed

## 14. Sample Instruction Screen (Level 11.4)
```
══════════════════════════════════════════════════════
LEVEL 11.4: Count Matching Lines

grep -r searches file contents recursively. Pipe to wc -l
to count total matching lines across all files.

Task:
1. Navigate to level-11/logs/
2. Search for 'ERROR' in all files: grep -r 'ERROR' .
3. Count matches: grep -r 'ERROR' . | wc -l
4. Submit the total count

This searches ALL files recursively, including subdirectories.

Submit with: shellgame submit -f <count>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- find enables file location by attributes, not just content.
- grep -r scales content search across entire trees.
- Combining tools (find + grep + wc) creates powerful queries.
- Case-insensitive search essential for user-generated content.
- -exec demonstrates command chaining at filesystem level.
- Real-world: log analysis, codebase exploration, security audits.

## 16. Future Cross-References
- Scripting: loops over find results.
- Performance: learn when to narrow search paths.
- Security: finding sensitive data (passwords, keys).

## 17. Summary (Instructor View)
Section 11 equips students with industrial-strength search capabilities. The find/grep combination is fundamental to system administration, security analysis, and software development. By mastering recursive search patterns, students gain the ability to interrogate large codebases and filesystems efficiently—transforming from manual navigators to automated discoverers.

Pacing Note: Core focuses on name/type/content + counts before advanced predicates.

End of Section 11 Specification.