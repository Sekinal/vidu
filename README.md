# vidu

A blazingly fast disk usage analyzer and cleaning utility for the terminal.

vidu combines the speed of parallel scanning with intelligent junk detection, duplicate finding, and safe cleaning capabilities. Think of it as ncdu with superpowers for disk cleanup.

## Features

### Disk Analysis
- **Parallel scanning** - Uses all CPU cores via rayon for fast directory traversal
- **Smart caching** - Compressed LZ4 cache for instant restarts on large directories
- **Real-time progress** - Live file/directory counts during scanning
- **Multiple sort modes** - Sort by size, name, modification time, or item count

### Cleaning Capabilities
- **Junk detection** - Auto-detects build artifacts, caches, and temporary files:
  - Build directories: `node_modules`, `target/`, `__pycache__`, `.next/`, `dist/`, `build/`
  - Cache files: `.cache/`, `*.tmp`, `*.swp`, `*~`
  - System junk: `.DS_Store`, `Thumbs.db`, `desktop.ini`
  - Log files: `*.log`, `logs/`

- **Duplicate finder** - Content-based detection using BLAKE3 hashing:
  - Multi-stage filtering: size → partial hash (4KB) → full hash
  - Identifies wasted space from duplicate files
  - Group duplicates for easy review

- **System cache detection** - Finds and measures:
  - Package managers: npm, cargo, pip, yarn, pnpm, go, composer
  - Browsers: Chrome, Firefox, Chromium, Edge
  - System: font caches, thumbnail caches, trash

- **Analysis views**:
  - File type breakdown by category (Documents, Images, Video, Code, etc.)
  - Age analysis to find old/unused files
  - Large file finder (top N biggest files)

### Safe Deletion
- **Trash support** - Cross-platform trash via the `trash` crate (default)
- **Permanent delete** - Optional direct deletion with confirmation
- **Batch operations** - Mark multiple items and clean at once
- **Confirmation dialogs** - Review before any destructive action

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/Sekinal/vidu.git
cd vidu

# Build and install
cargo install --path .
```

### Requirements
- Rust 1.70+ (uses edition 2021)
- A terminal with Unicode support

## Usage

```bash
# Analyze current directory
vidu

# Analyze a specific path
vidu /path/to/directory

# Force fresh scan (ignore cache)
vidu --fresh

# Show hidden files
vidu --hidden
```

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `←` / `h` / `Backspace` | Go to parent directory |
| `→` / `l` / `Enter` | Enter directory / Preview file |
| `g` / `Home` | Go to first item |
| `G` / `End` | Go to last item |
| `PgUp` / `PgDn` | Page up/down |
| `~` | Go to scan root |

### Actions
| Key | Action |
|-----|--------|
| `d` / `Delete` | Delete selected item |
| `Space` | Mark/unmark item for batch operations |
| `r` | Refresh current directory |
| `R` | Full rescan from root |
| `p` | Preview file contents |

### Cleaning
| Key | Action |
|-----|--------|
| `J` | Show junk analysis |
| `u` | Show duplicate files |
| `T` | Show file type breakdown |
| `o` | Show old/unused files |
| `L` | Show large files |
| `K` | Show system caches |
| `c` | Clean marked/selected items |
| `!` | Toggle deletion mode (Trash/Permanent) |

### View
| Key | Action |
|-----|--------|
| `s` | Cycle sort mode (size/name/modified/count) |
| `S` | Toggle sort order (asc/desc) |
| `.` | Toggle hidden files |
| `/` | Search in current directory |

### General
| Key | Action |
|-----|--------|
| `?` | Show/hide help |
| `q` / `Esc` | Quit / Go back / Close popup |

## Comparison with Other Tools

| Feature | vidu | ncdu | dust | dua |
|---------|------|------|------|-----|
| Parallel scanning | ✅ | ❌ | ✅ | ✅ |
| Caching | ✅ | ❌ | ❌ | ❌ |
| Junk detection | ✅ | ❌ | ❌ | ❌ |
| Duplicate finder | ✅ | ❌ | ❌ | ❌ |
| System cache detection | ✅ | ❌ | ❌ | ❌ |
| File type analysis | ✅ | ❌ | ❌ | ❌ |
| Age analysis | ✅ | ❌ | ❌ | ❌ |
| Safe trash deletion | ✅ | ❌ | ❌ | ❌ |
| Batch cleaning | ✅ | ❌ | ❌ | ❌ |
| Interactive TUI | ✅ | ✅ | ❌ | ✅ |
| Delete files | ✅ | ✅ | ❌ | ✅ |
| vim keybindings | ✅ | ❌ | N/A | ❌ |

## Architecture

```
src/
├── analyzer/       # Analysis modules
│   ├── age.rs          # File age analysis
│   ├── caches.rs       # System cache detection
│   ├── duplicates.rs   # Duplicate file finder
│   ├── file_types.rs   # File categorization
│   ├── hashing.rs      # BLAKE3 content hashing
│   ├── junk.rs         # Junk detection logic
│   ├── large_files.rs  # Large file finder
│   ├── patterns.rs     # Junk pattern definitions
│   └── presets.rs      # Cleaning presets
├── app/            # Application state & logic
│   ├── state.rs        # App state management
│   ├── actions.rs      # Action handlers
│   └── input.rs        # Keybinding definitions
├── cleaner/        # Deletion functionality
│   ├── safe_delete.rs  # Trash/permanent delete
│   └── batch.rs        # Batch cleaning operations
├── scanner/        # Directory scanning
│   ├── entry.rs        # Entry data structure
│   ├── scan.rs         # Parallel scanner
│   ├── preview.rs      # File preview
│   └── progress.rs     # Scan progress tracking
├── ui/             # Terminal UI
│   ├── render.rs       # Main render function
│   ├── theme.rs        # Color theme
│   └── components/     # UI components
├── cache.rs        # Scan result caching
├── config.rs       # Configuration
└── main.rs         # Entry point
```

## Configuration

vidu looks for a config file at `~/.config/vidu/config.toml`:

```toml
[general]
deletion_mode = "trash"  # "trash" or "permanent"
show_hidden = false

[junk_detection]
# Additional directories to treat as junk
custom_directories = ["vendor", ".bundle"]
# Additional file patterns to treat as junk
custom_files = ["*.bak", "*.orig"]
# Directories to never mark as junk
protected = [".git", ".ssh", ".gnupg"]
```

## Performance

vidu is designed for speed:

- **Parallel I/O** - Directory traversal uses rayon's work-stealing thread pool
- **Streaming hashing** - Large files are hashed in chunks, not loaded into memory
- **Multi-stage duplicate detection** - Size filter → 4KB partial hash → full hash
- **Compressed caching** - LZ4-compressed cache files for fast serialization
- **Lazy analysis** - Analysis views compute on-demand, not during scan

## Dependencies

Key dependencies:
- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal handling
- `rayon` - Parallel iteration
- `blake3` - Fast cryptographic hashing
- `trash` - Cross-platform trash support
- `serde` + `bincode` - Serialization for caching
- `lz4_flex` - Fast compression

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

Inspired by:
- [ncdu](https://dev.yorhel.nl/ncdu) - The classic disk usage analyzer
- [dust](https://github.com/bootandy/dust) - A more intuitive du
- [dua](https://github.com/Byron/dua-cli) - Disk Usage Analyzer with parallel traversal
