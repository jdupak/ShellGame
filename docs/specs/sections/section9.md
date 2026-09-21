# Section 9 Specification – "Redirection & Pipes"

> **Renumbered.** This section used to be number 8; wildcards were moved ahead of
> permissions and redirection so globbing is taught before it is used. Level IDs and
> `level-8/` workspace paths below still use the old `8.x` numbering, and the
> Extension/Optional level statuses were never implemented. Treat this file as the
> original design intent, not as a description of the shipped section.

## 1. Purpose & Scope
Section 8 introduces output redirection and pipes, fundamental Unix concepts for combining commands and saving output. Building on prior sections, students learn to:
- Redirect stdout to files using `>`
- Append stdout to files using `>>`
- Connect commands using pipes (`|`)
- Use `head` and `tail` to view file portions
- Count lines, words, and characters with `wc`

Deliberate exclusions:
- No stderr redirection (covered in Section 9)
- No advanced file descriptors
- No process substitution
- No tee command (optional mention only)

Allowed commands: All previous commands plus `>`, `>>`, `|`, `head`, `tail`, `wc`
Estimated Time: 8–10 minutes (core Levels 8.1–8.5) + optional Level 8.6 (~2 minutes)

## 2. Learning Objectives
By the end of Section 8 the player will:
1. Redirect command output to a file using `>`.
2. Understand that `>` overwrites existing files.
3. Append output to existing files using `>>`.
4. View the first N lines of a file with `head`.
5. View the last N lines of a file with `tail`.
6. Count lines, words, and characters with `wc`.
7. Combine commands using pipes (`|`).
8. Build simple pipelines to filter and process data.

## 3. Concept Tutorial (Displayed Before Level 8.1)
Key concepts:
- **Standard output (stdout)**: Where commands normally print their results
- **Redirection `>`**: Send stdout to a file instead of the screen
  - `ls > files.txt` saves directory listing to files.txt
  - WARNING: Overwrites existing file content!
- **Append `>>`**: Add to end of file without overwriting
  - `echo "new line" >> log.txt`
- **Pipes `|`**: Connect output of one command to input of another
  - `cat file.txt | wc -l` counts lines in file
  - `ls | head -5` shows first 5 items
- **`head`**: Show first N lines (default 10)
  - `head -3 file.txt` shows first 3 lines
- **`tail`**: Show last N lines (default 10)
  - `tail -5 file.txt` shows last 5 lines
- **`wc`**: Word count utility
  - `wc -l` counts lines
  - `wc -w` counts words
  - `wc -c` counts characters/bytes

Visual representation:
```
Command ──── stdout ───▶ Screen (default)

Command ──── > ────────▶ file.txt (overwrite)

Command ──── >> ───────▶ file.txt (append)

Command1 ──── | ───────▶ Command2 ──── | ───▶ Command3
```

## 4. Level Breakdown

### Level 8.0 – Section Introduction
- Type: Intro (non-interactive)
- Content: Tutorial material from Section 3 above
- Validation: None (auto-advance on Enter)

### Level 8.1 – Basic Output Redirection
- Goal: Save command output to a file
- Setup: Empty workspace directory
- Task: "Save the output of `ls /` to a file called `root_contents.txt`"
- Expected: `ls / > root_contents.txt`
- Validation: File exists and contains expected content
- Hints:
  1. "Use `>` to redirect output to a file"
  2. "The syntax is: command > filename"
  3. "Try: `ls / > root_contents.txt`"

### Level 8.2 – Overwrite Warning
- Goal: Understand that `>` overwrites
- Setup: Directory with existing file `data.txt` containing "original content"
- Task: "The file `data.txt` exists. Run `echo 'new content' > data.txt`, then check what happened to the original content."
- Submit: Answer what happened (e.g., "overwritten" or "replaced")
- Validation: String match for overwrite concept
- Hints:
  1. "Use `cat data.txt` to see the file content after redirection"
  2. "Was the original content preserved or replaced?"

### Level 8.3 – Append to File
- Goal: Use `>>` to preserve existing content
- Setup: File `log.txt` with some entries
- Task: "Add a new line 'Entry 4' to the end of `log.txt` without losing existing entries"
- Expected: `echo "Entry 4" >> log.txt`
- Validation: File contains original content plus new line
- Hints:
  1. "Use `>>` instead of `>` to append"
  2. "Syntax: echo 'text' >> filename"

### Level 8.4 – Head and Tail
- Goal: View portions of files
- Setup: File `numbers.txt` with lines 1-20
- Task: "How many is the sum of the first number and last number in `numbers.txt`?"
- Expected: Use `head -1` and `tail -1` to find first and last
- Validation: Integer answer (1 + 20 = 21)
- Hints:
  1. "Use `head -1 numbers.txt` to see the first line"
  2. "Use `tail -1 numbers.txt` to see the last line"
  3. "Add the two numbers together"

### Level 8.5 – Word Count
- Goal: Use wc to count lines/words/characters
- Setup: File `article.txt` with known line count
- Task: "How many lines are in `article.txt`?"
- Expected: `wc -l article.txt`
- Validation: Integer match
- Hints:
  1. "The `wc` command counts things in files"
  2. "Use `wc -l` to count lines specifically"
  3. "Try: `wc -l article.txt`"

### Level 8.6 – Simple Pipeline
- Goal: Combine commands with pipes
- Setup: Directory with many files (15+)
- Task: "Count how many items are in the current directory using `ls` and `wc`"
- Expected: `ls | wc -l`
- Validation: Integer match
- Hints:
  1. "Pipes (`|`) connect the output of one command to the input of another"
  2. "First, `ls` lists items. Then `wc -l` counts lines."
  3. "Try: `ls | wc -l`"

### Level 8.7 – Multi-stage Pipeline (Extension)
- Goal: Build longer pipelines
- Setup: File with data to filter
- Task: "Find how many unique words start with 'a' in `words.txt`"
- Expected: `grep '^a' words.txt | wc -l` or similar
- Validation: Integer match
- Hints:
  1. "You can chain multiple pipes: cmd1 | cmd2 | cmd3"
  2. "Use `grep '^a'` to find lines starting with 'a'"
  3. "Pipe the result to `wc -l` to count"

## 5. Common Mistakes & Guardrails
- Confusing `>` (overwrite) with `>>` (append)
- Forgetting that `>` destroys existing file content
- Putting the filename before `>` instead of after
- Using `|` when `>` is needed (or vice versa)

## 6. Time Budget
- Level 8.1: ~1 minute
- Level 8.2: ~1 minute
- Level 8.3: ~1 minute
- Level 8.4: ~2 minutes
- Level 8.5: ~1 minute
- Level 8.6: ~2 minutes
- Level 8.7: ~2 minutes (extension)

**Core path (8.1–8.6): ~8 minutes**
**Full section: ~10 minutes**

## 7. Dependencies
- Requires: Sections 1–5 (navigation, file inspection, cat)
- Leads to: Section 9 (stderr and advanced redirection)