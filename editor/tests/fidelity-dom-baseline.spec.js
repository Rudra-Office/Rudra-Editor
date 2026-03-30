// @ts-check
import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '..', '..');
const MANIFEST_PATH = path.join(REPO_ROOT, 'tests', 'fidelity', 'corpus_manifest.json');

function loadGeneratedCases() {
  const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));
  const requested = new Set(
    (process.env.S1_FIDELITY_CASES || '')
      .split(',')
      .map((value) => value.trim())
      .filter(Boolean)
  );

  const cases = (manifest.cases || []).filter((entry) => {
    if (entry.status !== 'generated') return false;
    if (!entry.source_document || !entry.dom_baseline_layout_json) return false;
    return requested.size === 0 || requested.has(entry.id);
  });

  return { manifest, cases };
}

function resolveRepoPath(rawPath) {
  return path.isAbsolute(rawPath) ? rawPath : path.join(REPO_ROOT, rawPath);
}

function ensureParentDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

async function waitForEngine(page) {
  await page.waitForFunction(() => {
    const label = document.getElementById('wasmLabel');
    return label && label.textContent === 's1engine ready';
  }, { timeout: 15000 });
}

async function openDocx(page, filePath) {
  await page.goto('/');
  await waitForEngine(page);
  await page.locator('#fileInput').setInputFiles(filePath);
  await page.waitForFunction(() => {
    const state = window.__s1_state;
    const hasPages = document.querySelectorAll('#pageContainer .doc-page').length > 0;
    const hasContent = !!document.querySelector('#pageContainer .doc-page .page-content');
    return !!state?.doc && hasPages && hasContent;
  }, { timeout: 15000 });
  await page.waitForFunction(() => !document.querySelector('.loading-overlay'), { timeout: 15000 });
  await page.waitForTimeout(500);
}

async function captureDomScene(page) {
  return page.evaluate(() => {
    const PX_TO_PT = 72 / 96;

    const round = (value) => Math.round(value * 1000) / 1000;
    const pxToPt = (value) => round(value * PX_TO_PT);
    const parsePx = (value) => {
      const parsed = Number.parseFloat(value || '0');
      return Number.isFinite(parsed) ? parsed : 0;
    };
    const normalizeText = (value) => {
      const text = (value || '').replace(/\s+/g, ' ').trim();
      return text ? text.slice(0, 120) : null;
    };

    function rectRelativeToPage(rect, pageRect) {
      return {
        x: pxToPt(rect.left - pageRect.left),
        y: pxToPt(rect.top - pageRect.top),
        width: pxToPt(rect.width),
        height: pxToPt(rect.height),
      };
    }

    function contentRectRelativeToPage(contentEl, pageRect) {
      const rect = contentEl.getBoundingClientRect();
      const style = getComputedStyle(contentEl);
      const paddingLeft = parsePx(style.paddingLeft);
      const paddingRight = parsePx(style.paddingRight);
      const paddingTop = parsePx(style.paddingTop);
      const paddingBottom = parsePx(style.paddingBottom);
      return {
        x: pxToPt(rect.left - pageRect.left + paddingLeft),
        y: pxToPt(rect.top - pageRect.top + paddingTop),
        width: pxToPt(Math.max(0, rect.width - paddingLeft - paddingRight)),
        height: pxToPt(Math.max(0, rect.height - paddingTop - paddingBottom)),
      };
    }

    const pageElements = Array.from(document.querySelectorAll('#pageContainer .doc-page'));
    const pages = pageElements.map((pageEl, pageIndex) => {
      const pageRect = pageEl.getBoundingClientRect();
      const headerEl = pageEl.querySelector('.page-header');
      const contentEl = pageEl.querySelector('.page-content');
      const footerEl = pageEl.querySelector('.page-footer');
      const nodeElements = contentEl
        ? Array.from(contentEl.querySelectorAll(':scope > [data-node-id]'))
        : [];

      const nodes = nodeElements.map((el, nodeIndex) => ({
        node_id: el.dataset.nodeId || '',
        tag_name: el.tagName.toLowerCase(),
        order: nodeIndex,
        bounds_pt: rectRelativeToPage(el.getBoundingClientRect(), pageRect),
        table_source: el.dataset.tableSource || null,
        is_table_continuation: el.dataset.isContinuation === 'true',
        split_first: el.dataset.splitFirst === 'true',
        split_continuation: el.dataset.splitContinuation === 'true',
        slice_rendered: el.dataset.sliceRendered === 'true',
        text_preview: normalizeText(el.innerText || el.textContent || ''),
      }));

      return {
        page_index: pageIndex,
        page_num: Number(pageEl.dataset.page || pageIndex + 1),
        bounds_pt: {
          x: 0,
          y: 0,
          width: pxToPt(pageRect.width),
          height: pxToPt(pageRect.height),
        },
        content_rect_pt: contentEl ? contentRectRelativeToPage(contentEl, pageRect) : null,
        header_rect_pt: headerEl ? rectRelativeToPage(headerEl.getBoundingClientRect(), pageRect) : null,
        footer_rect_pt: footerEl ? rectRelativeToPage(footerEl.getBoundingClientRect(), pageRect) : null,
        node_count: nodes.length,
        block_rects_pt: nodes.map((node) => node.bounds_pt),
        node_ids: nodes.map((node) => node.node_id),
        nodes,
      };
    });

    return {
      protocol_version: 1,
      source: 'dom_baseline',
      measurement_unit: 'pt',
      page_count: pages.length,
      pages,
    };
  });
}

test.describe.configure({ mode: 'serial' });

test('capture DOM fidelity baselines for generated corpus cases', async ({ page }) => {
  const { cases } = loadGeneratedCases();
  expect(cases.length).toBeGreaterThan(0);

  await page.setViewportSize({ width: 1600, height: 2200 });

  for (const entry of cases) {
    const sourcePath = resolveRepoPath(entry.source_document);
    const artifactPath = resolveRepoPath(entry.dom_baseline_layout_json);

    expect(fs.existsSync(sourcePath), `source document missing for ${entry.id}`).toBeTruthy();

    await openDocx(page, sourcePath);
    const scene = await captureDomScene(page);

    expect(scene.page_count, `no pages captured for ${entry.id}`).toBeGreaterThan(0);

    ensureParentDir(artifactPath);
    fs.writeFileSync(artifactPath, JSON.stringify(scene, null, 2) + '\n', 'utf8');
  }
});
