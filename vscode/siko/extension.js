// Minimal hand-written LSP client for the siko compiler's `lsp` command.
// No dependencies: only the vscode host API and node builtins.
const vscode = require('vscode');
const cp = require('child_process');
const http = require('http');
const net = require('net');

function pickPort() {
    return new Promise((resolve, reject) => {
        const server = net.createServer();
        server.on('error', reject);
        server.listen(0, '127.0.0.1', () => {
            const port = server.address().port;
            server.close(() => resolve(port));
        });
    });
}

function postJson(port, path, payload) {
    const body = JSON.stringify(Object.assign({ jsonrpc: '2.0' }, payload));
    return new Promise((resolve, reject) => {
        const req = http.request({
            host: '127.0.0.1',
            port,
            path,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(body),
            },
        }, (res) => {
            const chunks = [];
            res.on('data', (chunk) => chunks.push(chunk));
            res.on('end', () => {
                const text = Buffer.concat(chunks).toString();
                if (res.statusCode < 200 || res.statusCode >= 300) {
                    reject(new Error(`HTTP ${res.statusCode}: ${text}`));
                    return;
                }
                try {
                    const messages = JSON.parse(text);
                    resolve(Array.isArray(messages) ? messages : []);
                } catch (err) {
                    reject(new Error(`bad JSON response: ${err.message}`));
                }
            });
        });
        req.on('error', reject);
        req.write(body);
        req.end();
    });
}

function waitForServer(port, child) {
    const deadline = Date.now() + 5000;
    return new Promise((resolve, reject) => {
        function attempt() {
            if (child.exitCode !== null) {
                reject(new Error(`siko lsp exited before HTTP startup (code: ${child.exitCode})`));
                return;
            }
            const req = http.request({ host: '127.0.0.1', port, path: '/health', method: 'GET' }, (res) => {
                res.resume();
                if (res.statusCode === 200) {
                    resolve();
                } else if (Date.now() >= deadline) {
                    reject(new Error(`siko lsp health check returned HTTP ${res.statusCode}`));
                } else {
                    setTimeout(attempt, 50);
                }
            });
            req.on('error', () => {
                if (Date.now() >= deadline) {
                    reject(new Error('timed out waiting for siko lsp HTTP server'));
                } else {
                    setTimeout(attempt, 50);
                }
            });
            req.end();
        }
        attempt();
    });
}

async function activate(context) {
    const folder = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
    if (!folder) {
        return;
    }
    const root = folder.uri.fsPath;
    const output = vscode.window.createOutputChannel('Siko');
    const collection = vscode.languages.createDiagnosticCollection('siko');
    context.subscriptions.push(output, collection);

    const command = vscode.workspace.getConfiguration('siko').get('lspCommand');
    const port = await pickPort();
    const env = Object.assign({}, process.env);
    if (!env.SIKO_ROOT) {
        env.SIKO_ROOT = root;
    }
    if (!env.SIKO_TARGET_OS) {
        env.SIKO_TARGET_OS = process.platform === 'darwin' ? 'macos' : 'linux';
    }
    env.SIKO_LSP_PORT = String(port);

    output.appendLine(`starting: ${command.join(' ')} (cwd: ${root}, port: ${port})`);
    const server = cp.spawn(command[0], command.slice(1), { cwd: root, env });
    server.on('error', (err) => output.appendLine(`failed to start siko lsp: ${err.message}`));
    server.on('exit', (code, signal) => output.appendLine(`siko lsp exited (code: ${code}, signal: ${signal})`));
    server.stdout.on('data', (data) => output.append(data.toString()));
    server.stderr.on('data', (data) => output.append(data.toString()));

    let nextId = 1;
    const ready = waitForServer(port, server);
    let latestDiagnosticsRun = 0;
    function send(message, options) {
        const diagnosticsRun = options && options.clearDiagnostics ? ++latestDiagnosticsRun : 0;
        return ready.then(() => postJson(port, '/rpc', message))
            .then((messages) => {
                if (diagnosticsRun && diagnosticsRun !== latestDiagnosticsRun) {
                    return;
                }
                if (diagnosticsRun) {
                    collection.clear();
                }
                messages.forEach(handle);
            })
            .catch((err) => {
                output.appendLine(`siko lsp request failed: ${err.message}`);
            });
    }
    function request(method, params) {
        send({ id: nextId++, method, params });
    }
    function notify(method, params, options) {
        send({ method, params }, options);
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
        } else if (message.error) {
            output.appendLine(message.error.message || JSON.stringify(message.error));
        }
    }

    request('initialize', {
        processId: process.pid,
        rootUri: vscode.Uri.file(root).toString(),
        capabilities: {},
    });
    notify('initialized', {}, { clearDiagnostics: true });

    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument((document) => {
        if (document.languageId === 'siko') {
            notify('textDocument/didSave', { textDocument: { uri: document.uri.toString() } }, { clearDiagnostics: true });
        }
    }));

    context.subscriptions.push({
        dispose() {
            try {
                postJson(port, '/rpc', { method: 'exit' }).catch(() => {});
                server.kill();
            } catch (err) {
                // the server is already gone
            }
        },
    });
}

function deactivate() {}

module.exports = { activate, deactivate };
