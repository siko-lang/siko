(function () {
    'use strict';

    const KEYWORDS = new Set([
        'fn', 'let', 'match', 'if', 'else', 'return', 'for', 'while',
        'loop', 'break', 'continue', 'in', 'pub', 'struct', 'enum',
        'trait', 'instance', 'import', 'module', 'effect', 'implicit',
        'with', 'declare', 'as', 'type', 'auto',
    ]);

    function esc(s) {
        return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    function span(cls, text) {
        return '<span class="' + cls + '">' + esc(text) + '</span>';
    }

    function tokenize(src) {
        let out = '';
        let i = 0;
        const n = src.length;

        while (i < n) {
            const ch = src[i];

            // Line comment
            if (ch === '/' && src[i + 1] === '/') {
                let j = i;
                while (j < n && src[j] !== '\n') j++;
                out += span('sk-comment', src.slice(i, j));
                i = j;
                continue;
            }

            // String literal — treat ${} interpolation as plain content
            if (ch === '"') {
                let j = i + 1;
                while (j < n && src[j] !== '"') {
                    if (src[j] === '\\') j++;
                    j++;
                }
                out += span('sk-string', src.slice(i, j + 1));
                i = j + 1;
                continue;
            }

            // Char literal: 'x' or '\x'
            if (ch === "'") {
                const len = src[i + 1] === '\\' ? 4 : 3;
                if (i + len <= n && src[i + len - 1] === "'") {
                    out += span('sk-string', src.slice(i, i + len));
                    i += len;
                    continue;
                }
            }

            // Annotation: @identifier
            if (ch === '@') {
                let j = i + 1;
                while (j < n && /\w/.test(src[j])) j++;
                out += span('sk-annotation', src.slice(i, j));
                i = j;
                continue;
            }

            // Number
            if (ch >= '0' && ch <= '9') {
                let j = i;
                while (j < n && ((src[j] >= '0' && src[j] <= '9') || src[j] === '_')) j++;
                out += span('sk-number', src.slice(i, j));
                i = j;
                continue;
            }

            // Identifier, keyword, type, or constructor
            if (/[a-zA-Z_]/.test(ch)) {
                let j = i;
                while (j < n && /\w/.test(src[j])) j++;
                const word = src.slice(i, j);
                if (KEYWORDS.has(word)) {
                    out += span('sk-kw', word);
                } else if (word[0] >= 'A' && word[0] <= 'Z') {
                    out += span('sk-ctor', word);
                } else {
                    out += span('sk-ident', word);
                }
                i = j;
                continue;
            }

            // Everything else — escape and emit one character
            out += esc(ch);
            i++;
        }

        return out;
    }

    const css =
        'pre > code                { color: #8ab5ea; }\n' +
        'pre > code .sk-ident      { color: #dee5f7; }\n' +
        'pre > code .sk-kw         { color: #89b7ef; font-weight: bold; }\n' +
        'pre > code .sk-ctor       { color: #dc88ed; }\n' +
        'pre > code .sk-string     { color: #64ba78; }\n' +
        'pre > code .sk-comment    { color: #6e7781; font-style: italic; }\n' +
        'pre > code .sk-number     { color: #e5dd6d; }\n' +
        'pre > code .sk-annotation { color: #953800; font-weight: bold; }\n';

    const style = document.createElement('style');
    style.textContent = css;
    document.head.appendChild(style);

    document.querySelectorAll('pre > code').forEach(function (block) {
        block.innerHTML = tokenize(block.textContent);
    });
})();
