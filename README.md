# Use compile_commands.json and compile_flags.txt in Rust Programs

## Goal

Provide a thin wrapper type around the [compile_commands.json](https://clang.llvm.org/docs/JSONCompilationDatabase.html#format)
and  [compile_flags.txt](https://clang.llvm.org/docs/JSONCompilationDatabase.html#alternatives)
standards as provided by the LLVM project.

## Sample usage

Given the following `compile_commands.json` file:

```json
[
  { "directory": "/home/user/llvm/build",
    "arguments": ["/usr/bin/clang++", "-Irelative", "-DSOMEDEF=With spaces, quotes and \\-es.", "-c", "-o", "file.o", "file.cc"],
    "file": "file.cc" },

  { "directory": "/home/user/llvm/build",
    "command": "/usr/bin/clang++ -Irelative -DSOMEDEF=\"With spaces, quotes and \\-es.\" -c -o file.o file.cc",
    "file": "file2.cc" }
]

```

Or the following `compile_flags.txt` file:

```
-xc++
-I
libwidget/include/
```

Parse it and use as a type-safe object in your Rust project:

```rust
use std::path::PathBuf;

use compile_commands::CompilationDatabase;

fn main() {
    // Create a `CompilationDatabase` object directly from a compile_commands.json file
    let comp_cmds = include_str!("compile_commands.json");
    let comp_data = serde_json::from_str::<CompilationDatabase>(&comp_cmds).unwrap();
    _ = comp_data;

    // Or create a `CompilationDatabase` object from a compile_flags.txt file
    let comp_flags = include_str!("compile_flags.txt");
    let comp_data =
        compile_commands::from_compile_flags_txt(&PathBuf::from("~/foo/build"), &comp_flags);
    _ = comp_data;
}
```

## Usage in the Wild

### [asm-lsp](https://github.com/bergercookie/asm-lsp)

Used to provide inline error diagnostics and additional per-project include directories
