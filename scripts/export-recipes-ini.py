#!/usr/bin/env python3
"""Export recipes from Llama Recipe Manager SQLite database to llama-server presets INI format."""

import os
import shlex
import sqlite3
import sys
from pathlib import Path

DEFAULT_DB_PATH = Path.home() / ".local/share/com.ashar.llama-recipe-manager/recipes.db"


def is_value_token(token: str) -> bool:
    if not token.startswith("-"):
        return True
    after_minus = token[1:]
    return bool(after_minus and after_minus[0].isdigit())


def format_recipe(recipe: dict) -> str:
    lines = []
    lines.append(f"[{recipe['name']}]")
    lines.append(f"model = {recipe['model_path']}")
    if recipe.get("mmproj_path"):
        lines.append(f"mmproj = {recipe['mmproj_path']}")
    if recipe.get("description"):
        lines.append(f"; description: {recipe['description'].replace(chr(10), ' ')}")
    if recipe.get("tags"):
        lines.append(f"; tags: {recipe['tags']}")
    if recipe.get("gpu_info"):
        lines.append(f"; gpu: {recipe['gpu_info']}")

    cmd = recipe.get("command", "").strip()
    try:
        tokens = shlex.split(cmd)
    except Exception as e:
        tokens = cmd.split()

    if tokens and (tokens[0] == "llama-server" or tokens[0].endswith("/llama-server")):
        tokens = tokens[1:]

    alias_set = False
    i = 0
    while i < len(tokens):
        arg = tokens[i]
        if arg.startswith("--") or arg.startswith("-"):
            rest = arg[2:] if arg.startswith("--") else arg[1:]
            if "=" in rest:
                k, v = rest.split("=", 1)
                if k in ("m", "model", "mmproj"):
                    pass
                else:
                    if k in ("a", "alias"):
                        alias_set = True
                        lines.append(f"alias = {v}")
                    else:
                        lines.append(f"{k} = {v}")
                i += 1
            else:
                k = rest
                if k in ("m", "model", "mmproj"):
                    if i + 1 < len(tokens) and is_value_token(tokens[i + 1]):
                        i += 2
                    else:
                        i += 1
                elif i + 1 < len(tokens) and is_value_token(tokens[i + 1]):
                    v = tokens[i + 1]
                    if k in ("a", "alias"):
                        alias_set = True
                        lines.append(f"alias = {v}")
                    else:
                        lines.append(f"{k} = {v}")
                    i += 2
                else:
                    lines.append(f"{k} = true")
                    i += 1
        else:
            i += 1

    if not alias_set:
        lines.append(f"alias = {recipe['name']}")

    return "\n".join(lines)


def export_all(db_path: Path, out_path: Path):
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    rows = conn.execute("SELECT * FROM recipes ORDER BY updated_at DESC").fetchall()
    conn.close()

    header = [
        "; Auto-generated llama.cpp models configuration file",
        "; Exported from Llama Recipe Manager",
        "; For usage with: llama-server --models-preset <file.ini>\n",
        "# Shared global defaults for all models",
        "[*]",
        "n-gpu-layers = -1",
        "jinja = true",
        "flash-attn = on\n",
    ]

    sections = [format_recipe(dict(r)) for r in rows]
    content = "\n".join(header) + "\n\n".join(sections) + "\n"

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(content, encoding="utf-8")
    print(f"Exported {len(rows)} recipes to {out_path}")


if __name__ == "__main__":
    db = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_DB_PATH
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("llama_models.ini")
    export_all(db, out)
