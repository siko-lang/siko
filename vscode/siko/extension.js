// Minimal hand-written LSP client for the standalone siko-lsp server.
// No dependencies: only the vscode host API and node builtins.
const vscode = require('vscode');
const cp = require('child_process');
const http = require('http');
const net = require('net');
const path = require('path');

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
            if (child.exitCode !== null || child.signalCode !== null) {
                const reason = child.exitCode !== null ? `code: ${child.exitCode}` : `signal: ${child.signalCode}`;
                reject(new Error(`siko lsp exited before HTTP startup (${reason})`));
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
    const workspaceRoot = folder.uri.fsPath;
    const output = vscode.window.createOutputChannel('Siko');
    const collection = vscode.languages.createDiagnosticCollection('siko');
    context.subscriptions.push(output, collection);
    output.appendLine('activating Siko extension');

    const configuration = vscode.workspace.getConfiguration('siko', folder.uri);
    const configuredCommand = configuration.get('lspCommand');
    const command = Array.isArray(configuredCommand) && configuredCommand.length > 0
        ? configuredCommand
        : ['./siko-lsp.bin'];
    const resolvedCommand = command.slice();
    if (!path.isAbsolute(resolvedCommand[0]) &&
        (resolvedCommand[0].startsWith('.') || resolvedCommand[0].includes(path.sep))) {
        resolvedCommand[0] = path.resolve(workspaceRoot, resolvedCommand[0]);
    }
    const configuredProjectPath = configuration.get('projectPath', '');
    const root = configuredProjectPath ? path.resolve(workspaceRoot, configuredProjectPath) : workspaceRoot;
    output.appendLine(`workspace root: ${workspaceRoot}`);
    output.appendLine(`project root: ${root}${configuredProjectPath ? ' (configured)' : ' (workspace default)'}`);
    output.appendLine(`lsp command: ${resolvedCommand.join(' ')}`);
    output.appendLine('allocating local HTTP port');
    const port = await pickPort();
    output.appendLine(`allocated local HTTP port: ${port}`);
    const env = Object.assign({}, process.env);
    if (!env.SIKO_TARGET_OS) {
        env.SIKO_TARGET_OS = process.platform === 'darwin' ? 'macos' : 'linux';
    }
    env.SIKO_LSP_PORT = String(port);

    let nextId = 1;
    let latestDiagnosticsRun = 0;
    let server = null;
    let healthy = false;
    let disposed = false;
    let restartAttempt = 0;
    let restartTimer = null;
    let stableTimer = null;
    let readyWaiters = [];

    function settleReadyWaiters(error) {
        const waiters = readyWaiters;
        readyWaiters = [];
        waiters.forEach(({ resolve, reject }) => error ? reject(error) : resolve());
    }

    function waitUntilReady() {
        if (disposed) {
            return Promise.reject(new Error('siko lsp is shutting down'));
        }
        if (healthy) {
            return Promise.resolve();
        }
        return new Promise((resolve, reject) => readyWaiters.push({ resolve, reject }));
    }

    function applyMessages(messages, diagnosticsRun) {
        if (diagnosticsRun && diagnosticsRun !== latestDiagnosticsRun) {
            return;
        }
        if (diagnosticsRun) {
            collection.clear();
        }
        messages.forEach(handle);
    }

    async function initializeServer(child) {
        output.appendLine('waiting for siko lsp HTTP health check');
        await waitForServer(port, child);
        if (disposed || server !== child) {
            return;
        }

        output.appendLine('siko lsp health check passed; sending initialize');
        const initializeMessages = await postJson(port, '/rpc', {
            id: nextId++,
            method: 'initialize',
            params: {
                processId: process.pid,
                rootUri: vscode.Uri.file(root).toString(),
                capabilities: {},
            },
        });
        if (disposed || server !== child) {
            return;
        }
        initializeMessages.forEach(handle);

        output.appendLine('initialize completed; requesting initial project check');
        const diagnosticsRun = ++latestDiagnosticsRun;
        const initializedMessages = await postJson(port, '/rpc', { method: 'initialized', params: {} });
        if (disposed || server !== child) {
            return;
        }
        applyMessages(initializedMessages, diagnosticsRun);
        healthy = true;
        settleReadyWaiters();
        output.appendLine('siko lsp ready');

        // A server that stays up is no longer part of the previous crash streak.
        stableTimer = setTimeout(() => {
            restartAttempt = 0;
            stableTimer = null;
        }, 30000);
    }

    function scheduleRestart() {
        if (disposed || restartTimer !== null) {
            return;
        }
        const delay = Math.min(250 * Math.pow(2, restartAttempt), 10000);
        restartAttempt += 1;
        output.appendLine(`restarting siko lsp in ${delay}ms (attempt ${restartAttempt})`);
        restartTimer = setTimeout(() => {
            restartTimer = null;
            startServer();
        }, delay);
    }

    function startServer() {
        if (disposed) {
            return;
        }

        healthy = false;
        output.appendLine(`starting: ${resolvedCommand.join(' ')} (cwd: ${root}, port: ${port})`);
        const child = cp.spawn(resolvedCommand[0], resolvedCommand.slice(1), { cwd: root, env });
        server = child;
        child.on('error', (err) => output.appendLine(`failed to start siko lsp: ${err.message}`));
        child.on('close', (code, signal) => {
            output.appendLine(`siko lsp exited (code: ${code}, signal: ${signal})`);
            if (server !== child) {
                return;
            }
            server = null;
            healthy = false;
            if (stableTimer !== null) {
                clearTimeout(stableTimer);
                stableTimer = null;
            }
            scheduleRestart();
        });
        child.stdout.on('data', (data) => output.append(data.toString()));
        child.stderr.on('data', (data) => output.append(data.toString()));
        output.appendLine('siko lsp process spawned; stdout and stderr are attached');

        initializeServer(child).catch((err) => {
            if (disposed || server !== child) {
                return;
            }
            output.appendLine(`siko lsp startup failed: ${err.message}`);
            if (child.exitCode === null && child.signalCode === null) {
                child.kill();
            }
        });
    }

    function send(message, options) {
        const diagnosticsRun = options && options.clearDiagnostics ? ++latestDiagnosticsRun : 0;
        return waitUntilReady().then(() => postJson(port, '/rpc', message))
            .then((messages) => {
                applyMessages(messages, diagnosticsRun);
            })
            .catch((err) => {
                if (!disposed) {
                    output.appendLine(`siko lsp request failed: ${err.message}`);
                }
            });
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

    startServer();

    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument((document) => {
        if (document.languageId === 'siko') {
            output.appendLine(`saved Siko document: ${document.uri.toString()}; requesting project check`);
            notify('textDocument/didSave', { textDocument: { uri: document.uri.toString() } }, { clearDiagnostics: true });
        }
    }));

    context.subscriptions.push({
        dispose() {
            output.appendLine('stopping siko lsp');
            disposed = true;
            healthy = false;
            settleReadyWaiters(new Error('siko lsp is shutting down'));
            if (restartTimer !== null) {
                clearTimeout(restartTimer);
                restartTimer = null;
            }
            if (stableTimer !== null) {
                clearTimeout(stableTimer);
                stableTimer = null;
            }
            const child = server;
            server = null;
            try {
                postJson(port, '/rpc', { method: 'exit' }).catch(() => {});
                if (child !== null) {
                    child.kill();
                }
            } catch (err) {
                // the server is already gone
            }
        },
    });
}

function deactivate() {}

module.exports = { activate, deactivate };
