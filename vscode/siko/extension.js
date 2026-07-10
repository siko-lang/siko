// Minimal hand-written LSP client for the siko compiler's `lsp` command.
// No dependencies: only the vscode host API and node builtins.
const vscode = require('vscode');
const cp = require('child_process');

function activate(context) {
    const folder = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
    if (!folder) {
        return;
    }
    const root = folder.uri.fsPath;
    const output = vscode.window.createOutputChannel('Siko');
    const collection = vscode.languages.createDiagnosticCollection('siko');
    context.subscriptions.push(output, collection);

    const command = vscode.workspace.getConfiguration('siko').get('lspCommand');
    const env = Object.assign({}, process.env);
    if (!env.SIKO_ROOT) {
        env.SIKO_ROOT = root;
    }
    if (!env.SIKO_TARGET_OS) {
        env.SIKO_TARGET_OS = process.platform === 'darwin' ? 'macos' : 'linux';
    }

    output.appendLine(`starting: ${command.join(' ')} (cwd: ${root})`);
    const server = cp.spawn(command[0], command.slice(1), { cwd: root, env });
    server.on('error', (err) => output.appendLine(`failed to start siko lsp: ${err.message}`));
    server.on('exit', (code, signal) => output.appendLine(`siko lsp exited (code: ${code}, signal: ${signal})`));
    server.stderr.on('data', (data) => output.append(data.toString()));

    let nextId = 1;
    function send(message) {
        const body = JSON.stringify(Object.assign({ jsonrpc: '2.0' }, message));
        server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
    }
    function request(method, params) {
        send({ id: nextId++, method, params });
    }
    function notify(method, params) {
        send({ method, params });
    }

    const severities = {
        1: vscode.DiagnosticSeverity.Error,
        2: vscode.DiagnosticSeverity.Warning,
        3: vscode.DiagnosticSeverity.Information,
        4: vscode.DiagnosticSeverity.Hint,
    };
    function handle(message) {
        if (message.method === 'textDocument/publishDiagnostics') {
            const { uri, diagnostics } = message.params;
            collection.set(vscode.Uri.parse(uri), diagnostics.map((d) => {
                const diagnostic = new vscode.Diagnostic(
                    new vscode.Range(d.range.start.line, d.range.start.character,
                                     d.range.end.line, d.range.end.character),
                    d.message,
                    severities[d.severity] || vscode.DiagnosticSeverity.Error);
                diagnostic.source = d.source;
                return diagnostic;
            }));
        } else if (message.method === 'window/showMessage') {
            output.appendLine(message.params.message);
        }
    }

    // decode Content-Length framed messages from the server's stdout
    let buffered = Buffer.alloc(0);
    server.stdout.on('data', (chunk) => {
        buffered = Buffer.concat([buffered, chunk]);
        for (;;) {
            const headerEnd = buffered.indexOf('\r\n\r\n');
            if (headerEnd < 0) {
                return;
            }
            const match = /Content-Length: *(\d+)/i.exec(buffered.slice(0, headerEnd).toString());
            if (!match) {
                buffered = buffered.slice(headerEnd + 4);
                continue;
            }
            const length = parseInt(match[1], 10);
            if (buffered.length < headerEnd + 4 + length) {
                return;
            }
            const body = buffered.slice(headerEnd + 4, headerEnd + 4 + length).toString();
            buffered = buffered.slice(headerEnd + 4 + length);
            try {
                handle(JSON.parse(body));
            } catch (err) {
                output.appendLine(`bad message from server: ${err.message}`);
            }
        }
    });

    request('initialize', {
        processId: process.pid,
        rootUri: vscode.Uri.file(root).toString(),
        capabilities: {},
    });
    notify('initialized', {});

    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument((document) => {
        if (document.languageId === 'siko') {
            notify('textDocument/didSave', { textDocument: { uri: document.uri.toString() } });
        }
    }));

    context.subscriptions.push({
        dispose() {
            try {
                notify('exit', {});
                server.kill();
            } catch (err) {
                // the server is already gone
            }
        },
    });
}

function deactivate() {}

module.exports = { activate, deactivate };
