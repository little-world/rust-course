#!/bin/bash
# Generate PDF from Rust Patterns markdown files
# Requires: pandoc, weasyprint

set -e

# Check for required tools
check_deps() {
    if ! command -v pandoc &> /dev/null; then
        echo "pandoc not found. Install with: brew install pandoc"
        exit 1
    fi

    if ! command -v weasyprint &> /dev/null; then
        echo "weasyprint not found. Installing..."
        brew install weasyprint
    fi
}

# Create output directory
mkdir -p book/pdf

# Create CSS for PDF styling
cat > pdf-style.css << 'EOF'
/* Allow code blocks to break across pages */
pre, code {
    page-break-inside: auto !important;
    white-space: pre-wrap;
    word-wrap: break-word;
}

h1, h2, h3, h4, h5, h6 {
    page-break-after: avoid;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 11pt;
    line-height: 1.5;
}

pre {
    background-color: #f5f5f5;
    padding: 10px;
    border-radius: 4px;
    font-size: 9pt;
}

code {
    font-family: "SF Mono", Menlo, Monaco, monospace;
}

@page {
    margin: 1in;
    size: letter;
}
EOF

check_deps

echo "Generating PDF..."

cd src

pandoc \
    0*.md 1*.md 2*.md 3*.md \
    --pdf-engine=weasyprint \
    --css=../pdf-style.css \
    --toc \
    --toc-depth=2 \
    --highlight-style=tango \
    --metadata title="Rust Patterns" \
    -o ../book/pdf/rust-patterns.pdf

cd ..

echo "PDF generated: book/pdf/rust-patterns.pdf"
ls -lh book/pdf/rust-patterns.pdf
