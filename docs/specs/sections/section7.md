# Section 7 Specification – "Wildcards & Pattern Matching"

> **Renumbered.** This section used to be number 10; wildcards were moved ahead of
> permissions and redirection so globbing is taught before it is used. Level IDs and
> `level-10/` workspace paths below still use the old `10.x` numbering, and the
> Extension/Optional level statuses were never implemented. Treat this file as the
> original design intent, not as a description of the shipped section.

## 1. Purpose & Scope
Section 10 introduces shell globbing (wildcards) for batch file operations. Building on all prior sections, students learn to:
- Use `*` (match any characters)
- Use `?` (match single character)
- Use `[...]` (character classes)
- Use `[!...]` (negated character classes)
- Apply wildcards with commands learned previously (ls, cp, mv, rm)
- Understand glob expansion happens before command execution
- Recognize when patterns match nothing (literal interpretation)
- Combine wildcards with redirection and pipes

Deliberate exclusions:
- No extended globs (`**`, `?(pattern)`, etc.)
- No brace expansion (`{a,b,c}`)
- No regular expressions (grep regex covered minimally)
- No find command (deferred to Section 11)

Allowed commands: All previous commands plus wildcard patterns: `*`, `?`, `[abc]`, `[!abc]`, `[a-z]`

**Shell requirement:** Exercises using `?`, bracket sets, or ranges require
Bash (currently 10.2–10.4). Display this requirement before those commands.
Fish players run the provided quoted `bash -c 'command'` example, then submit
from their existing wrapped shell. Do not switch them into an unwrapped
interactive shell. Exercises using only `*` remain portable.

Estimated Time: 10–12 minutes (core Levels 10.1–10.6) + optional Level 10.7 (~2 minutes)
<!-- REVISED: -->
> Time Calibration: Target 7 minutes average; slow path 5–6 minutes (Levels 10.1–10.4). 10.5–10.6 become Extension (operations), 10.7 Optional.

## 2. Learning Objectives
By the end of Section 10 the player will:
1. Use `*` to match zero or more characters.
2. Use `?` to match exactly one character.
3. Use `[abc]` to match one character from a set.
4. Use `[!abc]` or `[^abc]` to match one character NOT in a set.
5. Use ranges like `[0-9]` or `[a-z]`.
6. Apply wildcards to list, copy, move, and remove multiple files at once.
7. Understand that the shell expands globs before passing to commands.
8. Count matches using wildcard patterns.
9. (Optional) Combine complex patterns for precise batch operations.

## 3. Concept Tutorial (Displayed Before Level 10.1)
Key concepts:
- **Wildcards (globs)**: Patterns the shell expands to matching filenames
- **`*`**: Matches zero or more characters
  - `*.txt` matches all files ending in .txt
  - `log_*` matches all files starting with log_
  - `*` matches all non-hidden files (no leading dot)
- **`?`**: Matches exactly one character
  - `file?.txt` matches file1.txt, fileA.txt but not file12.txt
  - `data_?.csv` matches data_1.csv through data_9.csv
- **`[...]`**: Character class—matches one character from set
  - `file[123].txt` matches file1.txt, file2.txt, file3.txt
  - `[aeiou]*` matches files starting with vowels
  - `[0-9]` matches single digit
  - `[a-z]` matches single lowercase letter
- **`[!...]`** or **`[^...]`**: Negated class—matches one character NOT in set
  - `[!0-9]*` matches files NOT starting with digit
- **Expansion order**: Shell expands globs BEFORE running command
  - `ls *.txt` → shell finds matches → runs `ls file1.txt file2.txt ...`
- **No matches**: If pattern matches nothing, often treated as literal string (varies by shell)

Visual example:
```
Files: data_1.csv, data_2.csv, image.jpg, readme.txt, script.sh

ls *.csv        → data_1.csv data_2.csv
ls data_?.csv   → data_1.csv data_2.csv
ls [dr]*        → data_1.csv data_2.csv readme.txt
ls *.[st]*      → readme.txt script.sh
```

Short prompt:
"Wildcards multiply your power. One pattern, many files. Master globs to work at scale."

## 4. Directory Layout (Initial for Section 10)
Base: `$WORKSPACE/level-10/`

Proposed structure:
```
level-10/
├── patterns/
│   ├── file1.txt
│   ├── file2.txt
│   ├── file3.txt
│   ├── file10.txt
│   ├── data_a.csv
│   ├── data_b.csv
│   ├── data_c.csv
│   ├── image1.jpg
│   ├── image2.jpg
│   ├── script.sh
│   ├── readme.md
│   └── archive.tar
├── single/
│   ├── log_1.txt
│   ├── log_2.txt
│   ├── log_3.txt
│   ├── log_a.txt
│   ├── log_b.txt
│   └── summary.txt
├── classes/
│   ├── alpha.txt
│   ├── beta.txt
│   ├── gamma.txt
│   ├── delta.txt
│   ├── 1_report.txt
│   ├── 2_report.txt
│   ├── 3_report.txt
│   └── summary.log
├── batch/
│   ├── temp1.tmp
│   ├── temp2.tmp
│   ├── temp3.tmp
│   ├── keep1.txt
│   ├── keep2.txt
│   └── important.doc
├── organize/
│   └── .placeholder
└── advanced/
    ├── a1.dat
    ├── a2.dat
    ├── b1.dat
    ├── b2.dat
    ├── c1.dat
    ├── x.dat
    └── y.dat
```

## 5. Level Index
| ID    | Title                              | Focus                                  | Answer Type           |
|-------|------------------------------------|----------------------------------------|-----------------------|
| 10.1  | Star Wildcard Basics               | * for multiple matches                 | Integer (count)       |
| 10.2  | Question Mark Single Match         | ? for single character                 | Integer (count)       |
| 10.3  | Character Classes                  | [abc] sets                             | Integer (count)       |
| 10.4  | Negated Character Classes          | [!abc] exclusion                       | Integer (count)       |
| 10.5  | Batch File Operations              | Extension                        | Integer (count)       |
| 10.6  | Pattern-Based Cleanup              | Extension                        | Integer (remaining)   |
| 10.7  | (Optional) Complex Pattern Combo   | Optional                         | Ordered list          |

## 6. Detailed Level Specifications

### Level 10.1 – Star Wildcard Basics
Start: `$WORKSPACE/level-10/patterns/`
Task: "Count how many files match the pattern *.txt using ls *.txt | wc -l. Submit the count."
Files matching: file1.txt, file2.txt, file3.txt, file10.txt (4 files)
Answer: `4`
Validation:
- Integer.
- Matches actual *.txt count.
Hints:
1. "Use: ls *.txt | wc -l"
2. "* matches zero or more characters."
3. "Answer: 4"
Failure:
- Wrong count → "Verify with ls *.txt"

### Level 10.2 – Question Mark Single Match
Start: `$WORKSPACE/level-10/single/`
Task: "Count files matching log_?.txt (single character between underscore and dot). Submit the count."
Files matching: log_1.txt, log_2.txt, log_3.txt, log_a.txt, log_b.txt (5 files)
NOT matching: summary.txt (doesn't match pattern)
Answer: `5`
Validation:
- Integer.
- ? matches exactly one character.
Hints:
1. "Use: ls log_?.txt | wc -l"
2. "? matches exactly one character."
3. "Answer: 5"
Failure:
- Includes summary.txt → "Pattern is log_?.txt—summary doesn't match."

### Level 10.3 – Character Classes
Start: `$WORKSPACE/level-10/classes/`
Task: "Count files starting with vowels (a, e, i, o, u) using pattern [aeiou]*.txt. Submit the count."
Files matching: alpha.txt (1 file)
NOT matching: beta.txt, gamma.txt, delta.txt (start with consonants)
Answer: `1`
Validation:
- Integer.
- Only files starting with vowels.
Hints:
1. "Use: ls [aeiou]*.txt | wc -l"
2. "Character class [aeiou] matches one vowel."
3. "Answer: 1"
Failure:
- Includes consonants → "Only vowels: a, e, i, o, u"

### Level 10.4 – Negated Character Classes
Start: `$WORKSPACE/level-10/classes/`
Task: "Count .txt files NOT starting with a digit using [!0-9]*.txt. Submit the count."
Files matching: alpha.txt, beta.txt, gamma.txt, delta.txt (4 files)
NOT matching: 1_report.txt, 2_report.txt, 3_report.txt
Answer: `4`
Validation:
- Integer.
- Excludes files starting with digits.
Hints:
1. "Use: ls [!0-9]*.txt | wc -l"
2. "[!0-9] excludes digits."
3. "Answer: 4"
Failure:
- Includes digit-starting files → "Pattern excludes digits 0-9."

### Level 10.5 – Batch File Operations
Start: `$WORKSPACE/level-10/batch/`
Task: "Copy all .txt files to ../organize/ using cp *.txt ../organize/. Count how many files are now in organize/. Submit the count."
Files to copy: keep1.txt, keep2.txt (2 files)
Answer: `2`
Validation:
- Files exist in organize/.
- Count matches.
Hints:
1. "Use: cp *.txt ../organize/"
2. "Then: ls ../organize/ | wc -l"
3. "Answer: 2"
Failure:
- Files not copied → "Did you use cp *.txt?"
- Wrong count → "Verify with ls ../organize/"

### Level 10.6 – Pattern-Based Cleanup
Start: `$WORKSPACE/level-10/batch/`
Task: "Remove all .tmp files using rm *.tmp. Count remaining files in current directory. Submit the count."
Files to remove: temp1.tmp, temp2.tmp, temp3.tmp (3 files)
Remaining: keep1.txt, keep2.txt, important.doc (3 files, or 2 if .txt files were copied)
Answer: `3` (if run before 10.5) or adjust based on state
Validation:
- No .tmp files remain.
- Count correct.
Hints:
1. "Use: rm *.tmp"
2. "Count remaining: ls | wc -l"
3. "Answer: 3"
Failure:
- .tmp files still present → "Use rm *.tmp to remove them."

### Level 10.7 – (Optional) Complex Pattern Combo
Start: `$WORKSPACE/level-10/advanced/`
Task: "List files matching [ab][12].dat (first char a or b, second char 1 or 2, extension .dat). Submit comma-separated sorted list of basenames without extensions."
Files matching: a1.dat, a2.dat, b1.dat, b2.dat
NOT matching: c1.dat, x.dat, y.dat
Answer: `a1,a2,b1,b2`
Validation:
- Ordered alphabetically.
- Extensions stripped.
- All matches included.
Hints:
1. "Use: ls [ab][12].dat"
2. "Pattern has two character classes."
3. "Answer: a1,a2,b1,b2"
Optional metadata: `optional=true`

## 7. General Validation Rules (Section 10)
- Trim whitespace.
- Integer answers: strict numeric parsing.
- File basename answers: strip extension.
- Ordered lists: comma-separated, alphabetically sorted.
- Verify wildcards expand correctly before validation.
- Count files after operations to ensure correct behavior.

## 8. Hint Strategy
Three hints per level:
1. Wildcard syntax and example.
2. Explanation of pattern matching behavior.
3. Explicit answer or verification command.
Track `attempts` and `hints_used`.

## 9. Telemetry / State Logging
Per completion:
```
"10.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 10.1–10.4 required. 10.5–10.6 Extension. 10.7 Optional.

## 10. Failure Message Templates
- Wrong count: "Count does not match—verify pattern."
- Pattern too broad: "Your pattern matched too many files."
- Pattern too narrow: "Your pattern matched too few files."
- Files not removed: "Target files still present—verify rm command."
- Files not copied: "Files not found in destination—verify cp command."
- List not ordered: "List must be alphabetically sorted."
- Extension included: "Remove extensions from basenames."

## 11. Edge Cases & Robustness
- Hidden files: `*` does NOT match files starting with dot (use `.*` explicitly).
- No matches: behavior varies (some shells pass literal pattern, others error).
- Quote protection: quoted patterns not expanded (teach: use unquoted).
- Case sensitivity: patterns are case-sensitive.
- Space in filenames: wildcards handle spaces (each match treated separately).

## 12. Implementation Checklist
- Pre-create all files with predictable names.
- Provide helpers:
  - `glob_expand(pattern, directory)` → list of matches
  - `count_matches(pattern, directory)` → int
  - `file_exists(path)` → bool
  - `strip_extension(filename)` → string
- Validate glob expansion matches expectations.
- Reset mechanism reconstructs Section 10 tree.
- Track state changes from operations (cp, mv, rm).

## 13. Advancement Criteria
After Level 10.4 success:
- `current_level = "11.1"`
Optional Level 10.7 accessible post-advancement.

## 14. Sample Instruction Screen (Level 10.3)
```
══════════════════════════════════════════════════════
LEVEL 10.3: Character Classes

Character classes let you match one character from a set.
[aeiou] matches any single vowel.

Task:
1. Navigate to level-10/classes/
2. Count .txt files starting with vowels
3. Use pattern: [aeiou]*.txt
4. Command: ls [aeiou]*.txt | wc -l
5. Submit the count

Remember: [set] matches ONE character from the set.

Submit with: shellgame submit <count>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 15. Pedagogical Reinforcement Points
- Wildcards enable batch operations without scripting.
- Pattern precision matters: too broad or narrow affects results.
- Shell expansion happens before command sees arguments.
- Character classes provide fine-grained control.
- Negation ([!...]) teaches inverse matching logic.
- Practical application: cleanup, organization, filtering at scale.

## 16. Future Cross-References
- Section 11 (find): more powerful searching beyond simple globs.
- Scripting: loops over wildcard matches.
- Regular expressions (grep): similar but more powerful pattern language.
- Advanced shells: extended globs, brace expansion.

## 17. Summary (Instructor View)
Section 10 transforms students from operating on single files to manipulating file sets with patterns. Wildcard literacy is fundamental to efficient command-line work—enabling rapid filtering, batch processing, and pattern-based organization. By mastering globs, students gain the multiplier effect essential for real-world filesystem management.

Pacing Note: Core emphasizes pattern comprehension before batch mutation.

End of Section 10 Specification.