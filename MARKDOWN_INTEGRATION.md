# Markdown Voice Commands Integration for Handy

This fork adds markdown voice command processing to Handy that converts dictated commands into formatted Markdown or Slack mrkdwn.

## ✅ Current Status

**Ready for testing and use!** Located at: `/Users/mike/dev/handy-markdown`

## Features

✅ **Voice-Controlled Formatting**: Say "markdown bold text markdown bold" to create **text**
✅ **Dual Format Support**: Choose between standard Markdown and Slack mrkdwn via keyboard shortcuts
✅ **Seamless Integration**: Works as a post-processing step after transcription
✅ **GUI Settings**: Configure via Settings > Advanced tab
✅ **Fully Tested**: Includes comprehensive Rust unit tests

## Supported Commands

Every command starts with the keyword **"markdown"**. Without it nothing is
formatted, so ordinary words like "bold" or "bullet" stay as plain text.
"markdown" on its own (or followed by a word that isn't a command) is also left
as text.

| Say                                   | Markdown shortcut                | Slack shortcut |
| ------------------------------------- | -------------------------------- | -------------- |
| "markdown bold … markdown bold"       | `**…**`                          | `*…*`          |
| "markdown italic … markdown italic"   | `*…*`                            | `_…_`          |
| "markdown header one / two / three"   | `#` / `##` / `###` on a new line | bold line      |
| "markdown bullet" (or "bullet point") | new line + `- `                  | same           |
| "markdown numbered bullet"            | new line + `1. `, `2. `…         | same           |
| "markdown new line" (or "newline")    | line break                       | same           |

**Forgiving matching:**

- Punctuation and capitals the speech model adds to the keyword or command are
  ignored: "Dark text. Markdown, bold." still closes the bold.
- "mark down" (two words) works as the keyword.
- Common mishearings are accepted, e.g. bald / bolt / bowled → bold,
  "header 1" → header one.
- Bold and italic toggle: the same command opens and closes. Anything left open
  is closed at the end of the line.

The accepted words live in one list at the top of
`src-tauri/src/audio_toolkit/markdown.rs`. Add new mishearings there as they
turn up.

## How to Use

### 1. Enable Markdown Processing

1. Open Handy Settings
2. Go to **Advanced** tab
3. Enable **"Markdown Processing"** toggle
4. Configure keyboard shortcuts (see below)

### 2. Configure Keyboard Shortcuts

In Settings > Advanced > Transcription section, you'll find two shortcuts:

- **Regular Transcribe Shortcut** (e.g., Ctrl+/): Uses **Markdown mode**
  - Produces `**bold**` and `*italic*`
  - Supports headers, bullets, numbered lists

- **Transcribe (Slack) Shortcut** (default Ctrl+.): Uses **Slack mode**
  - Produces `*bold*` and `_italic_`
  - Headers become a bold line (Slack has no headers)
  - Compatible with Slack's mrkdwn format

### 3. Dictate with Commands

**Example (Markdown mode - Ctrl+/):**

```
Dictate: "markdown header one Meeting Notes markdown bullet Action items markdown bold important markdown bold"
Result:  # Meeting Notes
         - Action items **important**
```

**Example (Slack mode - Ctrl+.):**

```
Dictate: "markdown bullet Action items markdown bold important markdown bold"
Result:  - Action items *important*
```

> [!IMPORTANT]
> **Slack:** after you paste, Slack may ask whether to apply the formatting.
> Click **Apply**. Do **not** click **"Don't ask again"**.

**Tip:** Press Command+Shift+F in Slack to toggle between plaintext and formatted view.

## Implementation Details

### Files

```
src-tauri/src/audio_toolkit/markdown.rs   - command parser + tests
src-tauri/src/audio_toolkit/mod.rs        - exports
src-tauri/src/settings.rs                 - markdown_processing_enabled + transcribe_slack binding
src-tauri/src/actions.rs                  - picks the mode from the shortcut, applies formatting
src-tauri/src/transcription_coordinator.rs - registers transcribe_slack as a transcribe shortcut
src-tauri/src/shortcut/mod.rs             - settings command
src-tauri/src/lib.rs                      - command registration
src/stores/settingsStore.ts               - toggle -> backend command
src/components/settings/                  - toggle + shortcut UI
src/i18n/locales/en/translation.json      - English strings
```

### Architecture

1. **Parser** (`src-tauri/src/audio_toolkit/markdown.rs`)
   - `MarkdownMode` enum (Markdown | Slack)
   - `process_markdown_commands()`: one pass over the words, looking for
     "markdown" + command word
   - Kept in its own file so upstream changes to `text.rs` don't conflict

2. **Settings** (`src-tauri/src/settings.rs`)
   - `markdown_processing_enabled: bool` - Master toggle
   - Two keyboard shortcuts: `transcribe` and `transcribe_slack`

3. **Actions** (`src-tauri/src/actions.rs`)
   - `TranscribeAction::stop` formats the text after transcription, covering
     both live-streamed and batch transcriptions
   - `transcribe` → Markdown mode, `transcribe_slack` → Slack mode

4. **UI Components**
   - `MarkdownProcessingToggle.tsx` - Enable/disable toggle
   - `ShortcutInput` component - Keyboard shortcut configuration
   - Located in Settings > Advanced > Transcription section

### Design Decisions

1. **Keyboard Shortcuts Over Dropdown**:
   - More intuitive - press different key combo for different format
   - No need to change settings between Slack and other apps
   - Muscle memory for each context

2. **Rust Implementation**:
   - Better performance (no IPC overhead)
   - Type safety
   - Integration with existing text processing pipeline

3. **Simplified Parser**:
   - Focused on common use cases
   - Toggle pattern detection with straightforward logic
   - Robust punctuation handling

4. **Markdown Mode by Default for Main Shortcut**:
   - Standard markdown is more universal
   - Slack users can use the dedicated shortcut

## Testing

### Run Unit Tests

```bash
cd src-tauri
cargo test --lib markdown
```

Covers bold/italic toggles, mishearings, ASR punctuation, headers, lists, line
breaks, keyword-only matching, and Slack vs Markdown output.

### Manual Testing

1. Build the app:

   ```bash
   bun install
   bun run tauri build --debug
   ```

2. Launch the app and enable Markdown Processing in Settings > Advanced

3. Test both shortcuts:
   - Press Ctrl+/ (or your transcribe shortcut): Dictate "markdown bold hello markdown bold" → "**hello**"
   - Press Ctrl+. (Slack shortcut): same dictation → "_hello_"

4. Test edge cases:
   - Punctuation: "markdown bold hello markdown bold." → "**hello**."
   - Multiple commands: "markdown bold one markdown bold markdown italic two markdown italic"
   - Headers: "markdown header one Title"
   - No keyword: "bold move" → stays "bold move"

## Bug Fixes

### Fixed Issues

- ✅ Punctuation duplication: "bold text bold." no longer produces "_text_.\*"
- ✅ Word-skipping in toggle patterns
- ✅ Case-insensitive command matching
- ✅ Word boundary detection at start/end of input

## Future Enhancements

Potential improvements for future versions:

- [ ] Strikethrough support (`~text~` in Slack)
- [ ] Code blocks (`` `code` ``)
- [ ] Links (`[text](url)`)
- [ ] Quote blocks
- [ ] More sophisticated nested formatting
- [ ] Custom command configuration
- [ ] Per-application mode selection

## Building & Distribution

### Local Build

```bash
cd /Users/mike/dev/handy-markdown
bun install
bun run tauri build
```

The app bundle will be at:

- macOS: `src-tauri/target/release/bundle/macos/HandyMarkdown.app`
- DMG: `src-tauri/target/release/bundle/dmg/HandyMarkdown_*.dmg`

### Creating a Pull Request

1. Fork the [Handy repository](https://github.com/cjpais/Handy)
2. Push your changes:

   ```bash
   git checkout -b feature/markdown-voice-commands
   git add -A
   git commit -m "feat: Add markdown voice command processing

   - Implements voice-to-markdown conversion with dual-mode support
   - Adds keyboard shortcuts for Markdown and Slack formatting
   - Includes GUI settings in Advanced tab
   - Comprehensive Rust unit tests
   - Bug fixes for punctuation handling"

   git push origin feature/markdown-voice-commands
   ```

3. Open PR with description linking to this document

## Troubleshooting

### Markdown commands aren't working

- Check that "Markdown Processing" is enabled in Settings > Advanced
- Verify you're using the correct shortcut (Ctrl+/ for Markdown, Ctrl+. for Slack)
- Check Handy logs for any errors

### Shortcuts conflict with other apps

- Customize the keyboard shortcuts in Settings > Advanced
- Click on the shortcut input and press your desired key combination

### Build fails

- On macOS, if `transcribe-cpp-sys` fails with `unsupported argument 'native' to option '-mcpu='`,
  build with `TRANSCRIBE_CMAKE_ARGS="-DGGML_NATIVE=OFF"` (upstream CI does the same).

1. Check Rust is installed: `rustc --version`
2. Check dependencies: `bun install`
3. Clean build: `cd src-tauri && cargo clean && cd .. && bun run tauri build`

### Tests fail

- Run individual test: `cargo test test_name`
- Check for breaking changes in dependencies
- See test output for specific assertion failures

## Original Parser Credit

Based on concepts from [dictation2markdown](https://github.com/yourusername/dictation2markdown) - reimplemented in Rust for Handy.

## License

Same as Handy - MIT License (check the main LICENSE file).

---

**Questions or Issues?**

- Open an issue in the Handy repository
- Or start a discussion in the Handy community

**Last Updated:** 2025 - Includes keyboard shortcut implementation and punctuation fixes
