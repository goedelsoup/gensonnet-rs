# Gensonnet-rs Documentation Site

This directory contains the documentation site for Gensonnet-rs, built with [Zola](https://www.getzola.org/), a fast static site generator written in Rust.

## Structure

```
book/
├── config.toml          # Zola configuration
├── content/             # Markdown content
│   ├── _index.md        # Homepage
│   ├── getting-started/ # Getting started guide
│   ├── plugins/         # Plugin documentation
│   │   ├── _index.md    # Plugin overview
│   │   ├── external-plugins.md
│   │   ├── openapi-generator.md
│   │   └── go-ast-generator.md
│   ├── api/             # API reference
│   └── examples/        # Examples and tutorials
├── templates/           # HTML templates
│   ├── index.html       # Base template
│   └── page.html        # Page template
├── static/              # Static assets
│   ├── css/             # Stylesheets
│   └── js/              # JavaScript files
└── README.md           # This file
```

## Getting Started

### Prerequisites

1. Install Zola: https://www.getzola.org/documentation/getting-started/installation/
2. Make sure you're in the `book` directory

### Base URL Configuration

The site supports different base URLs for local development vs production. Choose one of these approaches:

#### Option 1: Environment Variables (Recommended)

The default configuration uses environment variables:

```bash
# For local development
export BASE_URL="http://127.0.0.1:1111"
zola serve

# For production build
export BASE_URL="https://goedelsoup.github.io/gensonnet-rs"
zola build
```

#### Option 2: Convenience Scripts

Use the provided scripts:

```bash
# Start development server
./dev.sh

# Build for production
./build.sh

# Switch between configurations
./switch-config.sh dev    # Switch to development
./switch-config.sh prod   # Switch to production
```

#### Option 3: Multiple Config Files

Copy the appropriate config file:

```bash
# For development
cp config.dev.toml config.toml

# For production
cp config.prod.toml config.toml
```

#### Option 4: Just Commands (Recommended)

Use the integrated justfile commands from the project root:

```bash
# Development server
just docs-dev

# Production build
just docs-build

# Switch configurations
just docs-switch-dev
just docs-switch-prod

# Other commands
just docs-clean
just docs-check
just docs-help
```

### Development

1. **Start the development server:**
   ```bash
   # Using just commands (recommended)
   just docs-dev
   
   # Using the convenience script
   ./dev.sh
   
   # Or manually
   export BASE_URL="http://127.0.0.1:1111"
   zola serve
   ```
   This will start a local server at `http://127.0.0.1:1111`

2. **Build the site:**
   ```bash
   # Using just commands (recommended)
   just docs-build
   
   # Using the convenience script
   ./build.sh
   
   # Or manually
   export BASE_URL="https://goedelsoup.github.io/gensonnet-rs"
   zola build
   ```
   This will generate the static site in the `public` directory

3. **Check the site:**
   ```bash
   zola check
   ```
   This will validate the site configuration and content

### Adding Content

1. **Create a new page:**
   ```bash
   # Create a new markdown file in content/
   touch content/my-new-page.md
   ```

2. **Add front matter to the markdown file:**
   ```markdown
   +++
   title = "My New Page"
   description = "Description of the page"
   weight = 10
   +++

   # My New Page

   Content goes here...
   ```

3. **Add to navigation:**
   Edit `config.toml` and add a menu item:
   ```toml
   [[menu.main]]
   name = "My New Page"
   url = "/my-new-page/"
   weight = 10
   ```

### Styling

- CSS files are in `static/css/`
- The main stylesheet is `main.css`
- Dark mode is supported via `prefers-color-scheme`

### JavaScript

- JavaScript files are in `static/js/`
- The main script is `main.js`
- Features include:
  - Smooth scrolling for anchor links
  - Table of contents highlighting
  - Code block copy functionality
  - Search functionality (when search index is available)
  - Mobile menu toggle

## Configuration

The main configuration is in `config.toml`. Key settings:

- `base_url`: The base URL for the site
- `title`: Site title
- `description`: Site description
- `build_search_index`: Enable search functionality
- `markdown.toc`: Enable table of contents generation

## Deployment

### GitHub Pages

The site is automatically deployed to GitHub Pages via GitHub Actions when you push to the `main` branch.

**Manual deployment:**
1. Build the site:
   ```bash
   just docs-build
   ```

2. Push the `public` directory to the `gh-pages` branch

### Netlify

1. Connect your repository to Netlify
2. Set build command: `cd book && export BASE_URL="https://your-site.netlify.app" && zola build`
3. Set publish directory: `book/public`

### Other Platforms

Zola generates a static site that can be deployed to any static hosting platform:
- Vercel
- Cloudflare Pages
- AWS S3
- And many more

## Customization

### Themes

The site uses a custom theme. To modify:

1. Edit templates in `templates/`
2. Update styles in `static/css/`
3. Modify JavaScript in `static/js/`

### Adding Features

- **Search**: Enable `build_search_index = true` in `config.toml`
- **Comments**: Add a commenting system like Disqus
- **Analytics**: Add Google Analytics or similar
- **RSS**: Enable `generate_feed = true` in `config.toml`

## Contributing

1. Make changes to the content in `content/`
2. Test locally with `zola serve`
3. Build and check with `zola build && zola check`
4. Submit a pull request

## Resources

- [Zola Documentation](https://www.getzola.org/documentation/)
- [Zola Themes](https://www.getzola.org/themes/)
- [Tera Templates](https://tera.netlify.app/) (Zola's template engine)
