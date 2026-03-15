# GFM Table Edge Cases

## Basic Table with Alignment

| Left-aligned | Center-aligned | Right-aligned | Default |
|:-------------|:--------------:|--------------:|---------|
| Left 1       | Center 1       | Right 1       | Default |
| Left 2       | Center 2       | Right 2       | Default |

## Table with Special Characters

| Feature | Symbol | Code | Notes |
|---------|--------|------|-------|
| Pipe literal | use `\|` | `\|` | Escaped pipe in cell |
| Bold text | **bold** | `**bold**` | Formatting in cells |
| Code span | `code` | `` `code` `` | Inline code in cells |
| Link | [link](url) | `[link](url)` | Hyperlinks in cells |
| Emoji | :rocket: 🚀 | `:rocket:` | Unicode emoji |

## Table with Empty Cells

| Col A | Col B | Col C |
|-------|-------|-------|
| data  |       | data  |
|       | data  |       |
| data  | data  | data  |

## Single Column Table

| Header |
|--------|
| Row 1  |
| Row 2  |
| Row 3  |

## Wide Table

| A | B | C | D | E | F | G | H | I | J |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |

## Table with Long Content

| Short | Very Long Cell Content That Exceeds Typical Column Width |
|-------|----------------------------------------------------------|
| ok    | This cell contains a much longer piece of text that tests how the parser handles wide table cells with varying content lengths. |
