#!/usr/bin/env python3
from pathlib import Path
import re

if "Cargo.toml" not in [p.name for p in Path.cwd().iterdir()]:
    print("Error: this script should be run from the root of the rust project")
    exit(1)

# Load the schema and schema patch
schema = Path("src/schema.rs").read_text().splitlines()
schema_patch_file = Path("src/schema_patch.rs")

if schema_patch_file.exists():
    schema_patch = schema_patch_file.read_text().splitlines()
else:
    schema_patch = []

# If the original schema and the patch are the same, it means diesel applied the patch to the schema and nothing has changed
if schema == schema_patch:
    print("No changes in schema")
    exit(0)

# Reset the schema patch file
schema_patch: list[str] = []

for line in schema:
    # Find problematic lines
    match = re.search(r"Array<Nullable<(\w+)>>", line)

    if not match:
        # If the line is not problematic, just append it to the schema patch
        schema_patch.append(line)
        continue

    inner_dtype = match.group(1)
    fixed_line = line.replace(f"Array<Nullable<{inner_dtype}>>", f"Array<{inner_dtype}>")
    schema_patch.append(fixed_line)

schema_patch.append("\n")
schema_patch_file.write_text("\n".join(schema_patch))

print("Edit schema_patch.rs if additional changes are needed. Then, run:")
print("diff -U 6 src/schema.rs src/schema_patch.rs > src/schema.patch\n")

print("Now, add the following line to the [print_schema] section of the diesel.toml file if it's not already there:")
print("patch_file = \"src/schema.patch\"\n")

print("Now you can run migrations using diesel")
