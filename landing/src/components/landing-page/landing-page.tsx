import {
  component$,
  useSignal,
  useComputed$,
  useVisibleTask$,
  $,
} from "@builder.io/qwik";

// ── Fake dataset ────────────────────────────────────────────

const FILES = [
  { primary: "~/src/rsight/src/main.rs", meta: "rs · 2.1kb" },
  { primary: "~/src/rsight/src/search.rs", meta: "rs · 8.3kb" },
  { primary: "~/src/rsight/Cargo.toml", meta: "toml · 0.8kb" },
  { primary: "~/src/rsight/src/types.rs", meta: "rs · 3.1kb" },
  { primary: "~/src/rsight/src/ui/render.rs", meta: "rs · 5.7kb" },
  { primary: "~/src/rsight/src/app.rs", meta: "rs · 4.4kb" },
  { primary: "~/notes/project-ideas.md", meta: "md · 3.2kb" },
  { primary: "~/src/webapp/src/index.tsx", meta: "tsx · 6.2kb" },
  { primary: "~/.config/starship.toml", meta: "toml · 0.5kb" },
  { primary: "~/Documents/README.md", meta: "md · 0.3kb" },
];

const FOLDERS = [
  { primary: "~/src/rsight/", meta: "42 files" },
  { primary: "~/src/rsight/src/", meta: "18 files" },
  { primary: "~/notes/", meta: "23 files" },
  { primary: "~/src/webapp/", meta: "156 files" },
  { primary: "~/.config/", meta: "34 files" },
  { primary: "~/Documents/", meta: "89 files" },
];

const CONTENTS = [
  {
    primary: "~/src/rsight/src/search.rs:23",
    meta: "rs",
    snippet: "fn search_files(query: &str) -> Vec<Match> {",
  },
  {
    primary: "~/src/rsight/src/main.rs:1",
    meta: "rs",
    snippet: "fn main() -> anyhow::Result<()> {",
  },
  {
    primary: "~/notes/project-ideas.md:12",
    meta: "md",
    snippet: "## rsight — find anything in the $HOME directory",
  },
  {
    primary: "~/src/rsight/src/types.rs:8",
    meta: "rs",
    snippet: "pub struct SearchQuery {",
  },
  {
    primary: "~/src/rsight/Cargo.toml:1",
    meta: "toml",
    snippet: 'name = "rsight"',
  },
  {
    primary: "~/src/rsight/src/app.rs:44",
    meta: "rs",
    snippet: "impl App {",
  },
];

const AI = [
  { primary: "rsight TUI architecture · Mar 18", meta: "Claude Code" },
  { primary: "Rust error handling patterns · Mar 15", meta: "Claude Code" },
  { primary: "Search algorithm optimization · Mar 10", meta: "Cursor" },
  { primary: "TUI rendering with ratatui · Mar 8", meta: "Claude Code" },
  { primary: "Async search with tokio · Mar 3", meta: "Cursor" },
  { primary: "Refactor content search pipeline · Mar 1", meta: "Codex" },
];

const TABS = ["Files", "Folders", "Contents", "AI"];

const GITHUB_URL = "https://github.com/olucasandrade/rsight";

// ── Terminal Component ──────────────────────────────────────

const InteractiveTerminal = component$(() => {
  const query = useSignal("");
  const activeTab = useSignal(0);
  const selectedIdx = useSignal(0);

  const results = useComputed$(() => {
    const q = query.value.toLowerCase();
    const tab = activeTab.value;

    let pool: Array<{ primary: string; meta: string; snippet?: string }>;
    if (tab === 0) pool = FILES;
    else if (tab === 1) pool = FOLDERS;
    else if (tab === 2) pool = CONTENTS;
    else pool = AI;

    const filtered = q
      ? pool.filter(
          (r) =>
            r.primary.toLowerCase().includes(q) ||
            (r.snippet && r.snippet.toLowerCase().includes(q)),
        )
      : pool.slice(0, 6);

    return filtered.slice(0, 8);
  });

  const handleKeyDown = $((e: KeyboardEvent) => {
    const len = results.value.length;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIdx.value = Math.min(selectedIdx.value + 1, len - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIdx.value = Math.max(selectedIdx.value - 1, 0);
    } else if (e.key === "Tab") {
      e.preventDefault();
      activeTab.value = (activeTab.value + 1) % 4;
      selectedIdx.value = 0;
    }
  });

  useVisibleTask$(({ track }) => {
    track(() => activeTab.value);
    selectedIdx.value = 0;
  });

  return (
    <div class="terminal-wrap">
      <div class="terminal" role="application" aria-label="rsight TUI demo">
        {/* chrome */}
        <div class="term-chrome">
          <div class="term-chrome-title">
            <strong>rsight</strong>
            <span>─ terminal search</span>
          </div>
          <span class="term-chrome-quit">^C quit</span>
        </div>

        {/* search bar */}
        <div class="term-search">
          <span class="term-prompt">❯</span>
          <input
            class="term-input"
            type="text"
            value={query.value}
            onInput$={(e) => {
              query.value = (e.target as HTMLInputElement).value;
              selectedIdx.value = 0;
            }}
            onKeyDown$={handleKeyDown}
            placeholder="search $HOME directory"
            autoComplete="off"
            spellcheck={false}
            aria-label="Search query"
          />
          <span class="term-hint">Tab ⇥</span>
        </div>

        {/* tabs */}
        <div class="term-tabs">
          {TABS.map((tab, i) => (
            <button
              key={tab}
              class={activeTab.value === i ? "term-tab active" : "term-tab"}
              onClick$={() => {
                activeTab.value = i;
                selectedIdx.value = 0;
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* results */}
        <div class="term-results">
          {results.value.length === 0 ? (
            <div class="term-empty">no results for &quot;{query.value}&quot;</div>
          ) : (
            results.value.map((r, i) => (
              <div key={r.primary}>
                <div
                  class={
                    selectedIdx.value === i
                      ? "term-result selected"
                      : "term-result"
                  }
                  onClick$={() => (selectedIdx.value = i)}
                >
                  <span class="term-result-primary">{r.primary}</span>
                  <span class="term-result-meta">{r.meta}</span>
                </div>
                {r.snippet && (
                  <div
                    class={
                      selectedIdx.value === i
                        ? "term-result-snippet selected"
                        : "term-result-snippet"
                    }
                  >
                    {r.snippet}
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        {/* status bar */}
        <div class="term-status">
          <span>
            {results.value.length} result
            {results.value.length !== 1 ? "s" : ""}
          </span>
          <div class="term-status-keys">
            <span>↑↓ navigate</span>
            <span>↵ open</span>
            <span>Tab switch</span>
          </div>
        </div>
      </div>

      <div
        style="margin-top: 8px; font-size: 0.72rem; color: var(--dim); text-align: right;"
      >
        click to interact · type to search
      </div>
    </div>
  );
});

// ── Copy button ─────────────────────────────────────────────

const CopyBtn = component$(({ text }: { text: string }) => {
  const copied = useSignal(false);

  return (
    <button
      class={copied.value ? "copy-btn copied" : "copy-btn"}
      onClick$={async () => {
        await navigator.clipboard.writeText(text);
        copied.value = true;
        setTimeout(() => (copied.value = false), 1800);
      }}
    >
      {copied.value ? "copied!" : "copy"}
    </button>
  );
});

// ── Install Tabs ────────────────────────────────────────────

const INSTALL_METHODS = [
  {
    id: "brew",
    label: "Homebrew",
    recommended: true,
    lines: [
      { type: "cmd", text: "brew tap olucasandrade/rsight https://github.com/olucasandrade/rsight" },
      { type: "cmd", text: "brew install rsight" },
      { type: "out", text: "==> Downloading rsight-0.2.0.tar.gz" },
      { type: "out", text: "==> Installing rsight" },
      { type: "out", text: "==> Installed rsight" },
      { type: "gap" },
      { type: "cmd", text: "rsight" },
    ],
    copyText: "brew tap olucasandrade/rsight https://github.com/olucasandrade/rsight && brew install rsight",
    note: "Recommended. No Rust toolchain required.",
  },
  {
    id: "script",
    label: "Install script",
    recommended: false,
    lines: [
      {
        type: "cmd",
        text: "curl -fsSL https://raw.githubusercontent.com/olucasandrade/rsight/main/install.sh | bash",
      },
      { type: "out", text: "Installing rsight v0.2.0 for arm64..." },
      { type: "out", text: "Installed to ~/.local/bin/rsight" },
      { type: "gap" },
      { type: "cmd", text: "rsight" },
    ],
    copyText:
      "curl -fsSL https://raw.githubusercontent.com/olucasandrade/rsight/main/install.sh | bash",
    note: "Downloads the latest release binary. No Rust required. Installs to ~/.local/bin by default.",
  },
  {
    id: "cargo",
    label: "Cargo",
    recommended: false,
    lines: [
      { type: "cmd", text: "cargo install rsight" },
      { type: "out", text: "Compiling rsight v0.2.0 ..." },
      { type: "out", text: "Finished in 18.4s" },
      { type: "out", text: "Installed ~/.cargo/bin/rsight" },
      { type: "gap" },
      { type: "cmd", text: "rsight" },
    ],
    copyText: "cargo install rsight",
    note: "Requires Rust 1.70+.",
    noteHref: "https://rustup.rs",
    noteLinkText: "Install rustup ↗",
  },
  {
    id: "source",
    label: "Source",
    recommended: false,
    lines: [
      { type: "cmd", text: "git clone github.com/olucasandrade/rsight" },
      { type: "cmd", text: "cd rsight && cargo build --release" },
      { type: "out", text: "Compiling rsight v0.2.0 ..." },
      { type: "out", text: "Finished release [optimized]" },
      { type: "gap" },
      { type: "cmd", text: "./target/release/rsight" },
    ],
    copyText:
      "git clone https://github.com/olucasandrade/rsight && cd rsight && cargo build --release",
    note: "Requires Rust 1.70+. Builds from latest commits.",
    noteHref: "https://rustup.rs",
    noteLinkText: "Install rustup ↗",
  },
];

const InstallTabs = component$(() => {
  const active = useSignal(0);
  const method = useComputed$(() => INSTALL_METHODS[active.value]);

  return (
    <div class="itabs">
      {/* method selector */}
      <div class="itabs-nav">
        {INSTALL_METHODS.map((m, i) => (
          <button
            key={m.id}
            class={active.value === i ? "itab active" : "itab"}
            onClick$={() => (active.value = i)}
          >
            {m.label}
            {m.recommended && <span class="itab-badge">recommended</span>}
          </button>
        ))}
      </div>

      {/* shell block */}
      <div class="itabs-body">
        <div class="install-block">
          <div class="install-block-header">
            <span>shell</span>
            <CopyBtn text={method.value.copyText} />
          </div>
          <div class="install-block-body">
            {method.value.lines.map((line, i) =>
              line.type === "gap" ? (
                <div key={i} style="height: 8px;" />
              ) : line.type === "cmd" ? (
                <div key={i}>
                  <span class="prompt">$ </span>
                  <span class="cmd">{line.text}</span>
                </div>
              ) : (
                <div key={i} class="out">
                  {line.text}
                </div>
              ),
            )}
          </div>
        </div>

        {/* note */}
        <p class="itabs-note">
          {method.value.note}
          {method.value.noteHref && (
            <>
              {" "}
              <a
                href={method.value.noteHref}
                target="_blank"
                rel="noopener noreferrer"
                style="text-decoration: underline; text-underline-offset: 3px;"
              >
                {method.value.noteLinkText}
              </a>
            </>
          )}
        </p>
      </div>

      {/* after-install steps */}
      <div class="post-install">
        <div class="post-install-step">
          <span class="post-num">01</span>
          <div>
            <div class="post-title">Launch</div>
            <div class="post-desc">
              Run <code>rsight</code> from any terminal.
            </div>
          </div>
        </div>
        <div class="post-install-step">
          <span class="post-num">02</span>
          <div>
            <div class="post-title">Navigate</div>
            <div class="post-desc">
              <code>Tab</code> switches modes. <code>↑↓</code> moves.{" "}
              <code>↵</code> opens.
            </div>
          </div>
        </div>
        <div class="post-install-step">
          <span class="post-num">03</span>
          <div>
            <div class="post-title">Bind a shortcut</div>
            <div class="post-desc">
              Map <code>rsight</code> to a global hotkey for instant access.
            </div>
          </div>
        </div>
        <div class="post-install-step">
          <span class="post-num">04</span>
          <div>
            <div class="post-title">Set your editor</div>
            <div class="post-desc">
              rsight respects <code>$EDITOR</code> to open files at the right
              line.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});

// ── Main Landing Page ───────────────────────────────────────

export const LandingPage = component$(() => {
  return (
    <div class="page">
      <div class="wrap">
        {/* ── Nav ── */}
        <header class="nav">
          <div class="nav-logo">
            <img src="/logo.png" alt="rsight logo" class="nav-logo-img" />
            <span>rsight</span>
            <span class="nav-version">v0.2.0</span>
          </div>
          <div class="nav-right">
            <a href="#install" class="nav-link">
              install
            </a>
            <a href="#contribute" class="nav-link">
              contribute
            </a>
            <a
              href={GITHUB_URL}
              class="gh-btn"
              target="_blank"
              rel="noopener noreferrer"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
              </svg>
              GitHub
            </a>
          </div>
        </header>

        {/* ── Hero ── */}
        <section class="hero">
          <div class="hero-text">
            <div class="hero-tag">terminal-first · open source · rust · macos · <span class="tag-soon">linux soon</span></div>
            <h1 class="hero-h1">
              Find anything
              <br />
              $HOME directory.
            </h1>
            <p class="hero-sub">
              One TUI. Files, folders, content, and AI conversations searched
              locally in under a second.
            </p>

            <div class="hero-install">
              <span class="prompt">$</span>
              <code>brew install rsight</code>
              <CopyBtn text="brew tap olucasandrade/rsight https://github.com/olucasandrade/rsight && brew install rsight" />
            </div>
            <p class="hero-meta">
              macOS · <span class="tag-soon">linux soon</span> ·{" "}
              <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
                source on GitHub ↗
              </a>
            </p>
          </div>

          <InteractiveTerminal />
        </section>

        {/* ── Features ── */}
        <section class="section" id="features">
          <div class="section-label">features</div>

          <div class="features-grid">
            <div class="feature">
              <div class="feature-key">01</div>
              <div class="feature-title">Files</div>
              <p class="feature-desc">
                Case-insensitive file path search across your home directory.
                Opens instantly in $EDITOR.
              </p>
              <span class="feature-tag">paths over previews</span>
            </div>
            <div class="feature">
              <div class="feature-key">02</div>
              <div class="feature-title">Folders</div>
              <p class="feature-desc">
                Navigate project structure at a glance. Jump to any directory in
                Finder or your shell.
              </p>
              <span class="feature-tag">open in finder</span>
            </div>
            <div class="feature">
              <div class="feature-key">03</div>
              <div class="feature-title">Contents</div>
              <p class="feature-desc">
                Search inside files — code, docs, config — without any
                background indexer.
              </p>
              <span class="feature-tag">streamed</span>
            </div>
            <div class="feature">
              <div class="feature-key">04</div>
              <div class="feature-title">AI Sessions</div>
              <p class="feature-desc">
                Resume Claude Code, Cursor, and Codex conversations exactly
                where they left off.
              </p>
              <span class="feature-tag">context restored</span>
            </div>
          </div>

          <div class="stats-row">
            <div class="stat-item">
              <div class="stat-val">&lt; 1s</div>
              <div class="stat-desc">results on a typical dev machine</div>
            </div>
            <div class="stat-item">
              <div class="stat-val">0 daemon</div>
              <div class="stat-desc">scan on demand, low memory footprint</div>
            </div>
            <div class="stat-item">
              <div class="stat-val">100% local</div>
              <div class="stat-desc">here we value local-first :)</div>
            </div>
          </div>
        </section>

        {/* ── Install ── */}
        <section class="section" id="install">
          <div class="section-label">install</div>
          <InstallTabs />
        </section>

        {/* ── Contribute ── */}
        <section class="section" id="contribute">
          <div class="section-label">contribute</div>

          <div class="contribute-grid">
            <div class="contribute-text">
              <h2>Built in the open.</h2>
              <p>
                rsight is MIT-licensed and actively developed. Bug reports,
                feature ideas, and pull requests are all welcome. If you find it
                useful, a star helps others discover it.
              </p>

              <div class="contribute-links">
                <a
                  href={GITHUB_URL}
                  class="contribute-link"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
                  </svg>
                  View on GitHub
                  <span class="link-arrow">↗</span>
                </a>
                <a
                  href={`${GITHUB_URL}/issues`}
                  class="contribute-link"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Open an issue
                  <span class="link-arrow">↗</span>
                </a>
                <a
                  href={`${GITHUB_URL}/pulls`}
                  class="contribute-link"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Submit a pull request
                  <span class="link-arrow">↗</span>
                </a>
              </div>
            </div>

            <div class="contribute-steps">
              <div class="contribute-step">
                <span class="cs-num">1</span>
                <span>
                  <span class="cs-prompt">$ </span>
                  git clone github.com/rsight-dev/rsight
                </span>
              </div>
              <div class="contribute-step">
                <span class="cs-num">2</span>
                <span>
                  <span class="cs-prompt">$ </span>
                  cd rsight &amp;&amp; cargo build
                </span>
              </div>
              <div class="contribute-step">
                <span class="cs-num">3</span>
                <span>
                  <span class="cs-prompt">$ </span>
                  cargo test
                </span>
              </div>
              <div class="contribute-step">
                <span class="cs-num">4</span>
                <span>
                  <span class="cs-prompt">$ </span>
                  git checkout -b feat/your-feature
                </span>
              </div>
              <div class="contribute-step">
                <span class="cs-num">5</span>
                <span>open a PR on GitHub ↗</span>
              </div>
            </div>
          </div>
        </section>

        {/* ── Footer ── */}
        <footer class="footer">
          <div class="footer-left">
            <span class="footer-logo">rsight</span>
            <span>MIT License · written in Rust</span>
          </div>
          <div class="footer-right">
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
            <a href="#install">Install</a>
            <a href="#contribute">Contribute</a>
          </div>
        </footer>
      </div>
    </div>
  );
});
