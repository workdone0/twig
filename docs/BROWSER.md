# Explore a file in your browser

Open [twig.wtf](https://twig.wtf/) to explore JSON, YAML, or HAR without installing
anything. Files are processed in your browser and are not uploaded. The browser
and terminal app use the same Rust engine, navigation keys, and Dark/Light themes.

## Open your data

Choose **Open file**, drop a file onto the explorer, or choose **Paste data**.
For pasted input, select JSON, YAML, or HAR before choosing **Explore data**.
The JSON, YAML, and HAR example buttons load bundled sample data.

One document is open per tab. Opening another file replaces it. **Close** clears
the current document; **Cancel** stops a load and clears it. Reloading or closing
the tab also clears your data. Twig does not modify your original file.

## Follow a branch

Each column lists the children of a container. Select a row to see its path and
source preview in the inspector. Select a container again, or press Enter, to
open its children in the next column. Use the breadcrumbs or Esc to go back.

The deepest columns that fit stay visible. On a narrow screen, Twig shows the
current column with the inspector below it. Long columns scroll independently;
large sibling lists have page controls.

| Action | Keys |
| --- | --- |
| Move up / down | `↑` / `↓`, `k` / `j` |
| Open a container | `→`, `l`, `Enter` |
| Return to parent | `←`, `h`, `Esc` |
| First / last sibling | `Home` / `End`, `g` / `G` |
| Search | `/` |
| Next / previous match | `n` / `N` |
| Jump to a path | `:` |
| Copy path / value | `c` / `y` |
| Switch Dark / Light | `t` |
| Show shortcuts | `?` |
| Close the document | `q` |

Shortcuts apply when you are outside a text field. Press Enter to submit a search
or path and return to navigation. Esc leaves text entry or closes Help before it
moves up a level. Browser shortcuts with Ctrl, Command, or Alt keep their usual behavior.

## Find a value

Press `/`, type part of a key or value, and press Enter. Search is a literal
substring match, not a regular expression: `%` and `_` are ordinary characters.
ASCII letters match without case sensitivity; other characters match exactly.
Use `n` and `N`, or the search arrows, to move through matches in document order.
Search wraps when it reaches either end.

If you know the path, press `:` and enter it. Examples:

```text
.users[0].name
.regions["us-east-1"]
.["a.b"]
```

Paths identify values; they are not jq expressions or JSONPath queries. YAML
uses an array of documents: `.[0].kind` selects `kind` in the first document.
For the first YAML document, `.kind` also works as a fallback.

## Copy a path or value

**Copy path** copies the full path, even when the visible label is shortened.
**Copy value** exports the complete selection within the limits below. Large integers keep their precision when displayed or copied. Original spacing, YAML comments, anchors, and quoting are not preserved.

The inspector is a preview. It may replace deeper or larger content with
placeholders. Copying text directly from that preview is not the same as using
**Copy value**.

If copying fails, check the browser's clipboard permissions and retry. A preview
can still be selected manually, but it may be incomplete. For large exports,
use the terminal app or the CLI's formatting commands.

## Limits

| Operation | Browser limit |
| --- | --- |
| File or pasted input | 20 MiB before parsing |
| Parsed document | 250,000 nodes and 128 MiB of estimated retained allocations |
| Children returned per page | 256 |
| Source preview | Four levels, 200 nodes, 30 children per container, 64K characters |
| Complete value export | 10,000 nodes and 4 MiB of text |

A node is an object, array, or scalar value. A small file with many values or
long paths can reach a node or allocation limit before it reaches 20 MiB.
The allocation estimate is not a browser memory ceiling: input buffers, parsing,
exports, and WebAssembly add overhead. Available device memory also matters.

When a limit is reached, choose a smaller file or [install the terminal app](../README.md#installation).
The terminal app has no browser input cap, but its memory and disk use still
depend on the data. Invalid JSON or unsupported YAML must be corrected before
loading; the browser has no editing, formatting, or repair mode.

## Privacy and session data

The website downloads its own HTML, styles, scripts, and WebAssembly assets.
Your file contents stay in the tab's worker and are not sent in network requests.
There are no accounts, analytics, third-party runtime scripts, or persistent
document caches. Only your Dark/Light preference is saved in browser local storage.

Closing or replacing a document terminates its worker. This clears Twig's live
document session; it is not a guarantee of secure memory erasure. Clipboard data
is managed by your browser and operating system. See [privacy and security](../SECURITY.md).

## Move between web and terminal

Install Twig, then open the same file with `twig path/to/file.json`. Navigation,
paths, search behavior, and theme colors will be familiar. Each interface keeps
its own session and theme preference; files and selections do not sync.

Use the terminal app for offline work and larger files, and CLI commands for
formatting and JSON repair. See the [CLI reference](../README.md#cli-reference).
