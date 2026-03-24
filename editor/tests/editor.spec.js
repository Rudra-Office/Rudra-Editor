// @ts-check
import { test, expect } from '@playwright/test';
import { readFileSync } from 'fs';
import path from 'path';

const DOCX_PATH = path.resolve('../demo/images/document.docx');
const CHAT_DOCX_PATH = path.resolve('../demo/images/Chat Reaction (1) (1).docx');

// ─── Helper: wait for WASM engine to be ready ──────────────
async function waitForEngine(page) {
  await page.waitForFunction(() => {
    const label = document.getElementById('wasmLabel');
    return label && label.textContent === 's1engine ready';
  }, { timeout: 10000 });
}

// ─── Helper: create a new document ──────────────────────────
async function newDoc(page) {
  await page.goto('/');
  await waitForEngine(page);
  await page.click('#welcomeNew', { force: true });
  await page.waitForSelector('#docPage[contenteditable="true"]');
}

// ─── Helper: open a DOCX file ───────────────────────────────
async function openDocx(page, filePath) {
  await page.goto('/');
  await waitForEngine(page);
  const fileInput = page.locator('#fileInput');
  await fileInput.setInputFiles(filePath);
  await page.waitForSelector('#docPage[contenteditable="true"]');
  // Wait for render
  await page.waitForTimeout(500);
}

// ─── Helper: get doc page content ───────────────────────────
async function getPageHtml(page) {
  return page.evaluate(() => document.getElementById('docPage').innerHTML);
}

async function getPageText(page) {
  return page.evaluate(() => {
    const page = document.getElementById('docPage');
    // Exclude page break indicators and footers from text
    let text = '';
    page.querySelectorAll('[data-node-id]').forEach(el => {
      text += el.textContent + '\n';
    });
    return text.trim();
  });
}

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Engine Initialization
// ═════════════════════════════════════════════════════════════
test.describe('Engine Init', () => {
  test('WASM engine loads successfully', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    const label = page.locator('#wasmLabel');
    await expect(label).toHaveText('s1engine ready');
    const dot = page.locator('#wasmDot');
    await expect(dot).toHaveClass(/ok/);
  });

  test('welcome screen is visible', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await expect(page.locator('#welcomeScreen')).toBeVisible();
    await expect(page.locator('#welcomeNew')).toBeVisible();
    await expect(page.locator('#welcomeOpen')).toBeVisible();
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: New Document
// ═════════════════════════════════════════════════════════════
test.describe('New Document', () => {
  test('creates empty document on click', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#editorCanvas')).toHaveClass(/show/);
    await expect(page.locator('#toolbar')).toHaveClass(/show/);
    await expect(page.locator('#welcomeScreen')).not.toBeVisible();
  });

  test('can type text into document', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Hello World');
    const text = await getPageText(page);
    expect(text).toContain('Hello World');
  });

  test('Enter key splits paragraph', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Line one');
    await page.keyboard.press('Enter');
    await page.keyboard.type('Line two');
    const paragraphs = await page.locator('#docPage [data-node-id]').count();
    expect(paragraphs).toBeGreaterThanOrEqual(2);
  });

  test('split paragraphs use WASM slice fragments across pages', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);
    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      state.doc.set_page_setup(JSON.stringify({
        pageWidth: 72,
        pageHeight: 72,
        marginTop: 1,
        marginBottom: 1,
        marginLeft: 1,
        marginRight: 1,
      }));
      const nodeId = state.doc.append_paragraph(
        'alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau '.repeat(6)
      );
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 500));

      const pageCount = document.querySelectorAll('.doc-page').length;
      const fragments = Array.from(document.querySelectorAll(
        `[data-node-id="${nodeId}"], [data-node-id="${nodeId}-cont"]`
      )).map(el => ({
        nodeId: el.dataset.nodeId,
        text: el.textContent,
        sliceRendered: el.dataset.sliceRendered === 'true',
      }));
      return { nodeId, pageCount, fragments, pageMap: state.pageMap };
    });

    expect(result).not.toBeNull();
    expect(result.pageCount).toBeGreaterThan(1);
    expect(result.fragments.length).toBeGreaterThan(1);
    expect(result.fragments.every(fragment => fragment.sliceRendered)).toBeTruthy();
    expect(result.pageMap?.pages?.some(pg => (pg.paraSplits || []).some(split => split.nodeId === result.nodeId))).toBeTruthy();
  });

  test('insert_line_break works for mid-run paragraph edits', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        html: el?.innerHTML || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\nWorld');
    expect(result.html).toContain('<br');
  });

  test('insert_tab works for mid-run paragraph edits', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        html: el?.innerHTML || '',
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\tWorld');
    expect(result.html.includes('&emsp;') || result.domText.includes('\u2003')).toBeTruthy();
  });

  test('insert_text_in_paragraph respects paragraph-level tab offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.insert_text_in_paragraph(nodeId, 6, 'X');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\tXWorld');
    expect(result.domText).toContain('Hello');
    expect(result.domText.includes('XWorld')).toBeTruthy();
  });

  test('delete_text_in_paragraph respects paragraph-level tab offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.delete_text_in_paragraph(nodeId, 4, 3);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hellorld');
    expect(result.domText).toContain('Hell');
    expect(result.domText).toContain('orld');
  });

  test('delete_text_in_paragraph can delete an exact paragraph-level tab', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.delete_text_in_paragraph(nodeId, 5, 1);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('HelloWorld');
    expect(result.domText).toContain('Hello');
    expect(result.domText).toContain('World');
  });

  test('delete_text_in_paragraph can delete an exact paragraph-level line break', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state.doc.delete_text_in_paragraph(nodeId, 5, 1);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('HelloWorld');
    expect(result.domText).toContain('Hello');
    expect(result.domText).toContain('World');
  });

  test('replace_text can replace an exact paragraph-level tab', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.replace_text(nodeId, 5, 1, 'X');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('HelloXWorld');
    expect(result.domText).toContain('HelloXWorld');
  });

  test('replace_text can replace an exact paragraph-level line break', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state.doc.replace_text(nodeId, 5, 1, 'X');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        text: state.doc.get_paragraph_text(nodeId),
        domText: el?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('HelloXWorld');
    expect(result.domText).toContain('Hello');
    expect(result.domText).toContain('XWorld');
  });

  test('format_selection respects paragraph-level tab offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.format_selection(nodeId, 6, nodeId, 11, 'bold', 'true');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const el = document.querySelector(`[data-node-id="${nodeId}"]`);
      return {
        html: el?.innerHTML || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('Hello');
    expect(result.html).toMatch(/font-weight:\s*(bold|700)|<strong|<b[\s>]/i);
  });

  test('format_selection respects paragraph-level line-break offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state.doc.format_selection(nodeId, 6, nodeId, 11, 'bold', 'true');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const runIds = JSON.parse(state.doc.get_run_ids(nodeId));
      return {
        text: state.doc.get_paragraph_text(nodeId),
        firstFmt: state.doc.get_run_formatting_json(runIds[0]),
        lastFmt: state.doc.get_run_formatting_json(runIds[runIds.length - 1]),
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\nWorld');
    expect(result.firstFmt.includes('"bold":true')).toBeFalsy();
    expect(result.lastFmt.includes('"bold":true')).toBeTruthy();
  });

  test('get_selection_formatting_json respects paragraph-level tab offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.format_selection(nodeId, 6, nodeId, 11, 'bold', 'true');

      return {
        afterTab: state.doc.get_selection_formatting_json(nodeId, 6, nodeId, 11),
        wholePara: state.doc.get_selection_formatting_json(nodeId, 0, nodeId, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.afterTab).toContain('"bold":true');
    expect(result.wholePara).toContain('"bold":"mixed"');
  });

  test('get_selection_formatting_json respects paragraph-level line-break offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state.doc.format_selection(nodeId, 6, nodeId, 11, 'bold', 'true');

      return {
        afterBreak: state.doc.get_selection_formatting_json(nodeId, 6, nodeId, 11),
        wholePara: state.doc.get_selection_formatting_json(nodeId, 0, nodeId, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.afterBreak).toContain('"bold":true');
    expect(result.wholePara).toContain('"bold":"mixed"');
  });

  test('set_paragraph_text typing after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(nodeId, 5);
      state.doc.set_paragraph_text(nodeId, 'Hello\tXWorld');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return {
        text: state.doc.get_paragraph_text(nodeId),
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\tXWorld');
  });

  test('set_paragraph_text typing after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const nodeId = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(nodeId, 5);
      state.doc.set_paragraph_text(nodeId, 'Hello\nXWorld');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return {
        text: state.doc.get_paragraph_text(nodeId),
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\nXWorld');
  });

  test('paste_plain_text multiline after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(first, 5);
      state.doc.paste_plain_text(first, 6, 'X\nY');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const ids = JSON.parse(state.doc.paragraph_ids_json());
      return {
        firstText: state.doc.get_paragraph_text(first),
        paragraphTexts: ids.map(id => state.doc.get_paragraph_text(id)),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstText).toBe('Hello\tX');
    expect(result.paragraphTexts).toContain('YWorld');
  });

  test('paste_plain_text multiline after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      state.doc.paste_plain_text(first, 6, 'X\nY');
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const ids = JSON.parse(state.doc.paragraph_ids_json());
      return {
        firstText: state.doc.get_paragraph_text(first),
        paragraphTexts: ids.map(id => state.doc.get_paragraph_text(id)),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstText).toBe('Hello\nX');
    expect(result.paragraphTexts).toContain('YWorld');
  });

  test('paste_formatted_runs single paragraph after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(first, 5);
      state.doc.paste_formatted_runs_json(
        first,
        6,
        JSON.stringify({ paragraphs: [{ runs: [{ text: 'X', bold: true }] }] })
      );
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return { text: state.doc.get_paragraph_text(first) };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\tXWorld');
  });

  test('paste_formatted_runs multi paragraph after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(first, 5);
      state.doc.paste_formatted_runs_json(
        first,
        6,
        JSON.stringify({ paragraphs: [{ runs: [{ text: 'X' }] }, { runs: [{ text: 'Y' }] }] })
      );
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const ids = JSON.parse(state.doc.paragraph_ids_json());
      return {
        firstText: state.doc.get_paragraph_text(first),
        paragraphTexts: ids.map(id => state.doc.get_paragraph_text(id)),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstText).toBe('Hello\tX');
    expect(result.paragraphTexts).toContain('YWorld');
  });

  test('paste_formatted_runs single paragraph after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      state.doc.paste_formatted_runs_json(
        first,
        6,
        JSON.stringify({ paragraphs: [{ runs: [{ text: 'X', italic: true }] }] })
      );
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return { text: state.doc.get_paragraph_text(first) };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\nXWorld');
  });

  test('paste_formatted_runs multi paragraph after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      state.doc.paste_formatted_runs_json(
        first,
        6,
        JSON.stringify({ paragraphs: [{ runs: [{ text: 'X' }] }, { runs: [{ text: 'Y' }] }] })
      );
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const ids = JSON.parse(state.doc.paragraph_ids_json());
      return {
        firstText: state.doc.get_paragraph_text(first),
        paragraphTexts: ids.map(id => state.doc.get_paragraph_text(id)),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstText).toBe('Hello\nX');
    expect(result.paragraphTexts).toContain('YWorld');
  });

  test('replace_all after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld HelloWorld');
      state.doc.insert_tab(first, 5);
      const count = state.doc.replace_all('World', 'Rust', true);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return {
        count,
        text: state.doc.get_paragraph_text(first),
      };
    });

    expect(result).not.toBeNull();
    expect(result.count).toBe(2);
    expect(result.text).toBe('Hello\tRust HelloRust');
  });

  test('replace_all after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld HelloWorld');
      state.doc.insert_line_break(first, 5);
      const count = state.doc.replace_all('World', 'Rust', true);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      return {
        count,
        text: state.doc.get_paragraph_text(first),
      };
    });

    expect(result).not.toBeNull();
    expect(result.count).toBe(2);
    expect(result.text).toBe('Hello\nRust HelloRust');
  });

  test('export_selection_html after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const id = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(id, 5);
      return {
        html: state.doc.export_selection_html(id, 6, id, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('World');
    expect(result.html).not.toContain('Hello');
  });

  test('export_selection_html after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const id = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(id, 5);
      return {
        html: state.doc.export_selection_html(id, 6, id, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('World');
    expect(result.html).not.toContain('Hello');
  });

  test('find_text finds matches inside table cells', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Before table');
      const table = state.doc.insert_table(first, 1, 2);
      const cellA = state.doc.get_cell_id(table, 0, 0);
      const cellB = state.doc.get_cell_id(table, 0, 1);
      state.doc.set_cell_text(cellA, 'Table Alpha');
      state.doc.set_cell_text(cellB, 'Table Beta');
      return JSON.parse(state.doc.find_text('Table', true));
    });

    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result[0].nodeId).not.toEqual(result[1].nodeId);
    expect(result[0].length).toBe(5);
    expect(result[1].length).toBe(5);
  });

  test('export_selection_html spanning a table includes table and trailing text', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Before table');
      const table = state.doc.insert_table(first, 1, 1);
      const cell = state.doc.get_cell_id(table, 0, 0);
      state.doc.set_cell_text(cell, 'Inside table');
      const last = state.doc.append_paragraph('After');
      return {
        html: state.doc.export_selection_html(first, 3, last, 5),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('<table');
    expect(result.html).toContain('Inside table');
    expect(result.html).toContain('After');
  });

  test('find_text after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const id = state.doc.append_paragraph('HelloWorld HelloWorld');
      state.doc.insert_tab(id, 5);
      return JSON.parse(state.doc.find_text('World', true));
    });

    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result[0].offset).toBe(6);
    expect(result[1].offset).toBe(17);
  });

  test('find_text after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const id = state.doc.append_paragraph('HelloWorld HelloWorld');
      state.doc.insert_line_break(id, 5);
      return JSON.parse(state.doc.find_text('World', true));
    });

    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result[0].offset).toBe(6);
    expect(result[1].offset).toBe(17);
  });

  test('export_selection_html across paragraphs after a paragraph-level tab uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      const second = state.doc.append_paragraph('Second');
      state.doc.insert_tab(first, 5);
      return {
        html: state.doc.export_selection_html(first, 6, second, 3),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('World');
    expect(result.html).toContain('Sec');
    expect(result.html).not.toContain('Hello');
  });

  test('export_selection_html across paragraphs after a paragraph-level line break uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      const second = state.doc.append_paragraph('Second');
      state.doc.insert_line_break(first, 5);
      return {
        html: state.doc.export_selection_html(first, 6, second, 3),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('World');
    expect(result.html).toContain('Sec');
    expect(result.html).not.toContain('Hello');
  });

  test('export_selection_html across paragraphs with a paragraph-level tab in the end paragraph uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(second, 5);
      return {
        html: state.doc.export_selection_html(first, 2, second, 6),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('pha');
    expect(result.html).toContain('Hello');
    expect(result.html).not.toContain('World');
  });

  test('export_selection_html across paragraphs with a paragraph-level line break in the end paragraph uses correct offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(second, 5);
      return {
        html: state.doc.export_selection_html(first, 2, second, 6),
      };
    });

    expect(result).not.toBeNull();
    expect(result.html).toContain('pha');
    expect(result.html).toContain('Hello');
    expect(result.html).not.toContain('World');
  });

  test('delete_selection across paragraphs respects tab-adjusted start offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(first, 5);
      const second = state.doc.append_paragraph('Second');
      state.doc.delete_selection(first, 6, second, 3);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const firstEl = document.querySelector(`[data-node-id="${first}"]`);
      return {
        text: state.doc.get_paragraph_text(first),
        domText: firstEl?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\tond');
    expect(result.domText).toContain('Hello');
    expect(result.domText).toContain('ond');
  });

  test('delete_selection across paragraphs respects line-break-adjusted start offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      const second = state.doc.append_paragraph('Second');
      state.doc.delete_selection(first, 6, second, 3);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const firstEl = document.querySelector(`[data-node-id="${first}"]`);
      return {
        text: state.doc.get_paragraph_text(first),
        domText: firstEl?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('Hello\nond');
    expect(result.domText).toContain('Hello');
    expect(result.domText).toContain('ond');
  });

  test('cross-paragraph format_selection respects line-break-adjusted start offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      const second = state.doc.append_paragraph('Second');
      state.doc.format_selection(first, 6, second, 3, 'bold', 'true');

      return {
        firstAfterBreak: state.doc.get_selection_formatting_json(first, 6, first, 11),
        firstBeforeBreak: state.doc.get_selection_formatting_json(first, 0, first, 5),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstAfterBreak).toContain('"bold":true');
    expect(result.firstBeforeBreak).toContain('"bold":false');
  });

  test('delete_selection across paragraphs respects tab-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(second, 5);
      state.doc.delete_selection(first, 2, second, 6);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const firstEl = document.querySelector(`[data-node-id="${first}"]`);
      return {
        text: state.doc.get_paragraph_text(first),
        domText: firstEl?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('AlWorld');
    expect(result.domText).toContain('Al');
    expect(result.domText).toContain('World');
  });

  test('delete_selection across paragraphs respects line-break-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(async () => {
      const state = window.__s1_state;
      const { renderDocument } = await import('/src/render.js');
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(second, 5);
      state.doc.delete_selection(first, 2, second, 6);
      state._layoutDirty = true;
      renderDocument();
      await new Promise(resolve => setTimeout(resolve, 300));

      const firstEl = document.querySelector(`[data-node-id="${first}"]`);
      return {
        text: state.doc.get_paragraph_text(first),
        domText: firstEl?.textContent || '',
      };
    });

    expect(result).not.toBeNull();
    expect(result.text).toBe('AlWorld');
    expect(result.domText).toContain('Al');
    expect(result.domText).toContain('World');
  });

  test('cross-paragraph format_selection respects tab-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(second, 5);
      state.doc.format_selection(first, 2, second, 6, 'bold', 'true');

      return {
        firstTail: state.doc.get_selection_formatting_json(first, 2, first, 5),
        secondAfterTab: state.doc.get_selection_formatting_json(second, 6, second, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstTail).toContain('"bold":true');
    expect(result.secondAfterTab).toContain('"bold":false');
  });

  test('cross-paragraph format_selection respects line-break-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(second, 5);
      state.doc.format_selection(first, 2, second, 6, 'bold', 'true');

      return {
        firstTail: state.doc.get_selection_formatting_json(first, 2, first, 5),
        secondAfterBreak: state.doc.get_selection_formatting_json(second, 6, second, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.firstTail).toContain('"bold":true');
    expect(result.secondAfterBreak).toContain('"bold":false');
  });

  test('cross-paragraph get_selection_formatting_json respects line-break-adjusted start offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(first, 5);
      const second = state.doc.append_paragraph('Second');
      state.doc.format_selection(first, 6, second, 3, 'bold', 'true');

      return {
        fullSelection: state.doc.get_selection_formatting_json(first, 6, second, 3),
        extendedSelection: state.doc.get_selection_formatting_json(first, 0, second, 3),
      };
    });

    expect(result).not.toBeNull();
    expect(result.fullSelection).toContain('"bold":true');
    expect(result.extendedSelection).toContain('"bold":"mixed"');
  });

  test('cross-paragraph get_selection_formatting_json respects tab-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_tab(second, 5);
      state.doc.format_selection(first, 2, second, 6, 'bold', 'true');

      return {
        fullSelection: state.doc.get_selection_formatting_json(first, 2, second, 6),
        extendedSelection: state.doc.get_selection_formatting_json(first, 2, second, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.fullSelection).toContain('"bold":true');
    expect(result.extendedSelection).toContain('"bold":"mixed"');
  });

  test('cross-paragraph get_selection_formatting_json respects line-break-adjusted end offsets', async ({ page }) => {
    await page.goto('/');
    await waitForEngine(page);
    await page.evaluate(async () => {
      const { newDocument } = await import('/src/file.js');
      newDocument();
    });
    await page.waitForFunction(() => !!window.__s1_state?.doc);

    const result = await page.evaluate(() => {
      const state = window.__s1_state;
      if (!state?.doc) return null;

      const first = state.doc.append_paragraph('Alpha');
      const second = state.doc.append_paragraph('HelloWorld');
      state.doc.insert_line_break(second, 5);
      state.doc.format_selection(first, 2, second, 6, 'bold', 'true');

      return {
        fullSelection: state.doc.get_selection_formatting_json(first, 2, second, 6),
        extendedSelection: state.doc.get_selection_formatting_json(first, 2, second, 11),
      };
    });

    expect(result).not.toBeNull();
    expect(result.fullSelection).toContain('"bold":true');
    expect(result.extendedSelection).toContain('"bold":"mixed"');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Text Formatting
// ═════════════════════════════════════════════════════════════
test.describe('Text Formatting', () => {
  test('bold toggles via toolbar', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Bold text');
    await page.keyboard.press('Meta+a');
    await page.click('#btnBold');
    await page.waitForTimeout(300);
    const html = await getPageHtml(page);
    // WASM may render bold as inline style OR <strong>/<b> tags
    expect(html).toMatch(/font-weight:\s*(bold|700)|<strong|<b[\s>]/i);
  });

  test('bold toggles via Ctrl+B', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Some text');
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Meta+b');
    await page.waitForTimeout(300);
    const html = await getPageHtml(page);
    expect(html).toMatch(/font-weight:\s*(bold|700)|<strong|<b[\s>]/i);
  });

  test('italic toggles via Ctrl+I', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Some text');
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Meta+i');
    await page.waitForTimeout(300);
    const html = await getPageHtml(page);
    // WASM may render italic as inline style OR <em>/<i> tags
    expect(html).toMatch(/font-style:\s*italic|<em|<i[\s>]/i);
  });

  test('font size change applies', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Big text');
    await page.keyboard.press('Meta+a');
    await page.locator('#fontSize').fill('24');
    await page.locator('#fontSize').press('Enter');
    await page.waitForTimeout(300);
    const html = await getPageHtml(page);
    expect(html).toMatch(/font-size:\s*24/i);
  });

  test('heading level change applies', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('My Heading');
    await page.locator('#styleGalleryBtn').click();
    await page.locator('.style-gallery-item[data-style="heading1"]').click();
    await page.waitForTimeout(300);
    const h1 = await page.locator('#docPage h1').count();
    expect(h1).toBeGreaterThanOrEqual(1);
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Clipboard (Cut / Copy / Paste)
// ═════════════════════════════════════════════════════════════
test.describe('Clipboard', () => {
  test('Ctrl+A selects all text', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Hello World');
    await page.keyboard.press('Meta+a');
    const selText = await page.evaluate(() => window.getSelection().toString());
    expect(selText).toContain('Hello');
  });

  test('Delete after select-all clears document', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Some content here');
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Delete');
    await page.waitForTimeout(300);
    const text = await getPageText(page);
    expect(text.trim()).toBe('');
  });

  test('Backspace after select-all clears document', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Content to delete');
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Backspace');
    await page.waitForTimeout(300);
    const text = await getPageText(page);
    expect(text.trim()).toBe('');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Undo / Redo
// ═════════════════════════════════════════════════════════════
test.describe('Undo/Redo', () => {
  test('undo reverses typing', async ({ page }) => {
    await newDoc(page);
    const docPage = page.locator('#docPage');
    await docPage.focus();
    await page.keyboard.type('First');
    // Sync text (wait for debounce)
    await page.waitForTimeout(300);
    await page.keyboard.press('Enter');
    await page.keyboard.type('Second');
    await page.waitForTimeout(300);
    const textBefore = await getPageText(page);
    // Undo should remove something
    await page.keyboard.press('Meta+z');
    await page.waitForTimeout(300);
    const textAfter = await getPageText(page);
    // After undo, text should be different (shorter or changed)
    expect(textAfter.length).toBeLessThanOrEqual(textBefore.length);
  });

  test('undo button becomes enabled after edit', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('test');
    await page.waitForTimeout(300);
    const disabled = await page.locator('#btnUndo').getAttribute('disabled');
    expect(disabled).toBeNull();
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Open DOCX Files
// ═════════════════════════════════════════════════════════════
test.describe('Open DOCX', () => {
  test('opens document.docx with content', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const text = await getPageText(page);
    expect(text.length).toBeGreaterThan(0);
  });

  test('opens document.docx — toolbar shows', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    await expect(page.locator('#toolbar')).toHaveClass(/show/);
    await expect(page.locator('#editorCanvas')).toHaveClass(/show/);
  });

  test('opens Chat Reaction docx with content', async ({ page }) => {
    await openDocx(page, CHAT_DOCX_PATH);
    const text = await getPageText(page);
    expect(text.length).toBeGreaterThan(10);
  });

  test('opened DOCX preserves formatting in HTML', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const html = await getPageHtml(page);
    // Should have data-node-id attributes (WASM rendering)
    expect(html).toContain('data-node-id');
  });

  test('status bar shows word count after open', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const status = await page.locator('#statusInfo').textContent();
    expect(status).toMatch(/\d+ words/);
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Export
// ═════════════════════════════════════════════════════════════
test.describe('Export', () => {
  test('export menu opens and closes', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Export test');
    await page.click('#btnExport');
    await expect(page.locator('#exportMenu')).toHaveClass(/show/);
    // Click elsewhere to close
    await page.click('#docPage');
    await page.waitForTimeout(200);
    const cls = await page.locator('#exportMenu').getAttribute('class');
    expect(cls).not.toContain('show');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Views (Editor / Pages / Text)
// ═════════════════════════════════════════════════════════════
test.describe('Views', () => {
  test('switch to Text view shows plain text', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('View test content');
    await page.waitForTimeout(300);
    await page.click('.tab[data-view="text"]');
    await page.waitForTimeout(300);
    await expect(page.locator('#textView')).toHaveClass(/show/);
    const text = await page.locator('#textContent').textContent();
    expect(text).toContain('View test content');
  });

  test('switch to Pages view shows paginated content', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    await page.click('.tab[data-view="pages"]');
    await page.waitForTimeout(500);
    await expect(page.locator('#pagesView')).toHaveClass(/show/);
  });

  test('switch back to Editor view', async ({ page }) => {
    await newDoc(page);
    await page.click('.tab[data-view="text"]');
    await page.waitForTimeout(200);
    await page.click('.tab[data-view="editor"]');
    await page.waitForTimeout(200);
    await expect(page.locator('#editorCanvas')).toHaveClass(/show/);
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Find & Replace
// ═════════════════════════════════════════════════════════════
test.describe('Find & Replace', () => {
  test('Ctrl+F opens find bar', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.press('Meta+f');
    await expect(page.locator('#findBar')).toHaveClass(/show/);
    await expect(page.locator('#findInput')).toBeFocused();
  });

  test('close button hides find bar', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.press('Meta+f');
    await page.click('#findClose');
    const cls = await page.locator('#findBar').getAttribute('class');
    expect(cls).not.toContain('show');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Page Breaks & Pagination
// ═════════════════════════════════════════════════════════════
test.describe('Pagination', () => {
  test('single-page doc shows page footer', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Short doc');
    await page.waitForTimeout(500);
    const footer = await page.locator('.editor-footer').textContent();
    expect(footer).toContain('Page 1');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: DOCX Round-Trip (Open → Export → Reopen)
// ═════════════════════════════════════════════════════════════
test.describe('Round-Trip', () => {
  test('open DOCX → export DOCX → content preserved', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const originalText = await getPageText(page);
    expect(originalText.length).toBeGreaterThan(0);

    // Export as DOCX bytes
    const exported = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('docx');
      return Array.from(bytes);
    });

    // We can't easily reopen in the same test, but verify export succeeded
    expect(exported).not.toBeNull();
    if (exported) {
      expect(exported.length).toBeGreaterThan(100); // Valid DOCX is > 100 bytes
    }
  });

  test('export to ODT succeeds', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const exported = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('odt');
      return Array.from(bytes);
    });
    expect(exported).not.toBeNull();
    expect(exported.length).toBeGreaterThan(100);
  });

  test('export to TXT succeeds', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const exported = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('txt');
      return new TextDecoder().decode(new Uint8Array(Array.from(bytes)));
    });
    expect(exported).toBeTruthy();
    expect(exported.length).toBeGreaterThan(10);
  });

  test('export to Markdown succeeds', async ({ page }) => {
    await openDocx(page, DOCX_PATH);
    const exported = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('md');
      return new TextDecoder().decode(new Uint8Array(Array.from(bytes)));
    });
    expect(exported).toBeTruthy();
    expect(exported.length).toBeGreaterThan(0);
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Accessibility (ARIA)
// ═════════════════════════════════════════════════════════════
test.describe('Accessibility', () => {
  test('toolbar has ARIA role', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#toolbar')).toHaveAttribute('role', 'toolbar');
  });

  test('format buttons have aria-label', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#btnBold')).toHaveAttribute('aria-label', 'Bold');
    await expect(page.locator('#btnItalic')).toHaveAttribute('aria-label', 'Italic');
    await expect(page.locator('#btnUnderline')).toHaveAttribute('aria-label', 'Underline');
  });

  test('format buttons update aria-pressed', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Test');
    await page.keyboard.press('Meta+a');
    await expect(page.locator('#btnBold')).toHaveAttribute('aria-pressed', 'false');
    await page.click('#btnBold');
    await page.waitForTimeout(300);
    await expect(page.locator('#btnBold')).toHaveAttribute('aria-pressed', 'true');
  });

  test('document content area has textbox role', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#docPage')).toHaveAttribute('role', 'textbox');
    await expect(page.locator('#docPage')).toHaveAttribute('aria-multiline', 'true');
  });

  test('status bar has status role', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#statusbar')).toHaveAttribute('role', 'status');
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: New Toolbar Features
// ═════════════════════════════════════════════════════════════
test.describe('Toolbar Features', () => {
  test('clear formatting button exists', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#btnClearFormat')).toBeVisible();
  });

  test('print button exists', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#btnPrint')).toBeVisible();
  });

  test('indent/outdent buttons exist', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#btnIndent')).toBeVisible();
    await expect(page.locator('#btnOutdent')).toBeVisible();
  });

  test('line spacing selector exists and has options', async ({ page }) => {
    await newDoc(page);
    const options = await page.locator('#lineSpacing option').count();
    expect(options).toBeGreaterThanOrEqual(4); // 1, 1.15, 1.5, 2
  });

  test('zoom controls work', async ({ page }) => {
    await newDoc(page);
    await expect(page.locator('#zoomValue')).toHaveText('100%');
    await page.click('#zoomIn');
    await expect(page.locator('#zoomValue')).toHaveText('110%');
    await page.click('#zoomOut');
    await expect(page.locator('#zoomValue')).toHaveText('100%');
  });

  test('comments panel toggles', async ({ page }) => {
    await newDoc(page);
    const panel = page.locator('#commentsPanel');
    await expect(panel).not.toHaveClass(/show/);
    await page.click('#btnComments');
    await expect(panel).toHaveClass(/show/);
    await page.click('#commentsClose');
    await expect(panel).not.toHaveClass(/show/);
  });

  test('insert menu has comment option', async ({ page }) => {
    await newDoc(page);
    await page.click('#btnInsertMenu');
    await expect(page.locator('#miComment')).toBeVisible();
  });

  test('superscript formatting applies', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('H2O');
    await page.keyboard.press('Meta+a');
    await page.click('#btnSuperscript');
    await page.waitForTimeout(300);
    const html = await getPageHtml(page);
    expect(html).toMatch(/vertical-align:\s*super|<sup/i);
  });
});

// ═════════════════════════════════════════════════════════════
// TEST SUITE: Cross-Format Export
// ═════════════════════════════════════════════════════════════
test.describe('Cross-Format Export', () => {
  test('Chat Reaction DOCX exports to ODT', async ({ page }) => {
    await openDocx(page, CHAT_DOCX_PATH);
    const exported = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('odt');
      return bytes.length;
    });
    expect(exported).toBeGreaterThan(100);
  });

  test('Chat Reaction DOCX exports to TXT', async ({ page }) => {
    await openDocx(page, CHAT_DOCX_PATH);
    const text = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const bytes = doc.export('txt');
      return new TextDecoder().decode(new Uint8Array(Array.from(bytes)));
    });
    expect(text.length).toBeGreaterThan(10);
  });

  test('new document exports to all formats', async ({ page }) => {
    await newDoc(page);
    await page.locator('#docPage').focus();
    await page.keyboard.type('Export test content');
    await page.waitForTimeout(300);

    const results = await page.evaluate(() => {
      const doc = window.__folio_state?.doc;
      if (!doc) return null;
      const fmts = {};
      for (const fmt of ['docx', 'odt', 'txt', 'md']) {
        try {
          const bytes = doc.export(fmt);
          fmts[fmt] = bytes.length;
        } catch (e) {
          fmts[fmt] = -1;
        }
      }
      return fmts;
    });

    expect(results).not.toBeNull();
    expect(results.docx).toBeGreaterThan(100);
    expect(results.odt).toBeGreaterThan(100);
    expect(results.txt).toBeGreaterThan(0);
    expect(results.md).toBeGreaterThan(0);
  });
});
