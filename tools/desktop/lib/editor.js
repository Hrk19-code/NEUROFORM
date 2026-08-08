// lib/editor.js — CodeMirror 6 editor with TWO NAMED CURSORS (user vs brain).
// Zero-build: vendored ESM files in lib/cm/ + import map in index.html.
// The user cursor is CM's native caret ("you"); the brain cursor is a
// named caret + label ("<file>") rendered as a live decoration.
import { EditorState, EditorView, basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { StateField, StateEffect, RangeSet, Compartment, EditorSelection } from "@codemirror/state";
import { Decoration, WidgetType, keymap } from "@codemirror/view";
import { undo, redo } from "@codemirror/commands";

// ---------- brain cursor ----------
const setBrainCursor = StateEffect.define({ map: (v, m) => ({ pos: m.mapPos(v.pos), name: v.name }) });
class BrainCaret extends WidgetType {
  constructor(name) { super(); this.name = name; }
  eq(o) { return o.name === this.name; }
  toDOM() { const s = document.createElement('span'); s.className = 'bcursor'; s.textContent = '▍' + this.name; return s; }
  ignoreEvent() { return true; }
}
const brainCursorField = StateField.define({
  create: () => Decoration.none,
  update(deco, tr) {
    let out = deco.map(tr.changes);
    for (const e of tr.effects) if (e.is(setBrainCursor)) {
      out = (e.value.pos == null) ? Decoration.none
        : RangeSet.of(Decoration.widget({ widget: new BrainCaret(e.value.name), side: 1 }).range(e.value.pos));
    }
    return out;
  },
  provide: f => EditorView.decorations.from(f),
});

// ---------- dark theme (app palette, no external css) ----------
const darkTheme = EditorView.theme({
  '&': { backgroundColor: '#0f1620', color: '#c8d6e3', height: '100%' },
  '.cm-content': { caretColor: '#5fe3a0', fontFamily: '"Segoe UI", system-ui, sans-serif', fontSize: '13.5px', lineHeight: '1.6' },
  '.cm-cursor, .cm-dropCursor': { borderLeftColor: '#5fe3a0', borderLeftWidth: '2px' },
  '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground': { background: '#1d3a4d66' },
  '.cm-gutters': { backgroundColor: '#0f1620', color: '#33475a', border: 'none' },
  '.cm-activeLine': { backgroundColor: '#131c2644' },
  '&.cm-focused > .cm-scroller > .cm-activeLine': { backgroundColor: '#131c2633' },
  '.cm-activeLineGutter': { backgroundColor: '#131c26' },
  '.cm-selectionMatch': { backgroundColor: '#ffb34733' },
  '.cm-searchMatch': { backgroundColor: '#ffb34744', outline: 'none' },
  '.cm-panels': { backgroundColor: '#131c26', color: '#c8d6e3' },
  '.cm-panels input, .cm-panels button': { background: '#182330', color: '#c8d6e3', border: '1px solid #33475a', borderRadius: '4px' },
  '.cm-tooltip': { backgroundColor: '#131c26', border: '1px solid #33475a', color: '#c8d6e3' },
  '.cm-tooltip-autocomplete ul li[aria-selected]': { backgroundColor: '#16445f', color: '#c8d6e3' },
  '.bcursor': { color: '#ffb347', fontWeight: '700', animation: 'blink 1.1s step-end infinite' },
  '&.cm-focused .cm-matchingBracket': { backgroundColor: '#22303e', outline: '1px solid #33475a' },
});

// ---------- compartments (toggles) ----------
const focusCompartment = new Compartment();
const typewriterCompartment = new Compartment();
const mdCompartment = new Compartment();

const focusMode = EditorView.decorations.compute(['doc'], state => {
  const line = state.doc.lineAt(state.selection.main.head);
  const deco = [];
  for (let i = 1; i <= state.doc.lines; i++) {
    if (i !== line.number) deco.push(Decoration.line({ class: 'cm-dim' }).range(state.doc.line(i).from));
  }
  return Decoration.set(deco);
});

const typewriterScroll = EditorView.updateListener.of(u => {
  if (u.docChanged || u.selectionSet) {
    u.view.scrollIntoView(u.state.selection.main.head, { y: 'center' });
  }
});

// ---------- word/char counts ----------
function countStats(state) {
  const text = state.doc.toString();
  const words = (text.match(/\S+/g) || []).length;
  return { words, chars: text.length, lines: state.doc.lines, pos: state.selection.main.head };
}

// ---------- bold/italic (prose-friendly markdown toggles) ----------
const boldKey = keymap.of([{
  key: 'Ctrl-b', preventDefault: true,
  run: (view) => toggleAround(view, '**')
}, {
  key: 'Ctrl-i', preventDefault: true,
  run: (view) => toggleAround(view, '*')
}]);
function toggleAround(view, mark) {
  const { from, to } = view.state.selection.main;
  const sel = view.state.sliceDoc(from, to);
  const has = sel.startsWith(mark) && sel.endsWith(mark);
  const wrapped = has ? sel.slice(mark.length, sel.length - mark.length) : mark + sel + mark;
  view.dispatch({ changes: { from, to, insert: wrapped }, selection: EditorSelection.single(from, from + wrapped.length) });
  return true;
}

// ---------- the editor ----------
export function createEditor(host, opts = {}) {
  const { onCount, onDocChange } = opts;
  let typewriterOn = false;
  const countListener = EditorView.updateListener.of(u => {
    if (u.docChanged || u.selectionSet) onCount(countStats(u.state));
    if (u.docChanged && onDocChange) onDocChange(u.state);
  });
  const mdExt = mdCompartment.of(opts.markdownMode ? markdown() : []);
  const state = EditorState.create({
    doc: opts.doc || '',
    extensions: [
      basicSetup, darkTheme, boldKey,
      countListener,
      brainCursorField,
      focusCompartment.of([]),
      typewriterCompartment.of([]),
      mdExt,
    ],
  });
  const view = new EditorView({ state, parent: host });

  // brain cursor initial + API
  if (opts.brainCursor) setBrain(view, opts.brainCursor);
  onCount && onCount(countStats(state));

  return {
    view,
    setDoc(text, markdownMode) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text }, addToHistory: false });
      view.dispatch({ effects: mdCompartment.reconfigure(markdownMode ? markdown() : []), addToHistory: false });
    },
    setBrainCursor(pos, name) {
      view.dispatch({ effects: setBrainCursor.of({ pos: pos == null ? null : pos, name }), addToHistory: false });
    },
    setFocusMode(on) {
      view.dispatch({ effects: focusCompartment.reconfigure(on ? [focusMode] : []) });
    },
    setTypewriter(on) {
      typewriterOn = on;
      view.dispatch({ effects: typewriterCompartment.reconfigure(on ? [typewriterScroll] : []) });
    },
    undo() { view.focus(); undo(view); },
    redo() { view.focus(); redo(view); },
    getPos() { return view.state.selection.main.head; },
    setPos(pos) { view.dispatch({ selection: { anchor: pos }, addToHistory: false }); },
    stats() { return countStats(view.state); },
    destroy() { view.destroy(); },
  };
}

export function setBrain(editor, { pos, name }) {
  editor.setBrainCursor(pos, name);
}

// Classic scripts (app.js, tabs/*) cannot import ESM — expose the surface.
window.Writer = { createEditor, setBrain };
