# Section 5 Specification – "Zkoumání souborů" (aktuální implementace)

## 1. Purpose & Scope
Section 5 introduces inspection of file metadata and type identification, building on navigation and basic file interaction skills from Sections 1–4. Focus areas:
- Understanding `ls -l` output format (permissions, size, modification time)
- Reading file sizes and identifying files by size
- Using the `file` command to identify file types beyond extensions
- Distinguishing text from binary files
- Recognizing executables and scripts by metadata

Deliberate exclusions:
- No file copying (`cp`) or moving (`mv`) — deferred to Section 6
- No permission modification (`chmod`) — covered in Section 7
- No advanced time formats or sorting
- No ownership changes
- No pattern matching or wildcards yet

Allowed commands (typicky): `pwd`, `ls`, `ls -l`, `ls -lh`, `cd`, `cat`, `file`, `less`

Estimated Time: ~8–15 minutes (podle toho, zda student dělá rozšiřující/volitelné levely).

Poznámka: Tento dokument popisuje aktuální implementaci v `src/shellgame/levels/sections/section5.py`.

## 2. Learning Objectives
By the end of Section 5 the player will:
1. Parse `ls -l` output to extract: permissions string, size in bytes, and filename.
2. Identify a specific file by its exact byte size.
3. Use the `file` command to determine the actual file type, regardless of its extension.
4. Distinguish between text files, scripts, executables, and data files.
5. Locate an executable file by recognizing the `x` permission bit.
6. Find a "disguised" file (e.g., a script with a misleading extension).
7. (Optional) Count files within a specific size range using manual inspection.

## 3. Concept Tutorial (Displayed Before Level 5.1)
Key concepts:
- `ls -l` displays "long format" with columns:
  - Column 1: permissions (10 characters: type + owner/group/other)
  - Column 5: size in bytes
  - Final column: filename
- File size is measured in bytes.
- `file <name>` inspects file content to determine its type (it ignores the extension).
- Executable permission: look for an `x` in the permissions string.
- Extensions can lie; `file` reveals the truth.

Example `ls -l` output:
```
-rw-r--r-- 1 user group  1024 Nov 14 10:23 data.txt
-rwxr-xr-x 1 user group  4096 Nov 13 09:15 script.sh
drwxr-xr-x 2 user group  4096 Nov 12 14:30 folder
```

Short prompt:
"Learn to read file metadata. Size, type, and permissions tell the full story—extensions are just hints."

## 4. Directory Layout (aktuální)
Base: `$WORKSPACE/level-5/`

Aktuální struktura (zjednodušeně):
```
level-5/
├── sizes/
│   └── database.db        (12345 bytes)
├── search/
│   ├── file_a             (1000 bytes)
│   ├── file_b             (2000 bytes)
│   ├── file_c             (1338 bytes)
│   └── target_file        (1337 bytes)
├── types/
│   ├── file1              (text)
│   ├── file2              (binary)
│   └── file3              (JPEG header)
├── logs/
│   └── server.log         (200 lines; one has CRITICAL ... Code 42)
├── downloads/             (Extension)
│   ├── photo1.jpg         (JPEG header)
│   ├── photo2.jpg         (JPEG header)
│   └── secret.jpg         (text)
├── bin/                   (Optional)
│   ├── run.sh
│   ├── readme.txt
│   ├── calc.py            (python script)
│   └── program            (ELF header)
└── mystery/               (Summary)
  ├── data.bin           (text)
  ├── notes.xyz          (text)
  ├── config.txt         (PNG header)
  ├── analyzer.dat       (python script)
  └── readme.doc         (ELF header)
```

## 5. Level Index (aktuální)

Legenda:
- **Core** = nutné pro postup
- **Extension** = rozšiřující (není nutné)
- **Optional** = volitelné

| ID  | Title | Focus | Answer Type | Notes |
|-----|-------|-------|-------------|-------|
| 5.0 | Intro | čtení úvodu | — | auto-pass |
| 5.1 | Velikost souboru | `ls -l` a velikost v bajtech | integer | Core |
| 5.2 | Hledání podle velikosti | najít soubor s 1337 bajty | filename | Core |
| 5.3 | Typ souboru | `file` a "přípona nelže" | filename | Core |
| 5.4 | less + vyhledávání | práce s velkým logem | integer | Core |
| 5.5 | Zamaskovaný soubor | falešné `.jpg` | filename | Extension |
| 5.6 | Python skript | rozpoznání skriptu v `bin/` | filename | Optional |
| 5.7 | Souhrn Sekce 5 | kombinace `file *` a počítání typů | `text,img,script` | Core |

## 6. Detailed Level Specifications

### Level 5.1 – Velikost souboru
Start: `$WORKSPACE/level-5/sizes/`
Task: "Zjistěte přesnou velikost souboru `database.db` v bajtech."
Answer: integer (např. `12345`)
Hints:
1. Připomenutí `ls -l`.
2. "velikost je pátý sloupec".
3. "použijte `ls -l database.db`".

### Level 5.2 – Hledání podle velikosti
Start: `$WORKSPACE/level-5/search/`
Task: "Najděte soubor, který má přesně 1337 bajtů."\
Answer: filename (např. `target_file`)
Hints:
1. "Use `ls -l` to check sizes."
2. "Look for `1337` in the size column."
3. "The answer is `large`."

### Level 5.3 – Typ souboru
Start: `$WORKSPACE/level-5/types/`
Task: "Ze tří souborů bez přípony zjistěte, který je JPEG (pomocí `file`)."\
Answer: filename (např. `file3`)
Hints:
1. "Run: `file image.jpg`"
2. "Read the output; take the first word after the colon."
3. "The answer is `ASCII`."

### Level 5.4 – less (log)
Start: `$WORKSPACE/level-5/logs/`
Task: "Najděte v `server.log` řádek s `CRITICAL` a odevzdejte číslo na konci."\
Answer: integer (např. `42`)
Hints:
1. "Run `file` on each file in the directory."
2. "Look for 'shell script' or 'bash' in the output for a `.txt` file."
3. "The answer is `notes`."

### Level 5.5 – Zamaskovaný soubor (Extension)
Start: `$WORKSPACE/level-5/downloads/`
Task: "Najděte `.jpg`, který je ve skutečnosti text (ASCII text) pomocí `file *.jpg`."\
Answer: filename (např. `secret.jpg`)

### Level 5.6 – Python skript (Optional)
Start: `$WORKSPACE/level-5/bin/`
Task: "Najděte soubor, který je Python skript (pomocí `file *`)."\
Answer: filename (např. `calc.py`)
Hints:
1. "Use `ls -l` and look at the first column of 10 characters."
2. "Executable files have an 'x' in their permissions string, like `-rwx------`."
3. "The answer is `mystery`."

## 7. General Validation Rules
- Trim whitespace from answers.
- File basename answers should not include the extension.
- Single word answers (file types) are case-insensitive.
- Size validation requires an exact byte match.

## 8. Hint Strategy
Three hints per level:
1. Command introduction / reminder.
2. Column or output interpretation guidance.
3. Near-answer or explicit answer.

## 9. Telemetry / State Logging
Per completion:
```
"5.n": {
  "time_sec": <int>,
  "attempts": <int>,
  "hints": <int>
}
```
Advancement: Levels 5.1–5.3 required. 5.4 Extension. 5.5 Optional.

## 10. Failure Message Templates (doporučení)
- "Odpověď musí být číslo." / "Očekáváno celé číslo"
- "Zadejte název souboru (bez cesty)."
- "Zkontrolujte výstup `ls -l` / `file` a zkuste to znovu."

## 11. Implementation Checklist
- Pre-create all files with exact byte sizes and content.
- Ensure the `file` command is available.
- Implement helper functions for getting file size, type, and permissions.
- Implement a reset mechanism for the Section 5 directory tree.

## 12. Advancement Criteria
Aktuální implementace postupuje lineárně podle registry (včetně Extension/Optional dle toho, jak je registry interpretuje).

Poznámka: Pokud chceme, aby Extension/Optional nikdy neblokovaly postup, musí to vynutit logika v `levels/registry.py` (mimo rozsah tohoto dokumentu).

## 13. Sample Instruction Screen (Level 5.3)
```
══════════════════════════════════════════════════════
LEVEL 5.3: File Type Detection

Extensions can be misleading. The 'file' command inspects
actual content to determine the true file type.

Task:
1. Navigate to `level-5/types/`
2. Run: `file image.jpg`
3. Read the output after the colon.
4. Submit the FIRST word of the type description.

Example output: "image.jpg: ASCII text"
You would submit: ASCII

Submit with: shellgame submit <type-word>
Need help? Type: shellgame hint
══════════════════════════════════════════════════════
```

## 14. Pedagogical Reinforcement Points
- Emphasizes metadata over assumptions (extension ≠ type).
- Introduces structured output parsing (`ls -l` columns).
- Reinforces byte-level size awareness.
- Executable recognition prepares for the permissions section.
- The `file` command demystifies the binary vs. text distinction.

## 15. Future Cross-References
- **Section 6 (Copying & Moving)**: Will use these inspection skills before acting on files.
- **Section 7 (Permissions)**: Builds directly on executable recognition.
- **Section 8 (Content & Redirection)**: Will create files and verify their sizes.
- **Section 9 (Wildcards)**: Will filter files by size/type patterns.

## 16. Summary (Instructor View)
Section 5 transitions students from spatial navigation to attribute-aware inspection. Students learn to interrogate files beyond their names, establishing a diagnostic mindset essential for system administration and debugging. Metadata literacy enables confident manipulation in subsequent sections.

Time Note: Focus early on size + type (5.1–5.3); disguises and executables enrich after mastery.

End of Section 5 Specification.