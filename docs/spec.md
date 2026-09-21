# ShellGame – Complete Design & Level Specification

## 0. Purpose
ShellGame is a local, single-binary, Linux terminal learning experience for first-semester CS students. It focuses on interactive mastery of filesystem navigation and basic shell operations—especially fluent use of `cd`, path reasoning (absolute vs relative), and fundamental file manipulation. The design targets:
- Average completion time: ~75 minutes
- Slow path (essential core): ~20 minutes (Sections 1–3)
- Optional advanced / bonus: up to ~90 minutes

---

## 1. High-Level Features

### 1.1 Deployment
- Single static binary named `shellgame` in `PATH`
- User state stored at: `~/.config/shellgame/state.json`
- Workspace per user at: `/tmp/shellgame-$USER/`
- Temporary directories automatically cleared by system on reboot (no cleanup service required)

### 1.2 Core Commands
| Command | Purpose |
|---------|---------|
| `shellgame init` | Compatibility command; initialization normally happens automatically |
| `shellgame` | Show current level instructions (or tutorial splash) |
| `shellgame hint` | Show progressive hints (max 3 per level) |
| `shellgame submit ANSWER` | Submit an answer derived from navigation / file inspection / creation |
| `shellgame submit` | For state-based validations (location, existence of files) |
| `shellgame reset` | Rebuild current level's directory structure |
| `shellgame remove` | Delete state and workspace |
| `shellgame status` | Show progress, time per level, hints used |
| `shellgame hint --repeat` | Reprint already revealed hints without consuming a new one |
| `shellgame skip` | Advance past a bonus (optional or extension) level |
| `shellgame repeat` | Re-show a level or a section intro |
| `shellgame show --level` / `--section` | Re-show the current level or the current section intro |
| `shellgame exit` | Leave the ShellGame subshell |

### 1.3 Game Philosophy
Answers (formerly “flags”) are natural discoveries: directory names, file contents, counts, sizes—never artificial tokens. Reinforces authentic terminal literacy.

---

## 2. State & Data

### 2.1 State File Structure (example)
```json
{
  "username": "jdupak-ms",
  "workspace": "/tmp/shellgame-jdupak-ms",
  "current_level": "2.3",
  "start_time": "2025-11-14T09:00:00Z",
  "levels_complete": {
    "1.1": {"time_sec": 45, "hints": 0, "attempts": 1},
    "1.2": {"time_sec": 60, "hints": 1, "attempts": 2}
  }
}
```

### 2.2 Workspace Example Layout
```
$SHELLGAME_WORKSPACE/          (default: /tmp/shellgame-$USER)
├── level-1/                   section root for section 1
│   ├── alpha/
│   ├── gamma/deep/a/b/c/
│   └── maze/
├── level-2/
├── ...
└── level-11/
```

One directory per section, named `level-<N>`, declared once as the section root.
Every path a level declares is relative to that directory - see
[AUTHORING.md](AUTHORING.md).

---

## 3. Tutorial & Pedagogy

Each section starts with:
- Concept explanation
- Mental model (tree, addresses, relative vs absolute)
- Commands introduced (with short syntax examples)
- Then sublevels for practice
- Hints escalate: orientation → specific command → near-answer clue

---

## 4. Navigation Bootcamp (Deep Focus on `cd`)

This bootcamp can exist as:
- A dedicated early mega-section OR
- Interwoven as expanded Section 1 + part of Section 2

### Learning Outcomes
Students can:
- Identify current directory (`pwd`)
- Move with absolute paths from anywhere
- Traverse using relative chains (`../..`, siblings)
- Reset via `~` (home) or `/` (root)
- Use `cd -` to toggle
- Understand `.` vs `..`
- Build complex relative path sequences consciously

### Sublevels

| ID  | Focus | Task Summary | Expected Answer Type |
|-----|-------|--------------|----------------------|
| 1.1 | `pwd` | Report last directory name | `intro` |
| 1.2 | Home | Longest hidden dir in `~` | e.g. `.config` |
| 1.3 | Root | Count entries in `/` | Integer (e.g. `13`) |
| 1.4 | Absolute | Reach `/tmp/.../absolute/alpha/` and read file | `sample.txt` |
| 1.5 | Up one | `cd ..` basename after move | `absolute` |
| 1.6 | Multi-up | From deep chain go up 3 levels | `r1` |
| 1.7 | Sibling | Move to `notes/` via relative path | File inside (e.g. `reminder.md`) |
| 1.8 | Dot | `cd .` effect? yes/no | `no` |
| 1.9 | Chain | Compose `../../nextstage/step1` path | `complete.done` |
| 1.10 | Abs vs Rel | Contrast `cd data` vs absolute path | `alpha,data` |
| 1.11 | Home + Root | First letters of dirs reached | `h,r` |
| 1.12 | `cd -` | Toggle prior directory | `maze` |
| 1.13 | Maze | Follow sequence to endpoint | basename (e.g. `c`) |
| 1.14 | Tab (optional) | Long directory name; give 5th char | single character |
| Walk Root→Home | Drill | Stepwise journey `/` → `/home` → `/home/$USER` | Sequence or final path |
| Reverse Walk | Drill | From home up to `/`, then back down | Path articulation |
| Breadcrumb Reflection | Meta | List all dirs traversed | Ordered list |

---

## 5. Full Section Breakdown (Updated & Re-ordered)

> **Stale.** This breakdown is the original design intent and has drifted from the
> shipped game (it still lists bonus sections and extension levels that were never
> built). The shipped order is Navigation, Files, Hidden, Create/Delete, Inspect,
> Copy/Move, **Wildcards**, **Permissions**, **Redirection & Pipes**, **Error
> streams**, Search. The authoritative source is
> `src/shellgame/levels/sections/`; `docs/AUTHORING.md` describes how to change it.

### Section 1: Where Am I? (7 min)
Tools: `pwd`, `ls`, `cd`, `cd ~`
Sublevels: Find current directory, list contents, enter a directory, go home, use an absolute path.

### Section 2: Moving Around (7 min)
Tools: `cd ..`, `cd -`, relative chains
Sublevels: Parent directory recognition, multi-level ascent, **previous directory toggling with `cd -`**, relative maze navigation.

### Section 3: Hidden and Visible (6 min)
Tools: `ls -a`, `cat`
Sublevels: Find hidden files, read hidden files, count hidden directories.

### Section 4: Creating Your World (12 min)
Tools: `touch`, `mkdir`, `mkdir -p`, `rm`, `rmdir`
Sublevels: Create files and directories, create nested structures, remove files, attempt to remove non-empty directories.

### Section 5: File Detective (8 min)
Tools: `ls -l`, `file`
Sublevels: Identify files by size, detect file types, find disguised files.

### Section 6: Copying & Moving (10 min)
Tools: `cp`, `cp -r`, `mv`
Sublevels: Copy files, copy directories recursively, rename files, move files between directories.

### Section 7: Permissions Matter (7 min)
Tools: `ls -l`, `chmod`
Sublevels: Find executable files, make a script executable, remove write permissions, apply numeric modes.

### Section 8: Content Control & Redirection (10 min)
Tools: `echo >`, `>>`, `head`, `tail`, `|`, `grep`, `wc`, `sort`
Sublevels: Create files with content, append to files, view first/last lines, pipe commands, filter with `grep`, count with `wc`.

### Section 9: Wildcards & Patterns (8 min)
Tools: `*`, `?`, `[]`
Sublevels: Use wildcards to count, copy, move, and remove multiple files at once.

### Bonus Section 10: Search & Destroy (10 min)
Tools: `find`, `grep -r`, `grep -i`
Sublevels: Find files by name/type/size, search content recursively, use `-exec`.

### Bonus Section 11: Tab Completion Master (5 min)
Tools: Tab completion (implicit)
Sublevels: Character extraction from long names, handle ambiguous prefixes.

---

## 6. Answer Style Guidelines

Answer Types:
1. Directory basename (`alpha`, `workspace`)
2. File basename without extension (`roadmap`, `important`)
3. Single word from file content (`Tuesday`, `gold`)
4. Integer counts (`5`, `1337`, `30`)
5. Permission fragment (`rwx`, `r-x`)
6. Character extraction (`l`, `e`)
7. Ordered pair/list (`alpha,data`, `h,r`)

Validation Considerations:
- Trim whitespace
- Case-insensitive for content words, case-sensitive for filenames
- Numeric answers validated as integers
- Provide gentle mismatch feedback.

---

## 7. Hint System

Example Hint Ladder (Level 2.2 Multi-Up):
1. “You can go up one level with `cd ..`.”
2. “Chain them: `cd ../../..` moves up three at once.”
3. “From your current depth, `cd ../../..` should land you in `r1`.”

---

## 8. Example Instruction Screen

```
══════════════════════════════════════════════════════
LEVEL 2.4: Relative Maze Navigation

You are inside the maze directory. Each room uses compass
direction names. Your goal: reach the room containing the file
treasure.dat and read its first word.

Tools you need:
- cd (relative navigation only)
- ls (see available directions)
- cat (read the file when found)

Task:
1. Explore using ls
2. Move between rooms using cd <direction>
3. Locate treasure.dat
4. Read its first word and submit that word.

Submit with: shellgame submit <word>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

---

## 9. Directory Structure Examples for Navigation Tasks

Navigation levels build their own structure from a declarative
`WorkspaceFixture`, so the layout lives next to the level rather than in this
document. Section 1 is the reference example:

```
level-1/
├── alpha/                 1.3  enter a directory, read what is inside
├── delta/                 1.2  pattern matching with `ls`
├── gamma/deep/a/b/c/      1.6  multi-level ascent with one `cd ../../..`
├── absolute-target/       1.8  jump by absolute path
└── maze/00 .. maze/04     1.7  follow the GO_* markers to 04/final
```

---

## 10. Progress & Feedback Examples

### Success
```
✅ Correct!
You navigated from a deep path up three levels efficiently.
Key idea: Chaining ../ segments accelerates movement.
Time: 62s | Hints used: 1
```

### Failure
```
✗ Not quite.
You submitted: 'absolte'
No directory with that exact name.
Check spelling and verify with: ls
Try again or use: shellgame hint
```

---

## 11. Future Extensions (Not in Current Lab)
- Advanced redirection (`2>`, `&>`, `/dev/null`)
- Editing tools (nano/vim) — intentionally excluded now
- Environment awareness (`$HOME`, `$PWD`) — conceptual only
- Scripting mode (loops, variables) — out of scope

---

## 13. Summary Table (Condensed & Updated View)
<!-- REVISED TABLE (Aligned to ~60 min average path) -->
| Section | Focus | Avg Time (Core) | Core Levels (slow path) | Extension / Optional |
|---------|-------|-----------------|-------------------------|----------------------|
| 1 | Basic Location & Absolute | 10m | 1.1–1.7 | 1.8–1.10 / 1.11 |
| 2 | Relative Mastery + Simple Read | 7m | 2.1–2.4 | 2.5 (Ext) / 2.6 |
| 3 | Hidden Basics | 5m | 3.1–3.2 | 3.3 (Ext) / 3.4 |
| 4 | Creation & Cleanup | 9m | 4.1–4.5 | 4.6 (Ext) / 4.7 |
| 5 | File Inspection | 6m | 5.1–5.3 | 5.4 (Ext) / 5.5 |
| 6 | Copy vs Move | 8m | 6.1–6.4 | 6.5 (Ext) / 6.6 |
| 7 | Permissions Basics | 6m | 7.1–7.2,7.4 | 7.3 (Ext) |
| 8 | (Future Content & Redirection) | (TBD) | - | - |
| 9 | Stderr & Redirection | 7m | 9.1–9.4 | 9.5 (Ext) / 9.6 |
| 10 | Wildcards | 7m | 10.1–10.4 | 10.5–10.6 (Ext) / 10.7 |
| 11 | Search & Discovery | 9m | 11.1–11.4 | 11.5–11.6 (Ext) / 11.7 |
> Advancement Logic: Progression now triggers after each core block; extensions never gate advancement.