#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');
const sourceRoot = path.join(projectRoot, 'src');
const localeRoot = path.join(sourceRoot, 'locales');
const strict = process.argv.includes('--strict');
const MAX_PRINTED_FINDINGS = 60;

const paths = {
  en: path.join(localeRoot, 'en.json'),
  zh: path.join(localeRoot, 'zh.json'),
};

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function flatten(value, prefix = '', output = new Map()) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => flatten(item, `${prefix}[${index}]`, output));
    return output;
  }
  if (value && typeof value === 'object') {
    for (const [key, child] of Object.entries(value)) {
      flatten(child, prefix ? `${prefix}.${key}` : key, output);
    }
    return output;
  }
  output.set(prefix, value);
  return output;
}

function placeholders(value) {
  if (typeof value !== 'string') return [];
  return [...value.matchAll(/\{\s*([\w.-]+)\s*\}/g)]
    .map((match) => match[1])
    .sort();
}

function sameList(a, b) {
  return a.length === b.length && a.every((item, index) => item === b[index]);
}

function walk(dir) {
  const files = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name.startsWith('.') || entry.name === 'node_modules' || entry.name === 'dist') continue;
    const target = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...walk(target));
    else files.push(target);
  }
  return files;
}

const safeLiteralPatterns = [
  /^Lap$/i,
  /^(RAW|GPS|AI|EXIF|HEIC|HEIF|AVIF|JPEG|JPG|PNG|GIF|WebP|PSD|SVG|HDR|EXR|MP4|MOV|MKV|AVI)$/i,
  /^(Kodak|Toyo)$/i,
  /^https?:\/\//i,
  /^[A-Z0-9_.+\-/]{1,12}$/,
  /^#[0-9a-f]{3,8}$/i,
];

function isSafeLiteral(value) {
  return safeLiteralPatterns.some((pattern) => pattern.test(value.trim()));
}

function lineNumber(source, offset) {
  return source.slice(0, offset).split('\n').length;
}

function extractTemplate(source) {
  const match = source.match(/<template(?:\s[^>]*)?>([\s\S]*?)<\/template>/i);
  return match ? { content: match[1], offset: match.index + match[0].indexOf(match[1]) } : null;
}

function blankRangePreservingLines(value, pattern) {
  return value.replace(pattern, (match) => match.replace(/[^\n]/g, ' '));
}

function auditTemplate(filePath) {
  const source = fs.readFileSync(filePath, 'utf8');
  const extracted = extractTemplate(source);
  if (!extracted) return [];

  const findings = [];
  const relative = path.relative(projectRoot, filePath).replaceAll(path.sep, '/');
  let visibleTemplate = extracted.content;
  visibleTemplate = blankRangePreservingLines(visibleTemplate, /<!--[\s\S]*?-->/g);
  visibleTemplate = blankRangePreservingLines(visibleTemplate, /\{\{[\s\S]*?\}\}/g);

  // Static text nodes only. Vue interpolation has already been blanked, so
  // operators such as `>` inside expressions cannot be mistaken for text.
  const textPattern = />([^<\n]*[A-Za-z][^<\n]*)</g;
  for (const match of visibleTemplate.matchAll(textPattern)) {
    const value = match[1].replace(/&(?:gt|lt|amp|quot|apos);/g, ' ').replace(/\s+/g, ' ').trim();
    if (!value || isSafeLiteral(value)) continue;
    findings.push({
      file: relative,
      line: lineNumber(source, extracted.offset + match.index),
      kind: 'template-text',
      value,
    });
  }

  // Only unbound static attributes are user-visible literals. Bound forms
  // (`:title`, `v-bind:title`) are expressions and must not be reported.
  const attrPattern = /(?:^|\s)(title|placeholder|aria-label|alt|data-tip)\s*=\s*["']([^"']*[A-Za-z][^"']*)["']/gim;
  for (const match of visibleTemplate.matchAll(attrPattern)) {
    const value = match[2].trim();
    if (!value || isSafeLiteral(value)) continue;
    findings.push({
      file: relative,
      line: lineNumber(source, extracted.offset + match.index),
      kind: 'literal-attribute',
      value,
    });
  }

  return findings;
}

const en = flatten(readJson(paths.en));
const zh = flatten(readJson(paths.zh));
const missing = [];
const extra = [];
const blank = [];
const placeholderMismatch = [];
const suspiciousEnglish = [];

for (const [key, enValue] of en) {
  if (!zh.has(key)) {
    missing.push(key);
    continue;
  }
  const zhValue = zh.get(key);
  if (typeof zhValue === 'string' && zhValue.trim() === '') blank.push(key);
  const enPlaceholders = placeholders(enValue);
  const zhPlaceholders = placeholders(zhValue);
  if (!sameList(enPlaceholders, zhPlaceholders)) {
    placeholderMismatch.push({ key, en: enPlaceholders, zh: zhPlaceholders });
  }
  if (
    typeof enValue === 'string' &&
    typeof zhValue === 'string' &&
    /[A-Za-z]{3,}/.test(zhValue) &&
    !isSafeLiteral(zhValue) &&
    enValue.trim() === zhValue.trim()
  ) {
    suspiciousEnglish.push({ key, value: zhValue });
  }
}

for (const key of zh.keys()) {
  if (!en.has(key)) extra.push(key);
}

const hardcoded = walk(sourceRoot)
  .filter((file) => file.endsWith('.vue'))
  .flatMap(auditTemplate);

function printList(title, rows, formatter = (row) => String(row)) {
  if (!rows.length) return;
  console.log(`\n${title} (${rows.length})`);
  for (const row of rows.slice(0, MAX_PRINTED_FINDINGS)) console.log(`  - ${formatter(row)}`);
  if (rows.length > MAX_PRINTED_FINDINGS) {
    console.log(`  ... 其余 ${rows.length - MAX_PRINTED_FINDINGS} 项已省略`);
  }
}

console.log('Lap 简体中文界面审计');
console.log(`英文键数量: ${en.size}`);
console.log(`中文键数量: ${zh.size}`);
console.log(`严格模式: ${strict ? '开启' : '关闭'}`);

printList('缺失中文键', missing);
printList('中文空值', blank);
printList(
  '占位符不一致',
  placeholderMismatch,
  (row) => `${row.key}: en=[${row.en.join(', ')}], zh=[${row.zh.join(', ')}]`,
);
printList('中文语言包中疑似未翻译的英文', suspiciousEnglish, (row) => `${row.key}: ${row.value}`);
printList('Vue 模板中疑似硬编码英文', hardcoded, (row) => `${row.file}:${row.line} [${row.kind}] ${row.value}`);
printList('仅存在于中文语言包的键', extra);

const fatalCount = missing.length + blank.length + placeholderMismatch.length;
const strictCount = suspiciousEnglish.length + hardcoded.length;
const failed = fatalCount > 0 || (strict && strictCount > 0);

console.log('\n审计结果');
console.log(`  基础错误: ${fatalCount}`);
console.log(`  严格模式问题: ${strictCount}`);
console.log(`  状态: ${failed ? 'FAILED' : 'PASS'}`);

if (failed) process.exitCode = 1;
