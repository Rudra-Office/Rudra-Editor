# React Editor Example

Embed the s1engine document editor in a React application.

## Setup

```bash
npx create-react-app my-editor --template typescript
cd my-editor
npm install @s1engine/react @s1engine/editor @s1engine/wasm
```

Copy `App.tsx` into `src/App.tsx` and run:

```bash
npm start
```

## Features

- Open DOCX, ODT, TXT, Markdown files
- Full toolbar with formatting controls
- Export to PDF
- Real-time collaboration (configure `collab` prop)
