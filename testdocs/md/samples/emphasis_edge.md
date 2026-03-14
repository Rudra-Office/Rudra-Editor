# Emphasis Edge Cases

## Basic Emphasis

*italic text* and _also italic_

**bold text** and __also bold__

***bold italic*** and ___also bold italic___

## Nested Emphasis

**bold with _italic_ inside**

*italic with **bold** inside*

**bold _bold-italic_ bold again**

*italic **bold-italic** italic again*

## Strikethrough

~~deleted text~~

~~strikethrough with **bold** inside~~

**bold with ~~strike~~ inside**

## Combined

***~~bold italic strikethrough~~***

**bold** and *italic* and ~~strike~~ in one line

## Code in Emphasis

**`bold code`** and *`italic code`*

`code ignores *emphasis* and **bold**`

## Emphasis at Boundaries

**bold at start** of line

End of line is **bold**

Middle **bold** middle

A **single** word

## Edge Cases

*single asterisk emphasis*

**empty bold: ****

Not emphasis: 2*3*4 or file_name_with_underscores

Intra-word: foo*bar*baz (may or may not be emphasis per spec)

## Escapes

\*not italic\*

\*\*not bold\*\*

\~\~not strikethrough\~\~
