# GitHub Pages Documentation

This directory contains the source files for the project's GitHub Pages site, which combines a brochure-style landing page with automatically generated API documentation.

## Structure

```
docs/
├── index.html          # Main landing page
├── style.css           # Styles for the brochure site
└── README.md           # This file
```

When deployed, the site structure will be:

```
/                       # Landing page (index.html)
/api/                   # Rustdoc API documentation
/api/timetable_core/    # Core library documentation
/api/timetable_cli/     # CLI binary documentation
```

## Deployment

The site is automatically deployed via GitHub Actions on every push to the `main` branch. The workflow:

1. Builds the rustdoc documentation for all workspace crates
2. Copies the brochure site files (index.html, style.css) to the root
3. Copies rustdoc output to `/api/`
4. Deploys everything to GitHub Pages

See `.github/workflows/pages.yml` for the complete workflow configuration.

## Enabling GitHub Pages

To enable GitHub Pages for this repository:

1. Go to **Settings** → **Pages** in the GitHub repository
2. Under **Source**, select:
   - Source: **GitHub Actions**
3. Save the settings
4. Push to `main` branch to trigger the first deployment
5. The site will be available at: `https://richardslater.github.io/bromcom-timetable-formatter/`

## Local Development

To preview the site locally:

```bash
# Option 1: Simple HTTP server with Python
cd docs
python3 -m http.server 8000

# Option 2: Using npx
cd docs
npx serve

# Then open http://localhost:8000 in your browser
```

Note: The API documentation links will not work locally unless you build the rustdoc separately:

```bash
# Build rustdoc
cargo doc --no-deps --workspace --release

# Copy to docs for local preview
mkdir -p docs/api
cp -r target/doc/* docs/api/
```

## Updating the Landing Page

To update the brochure site:

1. Edit `docs/index.html` for content changes
2. Edit `docs/style.css` for styling changes
3. Commit and push to `main`
4. GitHub Actions will automatically rebuild and deploy

## Updating API Documentation

API documentation is automatically generated from Rust doc comments in the source code:

1. Update doc comments in `crates/core/src/*.rs` or `crates/cli/src/*.rs`
2. Commit and push to `main`
3. GitHub Actions will rebuild rustdoc and deploy

## Custom Domain (Optional)

To use a custom domain:

1. Add a `CNAME` file to the `docs/` directory with your domain name
2. Configure DNS records for your domain to point to GitHub Pages
3. Enable HTTPS in repository settings

See [GitHub Pages documentation](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site) for details.
