# Nested List Edge Cases

## Unordered Lists (4 levels deep)

- Level 1 item A
  - Level 2 item A.1
    - Level 3 item A.1.1
      - Level 4 item A.1.1.1
      - Level 4 item A.1.1.2
    - Level 3 item A.1.2
  - Level 2 item A.2
- Level 1 item B
  - Level 2 item B.1
    - Level 3 item B.1.1
      - Level 4 item B.1.1.1
        - Level 5 item B.1.1.1.1

## Ordered Lists (4 levels deep)

1. First item
   1. Sub-item one
      1. Sub-sub-item one
         1. Deep item one
         2. Deep item two
      2. Sub-sub-item two
   2. Sub-item two
2. Second item
   1. Sub-item one
   2. Sub-item two

## Mixed Ordered and Unordered

1. Ordered first
   - Unordered child
     1. Ordered grandchild
       - Unordered great-grandchild
         1. Ordered leaf
   - Another unordered child
2. Ordered second
   - Unordered child
     - Deeper unordered
       1. Switch to ordered
       2. Still ordered

## List with Paragraphs

- Item with a paragraph.

  This is a continuation paragraph under the same list item.

  And another continuation paragraph.

- Second item.

  Also with a continuation.

## List with Other Block Elements

- Item with a code block:

  ```python
  def hello():
      print("Hello from a list!")
  ```

- Item with a blockquote:

  > This is a quote inside a list item.

- Item with a sub-list and text:

  Some text before the sub-list.

  - Sub-item A
  - Sub-item B

  Some text after the sub-list.

## Empty List Items

- 
- Non-empty
- 
- Also non-empty
