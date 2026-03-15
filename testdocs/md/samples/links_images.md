# Links and Images Edge Cases

## Inline Links

[Basic link](https://example.com)

[Link with title](https://example.com "Example Title")

[Link with special chars](https://example.com/path?q=hello&lang=en#section)

[Empty URL]()

## Reference Links

[Reference link][ref1]

[Another reference][ref2]

[Implicit reference link][]

[ref1]: https://example.com "Reference 1"
[ref2]: https://example.com/other "Reference 2"
[Implicit reference link]: https://example.com/implicit

## Autolinks

<https://example.com>

<user@example.com>

## Links with Formatting

[**Bold link**](https://example.com)

[*Italic link*](https://example.com)

[`Code link`](https://example.com)

[Link with **bold** and *italic*](https://example.com)

## Images

![Alt text](image.png)

![Alt text with title](image.png "Image Title")

![](empty-alt.png)

![Complex alt: a "quoted" & <special> image](path/to/image.jpg)

## Images as Links

[![Image link alt](icon.png)](https://example.com)

## Nested and Adjacent

[Link 1](url1) and [Link 2](url2) on the same line.

Text before [link](url) text after [another](url2) more text.

## Edge Cases

[Link with (parentheses) inside](https://example.com/page_(disambiguation))

[Link with \[brackets\]](https://example.com)

![Image with spaces in path](path/to/my%20image.png)
