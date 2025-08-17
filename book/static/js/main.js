// Main JavaScript file for Gensonnet-rs documentation site

document.addEventListener('DOMContentLoaded', function() {
    // Initialize smooth scrolling for anchor links
    initSmoothScrolling();
    
    // Initialize table of contents highlighting
    initTocHighlighting();
    
    // Initialize code block copy functionality
    initCodeCopy();
    
    // Initialize search functionality
    initSearch();
    
    // Initialize mobile menu toggle
    initMobileMenu();
});

// Smooth scrolling for anchor links
function initSmoothScrolling() {
    const links = document.querySelectorAll('a[href^="#"]');
    
    links.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            
            if (targetElement) {
                const headerHeight = document.querySelector('.site-header').offsetHeight;
                const targetPosition = targetElement.offsetTop - headerHeight - 20;
                
                window.scrollTo({
                    top: targetPosition,
                    behavior: 'smooth'
                });
                
                // Update URL without page jump
                history.pushState(null, null, this.getAttribute('href'));
            }
        });
    });
}

// Table of contents highlighting
function initTocHighlighting() {
    const tocLinks = document.querySelectorAll('.toc-link');
    const headers = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    
    if (tocLinks.length === 0 || headers.length === 0) return;
    
    const headerHeight = document.querySelector('.site-header').offsetHeight;
    
    function updateTocHighlight() {
        const scrollPosition = window.scrollY + headerHeight + 100;
        
        let currentHeader = null;
        
        headers.forEach(header => {
            const headerTop = header.offsetTop;
            const headerBottom = headerTop + header.offsetHeight;
            
            if (scrollPosition >= headerTop && scrollPosition < headerBottom) {
                currentHeader = header;
            }
        });
        
        // Remove active class from all TOC links
        tocLinks.forEach(link => {
            link.classList.remove('active');
        });
        
        // Add active class to current TOC link
        if (currentHeader) {
            const currentLink = document.querySelector(`.toc-link[href="#${currentHeader.id}"]`);
            if (currentLink) {
                currentLink.classList.add('active');
            }
        }
    }
    
    // Update on scroll
    window.addEventListener('scroll', throttle(updateTocHighlight, 100));
    
    // Initial update
    updateTocHighlight();
}

// Code block copy functionality
function initCodeCopy() {
    const codeBlocks = document.querySelectorAll('pre');
    
    codeBlocks.forEach(block => {
        // Create copy button
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-button';
        copyButton.innerHTML = 'Copy';
        copyButton.setAttribute('aria-label', 'Copy code to clipboard');
        
        // Position the button
        block.style.position = 'relative';
        copyButton.style.position = 'absolute';
        copyButton.style.top = '0.5rem';
        copyButton.style.right = '0.5rem';
        copyButton.style.padding = '0.25rem 0.5rem';
        copyButton.style.fontSize = '0.75rem';
        copyButton.style.backgroundColor = '#f6f8fa';
        copyButton.style.border = '1px solid #d1d5da';
        copyButton.style.borderRadius = '0.25rem';
        copyButton.style.cursor = 'pointer';
        copyButton.style.opacity = '0';
        copyButton.style.transition = 'opacity 0.2s';
        
        block.appendChild(copyButton);
        
        // Show button on hover
        block.addEventListener('mouseenter', () => {
            copyButton.style.opacity = '1';
        });
        
        block.addEventListener('mouseleave', () => {
            copyButton.style.opacity = '0';
        });
        
        // Copy functionality
        copyButton.addEventListener('click', async () => {
            const code = block.querySelector('code');
            const textToCopy = code ? code.textContent : block.textContent;
            
            try {
                await navigator.clipboard.writeText(textToCopy);
                copyButton.innerHTML = 'Copied!';
                copyButton.style.backgroundColor = '#28a745';
                copyButton.style.color = 'white';
                
                setTimeout(() => {
                    copyButton.innerHTML = 'Copy';
                    copyButton.style.backgroundColor = '#f6f8fa';
                    copyButton.style.color = 'inherit';
                }, 2000);
            } catch (err) {
                console.error('Failed to copy text: ', err);
                copyButton.innerHTML = 'Failed';
                copyButton.style.backgroundColor = '#dc3545';
                copyButton.style.color = 'white';
                
                setTimeout(() => {
                    copyButton.innerHTML = 'Copy';
                    copyButton.style.backgroundColor = '#f6f8fa';
                    copyButton.style.color = 'inherit';
                }, 2000);
            }
        });
    });
}

// Search functionality
function initSearch() {
    const searchInput = document.querySelector('.search-input');
    if (!searchInput) return;
    
    let searchIndex = null;
    
    // Load search index
    fetch('/search_index.json')
        .then(response => response.json())
        .then(data => {
            searchIndex = data;
        })
        .catch(err => {
            console.error('Failed to load search index:', err);
        });
    
    searchInput.addEventListener('input', throttle(function() {
        const query = this.value.toLowerCase().trim();
        
        if (query.length < 2) {
            hideSearchResults();
            return;
        }
        
        if (!searchIndex) {
            return;
        }
        
        const results = searchIndex.filter(item => {
            return item.title.toLowerCase().includes(query) ||
                   item.content.toLowerCase().includes(query);
        }).slice(0, 10);
        
        showSearchResults(results, query);
    }, 300));
    
    // Close search results when clicking outside
    document.addEventListener('click', function(e) {
        if (!e.target.closest('.search-container')) {
            hideSearchResults();
        }
    });
}

function showSearchResults(results, query) {
    let resultsContainer = document.querySelector('.search-results');
    
    if (!resultsContainer) {
        resultsContainer = document.createElement('div');
        resultsContainer.className = 'search-results';
        resultsContainer.style.position = 'absolute';
        resultsContainer.style.top = '100%';
        resultsContainer.style.left = '0';
        resultsContainer.style.right = '0';
        resultsContainer.style.backgroundColor = 'white';
        resultsContainer.style.border = '1px solid #d1d5da';
        resultsContainer.style.borderRadius = '0.375rem';
        resultsContainer.style.boxShadow = '0 4px 6px rgba(0, 0, 0, 0.1)';
        resultsContainer.style.zIndex = '1000';
        resultsContainer.style.maxHeight = '400px';
        resultsContainer.style.overflowY = 'auto';
        
        document.querySelector('.search-container').appendChild(resultsContainer);
    }
    
    if (results.length === 0) {
        resultsContainer.innerHTML = '<div class="search-result-item">No results found</div>';
        return;
    }
    
    resultsContainer.innerHTML = results.map(result => `
        <a href="${result.url}" class="search-result-item">
            <div class="search-result-title">${highlightText(result.title, query)}</div>
            <div class="search-result-excerpt">${highlightText(result.excerpt, query)}</div>
        </a>
    `).join('');
}

function hideSearchResults() {
    const resultsContainer = document.querySelector('.search-results');
    if (resultsContainer) {
        resultsContainer.remove();
    }
}

function highlightText(text, query) {
    const regex = new RegExp(`(${query})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
}

// Mobile menu toggle
function initMobileMenu() {
    const menuToggle = document.querySelector('.menu-toggle');
    const navbarMenu = document.querySelector('.navbar-menu');
    
    if (!menuToggle || !navbarMenu) return;
    
    menuToggle.addEventListener('click', function() {
        navbarMenu.classList.toggle('active');
        this.classList.toggle('active');
    });
    
    // Close menu when clicking on a link
    navbarMenu.addEventListener('click', function(e) {
        if (e.target.tagName === 'A') {
            navbarMenu.classList.remove('active');
            menuToggle.classList.remove('active');
        }
    });
}

// Utility function: throttle
function throttle(func, limit) {
    let inThrottle;
    return function() {
        const args = arguments;
        const context = this;
        if (!inThrottle) {
            func.apply(context, args);
            inThrottle = true;
            setTimeout(() => {
                inThrottle = false;
            }, limit);
        }
    };
}

// Add CSS for search results
const searchStyles = `
    .search-result-item {
        display: block;
        padding: 0.75rem 1rem;
        border-bottom: 1px solid #e1e5e9;
        text-decoration: none;
        color: inherit;
        transition: background-color 0.2s;
    }
    
    .search-result-item:hover {
        background-color: #f6f8fa;
        text-decoration: none;
    }
    
    .search-result-item:last-child {
        border-bottom: none;
    }
    
    .search-result-title {
        font-weight: 600;
        margin-bottom: 0.25rem;
    }
    
    .search-result-excerpt {
        font-size: 0.875rem;
        color: #586069;
        line-height: 1.4;
    }
    
    .search-result-item mark {
        background-color: #fff3cd;
        padding: 0.125rem 0.25rem;
        border-radius: 0.125rem;
    }
    
    .toc-link.active {
        color: #0366d6;
        font-weight: 600;
    }
    
    @media (prefers-color-scheme: dark) {
        .search-results {
            background-color: #161b22;
            border-color: #30363d;
        }
        
        .search-result-item:hover {
            background-color: #21262d;
        }
        
        .search-result-excerpt {
            color: #8b949e;
        }
        
        .search-result-item mark {
            background-color: #3c4043;
        }
        
        .toc-link.active {
            color: #58a6ff;
        }
    }
`;

// Inject search styles
const styleSheet = document.createElement('style');
styleSheet.textContent = searchStyles;
document.head.appendChild(styleSheet);
