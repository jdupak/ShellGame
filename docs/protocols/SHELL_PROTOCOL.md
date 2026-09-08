# Shell Command Protocol

ShellGame uses a small line protocol to update the wrapped shell after a Python
command exits. Protocol directives are written to stderr; normal stderr is
preserved and shown to the player.

## Format

```text
__SHELLGAME_EXEC__v1 <verb> [base64-argument...]
```

Arguments are UTF-8 strings encoded with standard base64. This avoids shell
word splitting and preserves spaces and shell metacharacters without `eval`.

| Verb | Arguments | Effect |
| --- | --- | --- |
| `cd` | destination | Change the wrapper's directory with `builtin cd` |
| `export` | variable name, value | Set and export one environment variable |
| `echo` | message | Print one message |
| `pwd` | none | Print the wrapper's working directory |
| `exit` | none | Exit the wrapped subshell |

Unknown versions, verbs, or argument counts are ignored. Environment variable
names must match `[A-Za-z_][A-Za-z0-9_]*`.

## Python side

`src/shellgame/shell/client.py` validates verbs, encodes each argument, and
emits the directive. Callers use `ShellClient`; they do not build protocol
lines manually.

## Shell side

The bash and fish templates:

1. capture command stderr in a temporary file,
2. recognize only lines beginning with `__SHELLGAME_EXEC__`,
3. split the protocol header and encoded tokens,
4. decode arguments,
5. dispatch a fixed verb implementation,
6. print all non-protocol stderr unchanged.

There is no arbitrary-command fallback.

## Wrapper launch

### Bash

```text
bash --rcfile <integration_script> -i
```

`--norc` must not be used because it disables the selected rcfile.

### Fish

```text
fish --init-command "function fish_greeting; end; source <integration_script>"
```

Autostart remains at the end of each template after the `shellgame` function
has been defined.

## Remove handling

`shellgame remove` is handled specially by the wrapper. After successful
removal it exits directly, avoiding cwd errors when the workspace being
removed is the current directory.

## Extending the protocol

Adding a verb expands the shell-facing security boundary. New verbs must:

- have fixed arity,
- validate arguments in Python and shell code,
- avoid generic evaluation,
- be implemented for both bash and fish,
- include runtime parity tests.
