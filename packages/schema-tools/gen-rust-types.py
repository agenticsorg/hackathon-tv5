#!/usr/bin/env python3
"""
ARW Rust Type Generator

Generates Rust types with serde support from LinkML schema.
LinkML doesn't natively generate Rust, so this is a custom generator.

Usage:
    python gen-rust-types.py schemas/arw_model.yaml > schemas/generated/types.rs
"""

import sys
import yaml
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime


class RustTypeGenerator:
    """Generates Rust types from LinkML schema"""

    def __init__(self, schema: Dict[str, Any]):
        self.schema = schema
        self.classes = schema.get("classes", {})
        self.enums = schema.get("enums", {})
        self.types = schema.get("types", {})
        self.slots = schema.get("slots", {})

    def generate(self) -> str:
        """Generate complete Rust module"""

        output = []

        # Header
        output.append(self._generate_header())

        # Imports
        output.append(self._generate_imports())

        # Type aliases
        output.append(self._generate_type_aliases())

        # Enums
        for enum_name, enum_def in self.enums.items():
            output.append(self._generate_enum(enum_name, enum_def))

        # Structs (classes)
        for class_name, class_def in self.classes.items():
            output.append(self._generate_struct(class_name, class_def))

        return "\n\n".join(output)

    def _generate_header(self) -> str:
        """Generate file header"""
        return f"""//! ARW Type Definitions
//!
//! Auto-generated from LinkML schema
//! Generated: {datetime.now().isoformat()}
//! Source: schemas/arw_model.yaml
//!
//! DO NOT EDIT MANUALLY - regenerate with:
//!   python tools/schema-tools/gen-rust-types.py schemas/arw_model.yaml

#![allow(dead_code, non_snake_case)]"""

    def _generate_imports(self) -> str:
        """Generate imports"""
        return """use serde::{Deserialize, Serialize};
use std::collections::HashMap;"""

    def _generate_type_aliases(self) -> str:
        """Generate type aliases for custom types"""

        output = ["// Type Aliases"]

        for type_name, type_def in self.types.items():
            if type_name == "url":
                output.append("pub type Url = String; // TODO: Use url::Url")
            elif type_name == "email":
                output.append("pub type Email = String;")

        return "\n".join(output)

    def _generate_enum(self, enum_name: str, enum_def: Dict) -> str:
        """Generate Rust enum"""

        values = enum_def.get("permissible_values", {})

        lines = [
            f"/// {enum_def.get('description', enum_name)}",
            "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]",
            f"pub enum {enum_name} {{",
        ]

        for value in values.keys():
            # Convert to PascalCase if needed
            variant_name = self._to_pascal_case(value)
            lines.append(f'    #[serde(rename = "{value}")]')
            lines.append(f"    {variant_name},")

        lines.append("}")

        return "\n".join(lines)

    def _generate_struct(self, class_name: str, class_def: Dict) -> str:
        """Generate Rust struct"""

        description = class_def.get("description", class_name)
        attributes = class_def.get("attributes", {})
        slots_list = class_def.get("slots", [])

        lines = [
            f"/// {description}",
            "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]",
            f"pub struct {class_name} {{",
        ]

        # Generate fields
        for slot_name in slots_list:
            slot_def = attributes.get(slot_name, {})
            if not slot_def:
                slot_def = self.slots.get(slot_name, {})

            field_lines = self._generate_field(slot_name, slot_def)
            lines.extend(field_lines)

        lines.append("}")

        return "\n".join(lines)

    def _generate_field(self, slot_name: str, slot_def: Dict) -> List[str]:
        """Generate struct field"""

        lines = []

        # Doc comment
        description = slot_def.get("description", "")
        if description:
            lines.append(f"    /// {description}")

        # Required/optional
        required = slot_def.get("required", False)
        multivalued = slot_def.get("multivalued", False)

        # Type mapping
        rust_type = self._map_type(slot_def, multivalued)

        # Serde attributes
        if slot_name != self._to_snake_case(slot_name):
            lines.append(f'    #[serde(rename = "{slot_name}")]')

        if not required:
            lines.append('    #[serde(skip_serializing_if = "Option::is_none")]')
            rust_type = f"Option<{rust_type}>"

        # Field declaration
        field_name = self._to_snake_case(slot_name)
        lines.append(f"    pub {field_name}: {rust_type},")

        return lines

    def _map_type(self, slot_def: Dict, multivalued: bool) -> str:
        """Map LinkML type to Rust type"""

        range_type = slot_def.get("range", "string")

        # Map to Rust types
        type_map = {
            "string": "String",
            "integer": "i64",
            "float": "f64",
            "double": "f64",
            "boolean": "bool",
            "date": "String",  # TODO: Use chrono::NaiveDate
            "datetime": "String",  # TODO: Use chrono::DateTime<Utc>
            "uri": "Url",
            "url": "Url",
            "email": "Email",
        }

        rust_type = type_map.get(range_type, range_type)

        # Handle enums and classes (PascalCase)
        if range_type in self.enums or range_type in self.classes:
            rust_type = range_type

        # Handle multivalued (arrays)
        if multivalued:
            rust_type = f"Vec<{rust_type}>"

        return rust_type

    def _to_snake_case(self, name: str) -> str:
        """Convert to snake_case"""
        import re

        s1 = re.sub("(.)([A-Z][a-z]+)", r"\1_\2", name)
        return re.sub("([a-z0-9])([A-Z])", r"\1_\2", s1).lower()

    def _to_pascal_case(self, name: str) -> str:
        """Convert to PascalCase"""
        return "".join(word.capitalize() for word in name.replace("-", "_").split("_"))


def main():
    if len(sys.argv) < 2:
        print("Usage: gen-rust-types.py <schema.yaml>", file=sys.stderr)
        sys.exit(1)

    schema_path = Path(sys.argv[1])

    if not schema_path.exists():
        print(f"Error: Schema file not found: {schema_path}", file=sys.stderr)
        sys.exit(1)

    with open(schema_path) as f:
        schema = yaml.safe_load(f)

    generator = RustTypeGenerator(schema)
    rust_code = generator.generate()

    print(rust_code)


if __name__ == "__main__":
    main()
