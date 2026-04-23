# POCKET IDE — Spec Completa

> **Replit de bolso. Funciona em qualquer navegador. Zero setup.**

---

## CONCEITO

```
┌─────────────────────────────────────────────────────────────┐
│                                                              │
│  REPLIT DE BOLSO                                           │
│  ─────────────────                                          │
│                                                              │
│  • Abre no navegador (Chrome, Firefox, Safari)            │
│  • Monaco Editor (VS Code quality)                         │
│  • Executa código em WASM sandbox                         │
│  • Terminal integrado (xterm.js)                          │
│  • AI via nexus-protocol (Magic Click)                   │
│  • Funciona offline (Service Worker)                     │
│  • Mobile-friendly (touch support)                       │
│  • 100KB total (gzip)                                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## STACK TÉCNICO

### Frontend (Browser)
```html
<!-- Zero frameworks, vanilla JS para performance máxima -->
<!-- Compilado de Rust/WASM para bundles pequenos -->

<link rel="stylesheet" href="pocket-ide.css">
<div id="editor"></div>
<div id="terminal"></div>
<div id="ai-sidebar"></div>
<script type="module" src="pocket-ide.js"></script>
```

### Editor
- **Monaco Editor** — VS Code editor, 3MB gzipped
- Syntax highlighting: 50+ linguagens
- IntelliSense: JavaScript, TypeScript, Python, Rust, Go
- Minimap, folding, search/replace

### Terminal
- **xterm.js** — terminal emulator
- 256 colors, ligatures, hyperlinks
- Copy/paste support
- Scrollback ilimitado

### Sandbox (WASM)
- **wasm3** — interpretador ultra-rápido
- **wasmer** — compilação JIT opcional
- **Firecracker** — microVM para Rust/Go (futuro)

### AI Integration
- **nexus-protocol** — protocolo aberto
- **Groq API** — inferência cloud (gratuito tier)
- **Ollama local** — inferência offline

---

## ARQUITETURA

```
┌─────────────────────────────────────────────────────────────┐
│                    POCKET IDE (Browser)                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │   MONACO    │  │   XTERM.JS   │  │   AI SIDEBAR    │ │
│  │   EDITOR    │  │   TERMINAL   │  │   (Magic Click) │ │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘ │
│         │                  │                   │            │
│         └──────────────────┼───────────────────┘            │
│                            │                                │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │              NEXUS PROTOCOL (WASM)                    │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │   WASM3    │  │   OLLAMA   │  │   GROQ     │   │  │
│  │  │  SANDBOX   │  │   LOCAL    │  │   API      │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
│                            │                                │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │           FILE SYSTEM ACCESS API                       │  │
│  │   (ou IndexedDB fallback pra browsers antigos)        │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## FUNCIONALIDADES

### 1. Editor de Código
```javascript
// Features:
// - 50+ linguagens
// - IntelliSense
// - Multi-cursor
// - Find/Replace regex
// - Vim/Emacs keybindings (opcional)
// - Minimap
// -Bracket matching

const editor = new PocketEditor({
  element: '#editor',
  language: 'python',
  theme: 'vs-dark',
  fontSize: 14,
  tabSize: 4,
  wordWrap: 'on',
});
```

### 2. Terminal Integrado
```javascript
// Features:
// - 256 colors
// - Hyperlinks clicáveis
// - Selection copy
// - Scrollback infinito
// -输入法 support

const terminal = new PocketTerminal({
  element: '#terminal',
  fontFamily: 'JetBrains Mono, Fira Code, monospace',
  fontSize: 13,
  scrollback: 10000,
});
```

### 3. Execução de Código
```javascript
// Run Python
await pocket.run({
  language: 'python',
  code: 'print("Hello, World!")',
});

// Run JavaScript (WASM sandbox)
await pocket.run({
  language: 'javascript',
  code: 'console.log("Hello!")',
});

// Run Rust (requer backend WASM)
await pocket.run({
  language: 'rust',
  code: 'fn main() { println!("Hello!"); }',
  target: 'wasm',
});
```

### 4. Magic Click (AI)
```javascript
// Clique em qualquer código → Menu de AI aparece
// 1. "Explain this" → IA explica
// 2. "Fix bugs" → IA corrige
// 3. "Add tests" → IA escreve testes
// 4. "Refactor" → IA refatora

const ai = new NexusAI({
  apiKey: 'groq_...', // ou usa Ollama local
  model: 'qwen2.5-coder:3b',
});

await ai.magicClick({
  selection: editor.getSelection(),
  action: 'explain',
});
```

### 5. File System
```javascript
// Abre diálogo nativo do sistema
const file = await window.showOpenFilePicker();
const content = await file.getContent();

// Ou usa IndexedDB (offline)
await pocket.files.save('main.py', content);
const files = await pocket.files.list();
```

### 6. Templates
```javascript
// Templates pré-configurados
const templates = {
  'python-ai': {
    files: { 'main.py': '...', 'requirements.txt': '...' },
    ai: { provider: 'ollama', model: 'qwen2.5-coder:3b' },
  },
  'rust-cli': {
    files: { 'main.rs': '...', 'Cargo.toml': '...' },
    sandbox: { target: 'wasm32' },
  },
  'html-game': {
    files: { 'index.html': '...', 'style.css': '...', 'game.js': '...' },
  },
};
```

---

## API

### Main API
```typescript
class PocketIDE {
  constructor(options?: PocketOptions);

  // Editor
  editor: MonacoEditor;
  terminal: XTerm;

  // File operations
  files: FileSystem;

  // Code execution
  run(options: RunOptions): Promise<RunResult>;

  // AI
  ai: NexusAI;

  // Templates
  loadTemplate(name: string): Promise<void>;
  saveTemplate(name: string): Promise<void>;

  // Events
  on(event: string, callback: Function): void;
  off(event: string, callback: Function): void;

  // Lifecycle
  destroy(): void;
}

interface PocketOptions {
  element?: string;           // Container selector
  language?: string;         // Default language
  theme?: 'vs-dark' | 'vs-light';
  fontSize?: number;
  terminal?: boolean;         // Show terminal
  ai?: boolean;              // Enable AI sidebar
}

interface RunOptions {
  language: string;
  code: string;
  stdin?: string;
  timeout?: number;
  target?: 'wasm' | 'native';
}

interface RunResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  duration: number;
}
```

### FileSystem API
```typescript
interface FileSystem {
  save(name: string, content: string): Promise<void>;
  load(name: string): Promise<string>;
  delete(name: string): Promise<void>;
  list(): Promise<FileInfo[]>;
  exists(name: string): Promise<boolean>;
  watch(callback: Function): void;
}

interface FileInfo {
  name: string;
  size: number;
  modified: Date;
  language: string;
}
```

### NexusAI API
```typescript
interface NexusAI {
  // Magic Click
  magicClick(options: MagicClickOptions): Promise<MagicClickResult>;

  // Chat
  chat(prompt: string): Promise<string>;
  chatStream(prompt: string): AsyncGenerator<string>;

  // Code actions
  explain(code: string): Promise<string>;
  fixBugs(code: string): Promise<FixResult>;
  addTests(code: string): Promise<string>;
  refactor(code: string): Promise<string>;

  // Model management
  setModel(model: string): void;
  setProvider(provider: 'groq' | 'ollama' | 'openai'): void;
}

interface MagicClickOptions {
  selection: Selection;
  action: 'explain' | 'fix' | 'test' | 'refactor' | 'optimize';
  context?: string;
}
```

---

## PERFORMANCE

### Targets
| Métrica | Target | Budget |
|---------|--------|--------|
| Initial load | < 2s | 3G |
| Time to interactive | < 3s | 3G |
| Bundle size | < 200KB | gzip |
| Memory usage | < 100MB | baseline |
| Editor input latency | < 16ms | 60fps |
| Terminal render | < 60fps | 60fps |

### Lazy Loading
```javascript
// Carrega módulos sob demanda
const pocket = await import('pocket-ide');

// Monaco carrega só quando abre editor
import('monaco-editor').then(monaco => {
  editor = monaco.editor.create(container, options);
});

// xterm carrega só quando abre terminal
import('xterm').then(xterm => {
  terminal = new xterm.Terminal(options);
});
```

### Service Worker (Offline)
```javascript
// Cache estratégias
const strategies = {
  // Cache first para assets
  '/pocket-ide.js': 'cache-first',
  '/monaco-editor/': 'stale-while-revalidate',

  // Network first para API
  '/api/ai': 'network-first',

  // Imutáveis pra sempre
  '/wasm/wasm3.wasm': 'cache-only',
};
```

---

## DESIGN

### Layouts

**Desktop (> 1024px)**
```
┌──────────────────────────────────────────────────────────────┐
│  TOOLBAR  │ File  Edit  View  Run  AI  │  │  ┌──────────┐│
├───────────┴────────────────────────────┴──┴──│ AI PANEL ││
│                                              │           ││
│                                              │ Magic     ││
│              MONACO EDITOR                  │ Click    ││
│              (resizable)                     │           ││
│                                              │ Explain   ││
│                                              │ Fix       ││
│──────────────────────────────────────────────│ Refactor  ││
│                                              │           ││
│              XTERM TERMINAL                  └───────────┘│
│              (collapsible)                                    │
└──────────────────────────────────────────────────────────────┘
```

**Mobile (< 768px)**
```
┌─────────────────────┐
│ ≡  │ Pocket IDE  │ ⚙️ │
├─────────────────────┤
│                     │
│    MONACO EDITOR    │
│    (touch keyboard) │
│                     │
├─────────────────────┤
│ 📁 │ ▶ │ 🔍 │ 🤖 │
├─────────────────────┤
│                     │
│    XTERM TERMINAL   │
│    (collapsible)    │
│                     │
└─────────────────────┘
```

### Temas
```javascript
const themes = {
  'vs-dark': {
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    accent: '#007acc',
  },
  'monokai': {
    background: '#272822',
    foreground: '#f8f8f2',
    accent: '#66d9ef',
  },
  'github-dark': {
    background: '#0d1117',
    foreground: '#c9d1d9',
    accent: '#58a6ff',
  },
};
```

### Cores (CSS Variables)
```css
:root {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #333333;
  --text-primary: #ffffff;
  --text-secondary: #cccccc;
  --accent: #0078d4;
  --accent-hover: #1084d9;
  --success: #4ec9b0;
  --warning: #dcdcaa;
  --error: #f14c4c;
  --border: #3c3c3c;
}
```

---

## COMPATIBILIDADE

### Browsers
| Browser | Versão | Suporte |
|---------|--------|---------|
| Chrome | 90+ | ✅ Full |
| Firefox | 90+ | ✅ Full |
| Safari | 15+ | ✅ Full |
| Edge | 90+ | ✅ Full |
| Mobile Safari | 15+ | ✅ Full |
| Chrome Android | 90+ | ✅ Full |

### Features
| Feature | Fallback | browsers |
|---------|----------|----------|
| File System Access API | IndexedDB | Safari, Firefox |
| WebAssembly | asm.js | IE11 (não suporta) |
| Service Workers | Cache API | Safari antigo |
| WebGPU | WebGL | Safari |
| SharedArrayBuffer | postMessage | Safari |

---

## INSTALAÇÃO

### Via CDN (1 comando)
```html
<script type="module">
  import PocketIDE from 'https://cdn.pocket-ide.dev/pocket-ide.js';
  const pocket = new PocketIDE({ element: '#app' });
</script>
```

### Via NPM
```bash
npm install pocket-ide
```

```javascript
import PocketIDE from 'pocket-ide';
const pocket = new PocketIDE({ element: '#app' });
```

### Via CLI (servidor local)
```bash
npx pocket-ide serve
# Abre http://localhost:3000
```

---

## EXEMPLOS

### Exemplo 1: Python AI Assistant
```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" href="pocket-ide.css">
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import PocketIDE from 'pocket-ide.js';

    const pocket = new PocketIDE({
      element: '#app',
      language: 'python',
      ai: true,
    });

    // Abre template de AI
    await pocket.loadTemplate('python-ai');

    // Executa código
    const result = await pocket.run({
      language: 'python',
      code: `import numpy as np
x = np.array([1, 2, 3])
print(x ** 2)`,
    });

    console.log(result.stdout); // [1 4 9]
  </script>
</body>
</html>
```

### Exemplo 2: JavaScript Game
```javascript
// Cria jogo no browser
const pocket = new PocketIDE({ element: '#game' });

await pocket.loadTemplate('html-game');

// O código é executado no sandbox
await pocket.run({
  language: 'javascript',
  code: `
const canvas = document.createElement('canvas');
canvas.width = 400;
canvas.height = 300;
document.body.appendChild(canvas);
const ctx = canvas.getContext('2d');
ctx.fillStyle = 'red';
ctx.fillRect(10, 10, 50, 50);
  `,
});
```

---

## ROADMAP

### v0.1 (MVP)
- [x] Monaco Editor
- [x] xterm.js Terminal
- [x] Python execution (WASM sandbox)
- [x] JavaScript execution (WASM sandbox)
- [x] Groq API integration

### v0.2
- [ ] Go execution
- [ ] Rust execution
- [ ] Magic Click (AI sidebar)
- [ ] File System Access API
- [ ] IndexedDB fallback

### v0.3
- [ ] Real-time collaboration (WebRTC)
- [ ] Multiple files tabs
- [ ] Git integration
- [ ] Debugger (breakpoints, step)
- [ ] Mobile optimizations

### v1.0
- [ ] Plugin system
- [ ] Custom sandbox policies
- [ ] Deploy to hosting
- [ ] Share/embed projects
- [ ] PWA (offline support)

---

## LICENÇA

**MIT** — Livre para uso, fork, modificação.

**Comercial**: Entre em contato para features enterprise (SSO, audit logs, SLA).

---

*Mão Santa do Código — Código que funciona em qualquer lugar.*
