# Code Block Edge Cases

## Fenced Code Blocks

### Rust

```rust
use std::collections::HashMap;

fn main() {
    let mut map: HashMap<&str, Vec<i32>> = HashMap::new();
    map.insert("key", vec![1, 2, 3]);
    
    // Generic function with lifetime
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }
    
    println!("Result: {}", longest("hello", "world!"));
}
```

### Python

```python
import asyncio
from dataclasses import dataclass
from typing import Optional

@dataclass
class Document:
    """A document with metadata."""
    title: str
    content: str
    author: Optional[str] = None
    
    def word_count(self) -> int:
        return len(self.content.split())

async def process_documents(docs: list[Document]) -> None:
    for doc in docs:
        await asyncio.sleep(0.1)
        print(f"{doc.title}: {doc.word_count()} words")
```

### JavaScript

```javascript
class DocumentEngine {
  #model;
  
  constructor(format = 'docx') {
    this.#model = new Map();
    this.format = format;
  }
  
  async load(url) {
    const response = await fetch(url);
    const buffer = await response.arrayBuffer();
    return this.parse(new Uint8Array(buffer));
  }
  
  parse(bytes) {
    // Template literal with expression
    console.log(`Parsing ${bytes.length} bytes as ${this.format}`);
    return { nodes: [], styles: {} };
  }
}
```

### Shell

```bash
#!/bin/bash
set -euo pipefail

for file in *.docx; do
    echo "Processing: $file"
    s1engine convert "$file" --to odt --output "${file%.docx}.odt"
done
```

## Code Block with No Language

```
This is a plain code block.
No syntax highlighting should be applied.
It preserves    spacing    and
line breaks exactly as written.
```

## Code Block with Backtick Content

````markdown
You can use triple backticks inside a code block:

```rust
fn main() {}
```

As long as the outer fence uses more backticks.
````

## Inline Code

Regular text with `inline code` in the middle.

Inline code with special chars: `<div class="test">`, `a && b || c`, `$variable`.

Backtick in inline code: `` `backtick` `` and ``` ``double`` ```.

## Indented Code Block

    This is an indented code block.
    It uses 4 spaces of indentation.
    No language annotation is possible.
    
    fn indented_example() {
        // This style is less common but valid.
    }
