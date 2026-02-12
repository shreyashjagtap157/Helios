import * as vscode from 'vscode';
import * as path from 'path';
import { spawn } from 'child_process';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Omni');
    outputChannel.appendLine('Omni language extension activated');

    // Register formatter
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider('omni', {
            provideDocumentFormattingEdits: formatDocument
        })
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('omni.formatDocument', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'omni') {
                await vscode.commands.executeCommand('editor.action.formatDocument');
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omni.restartLsp', async () => {
            if (client) {
                await client.stop();
            }
            startLanguageServer(context);
        })
    );

    // Start language server
    startLanguageServer(context);

    // Format on save if configured
    context.subscriptions.push(
        vscode.workspace.onWillSaveTextDocument(async (event) => {
            const config = vscode.workspace.getConfiguration('omni');
            if (config.get('formatOnSave') && event.document.languageId === 'omni') {
                event.waitUntil(formatAndReturnEdits(event.document));
            }
        })
    );
}

function startLanguageServer(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('omni');
    let lspPath = config.get<string>('lsp.path');

    if (!lspPath) {
        // Try default locations
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (workspaceRoot) {
            const candidates = [
                path.join(workspaceRoot, 'tools', 'omni-lsp', 'target', 'release', 'omni-lsp'),
                path.join(workspaceRoot, 'tools', 'omni-lsp', 'target', 'debug', 'omni-lsp'),
                'omni-lsp' // Try PATH
            ];

            for (const candidate of candidates) {
                try {
                    const ext = process.platform === 'win32' ? '.exe' : '';
                    lspPath = candidate + ext;
                    break;
                } catch {
                    continue;
                }
            }
        }
    }

    if (!lspPath) {
        outputChannel.appendLine('Warning: omni-lsp not found. Language server features disabled.');
        return;
    }

    const serverOptions: ServerOptions = {
        run: { command: lspPath, transport: TransportKind.stdio },
        debug: { command: lspPath, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'omni' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.omni')
        },
        outputChannel
    };

    client = new LanguageClient(
        'omniLanguageServer',
        'Omni Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
    outputChannel.appendLine('Omni language server started');
}

async function formatDocument(document: vscode.TextDocument): Promise<vscode.TextEdit[]> {
    return formatAndReturnEdits(document);
}

async function formatAndReturnEdits(document: vscode.TextDocument): Promise<vscode.TextEdit[]> {
    const config = vscode.workspace.getConfiguration('omni.formatter');
    const indentSpaces = config.get<number>('indentSpaces', 4);
    const maxLineWidth = config.get<number>('maxLineWidth', 100);

    // Try to use omni-fmt
    const fmtPath = findOmniFmt();
    if (fmtPath) {
        try {
            const formatted = await runFormatter(fmtPath, document.getText(), indentSpaces, maxLineWidth);
            if (formatted !== document.getText()) {
                const fullRange = new vscode.Range(
                    document.positionAt(0),
                    document.positionAt(document.getText().length)
                );
                return [vscode.TextEdit.replace(fullRange, formatted)];
            }
        } catch (e) {
            outputChannel.appendLine(`Formatter error: ${e}`);
        }
    } else {
        // Fallback: basic formatting
        const formatted = basicFormat(document.getText(), indentSpaces);
        if (formatted !== document.getText()) {
            const fullRange = new vscode.Range(
                document.positionAt(0),
                document.positionAt(document.getText().length)
            );
            return [vscode.TextEdit.replace(fullRange, formatted)];
        }
    }

    return [];
}

function findOmniFmt(): string | undefined {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!workspaceRoot) return undefined;

    const ext = process.platform === 'win32' ? '.exe' : '';
    const candidates = [
        path.join(workspaceRoot, 'tools', 'omni-fmt', 'target', 'release', 'omni-fmt' + ext),
        path.join(workspaceRoot, 'tools', 'omni-fmt', 'target', 'debug', 'omni-fmt' + ext),
    ];

    for (const candidate of candidates) {
        try {
            require('fs').accessSync(candidate, require('fs').constants.X_OK);
            return candidate;
        } catch {
            continue;
        }
    }

    return undefined;
}

function runFormatter(fmtPath: string, content: string, indentSpaces: number, maxWidth: number): Promise<string> {
    return new Promise((resolve, reject) => {
        const proc = spawn(fmtPath, ['--stdout', '-s', indentSpaces.toString(), '-w', maxWidth.toString(), '-'], {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let stdout = '';
        let stderr = '';

        proc.stdout.on('data', (data) => {
            stdout += data.toString();
        });

        proc.stderr.on('data', (data) => {
            stderr += data.toString();
        });

        proc.on('close', (code) => {
            if (code === 0) {
                resolve(stdout);
            } else {
                reject(new Error(`omni-fmt exited with code ${code}: ${stderr}`));
            }
        });

        proc.on('error', reject);

        proc.stdin.write(content);
        proc.stdin.end();
    });
}

function basicFormat(content: string, indentSpaces: number): string {
    const lines = content.split('\n');
    const formatted: string[] = [];

    for (const line of lines) {
        const trimmed = line.trim();

        // Preserve indentation structure
        const leadingSpaces = line.length - line.trimStart().length;
        const indentLevel = Math.floor(leadingSpaces / indentSpaces);
        const indent = ' '.repeat(indentLevel * indentSpaces);

        // Basic cleanup
        let formattedLine = trimmed
            .replace(/\s+/g, ' ')  // Collapse multiple spaces
            .replace(/\s*,\s*/g, ', ')  // Normalize comma spacing
            .replace(/\s*:\s*/g, ': ')  // Normalize colon spacing (except ::)
            .replace(/: :/g, '::');  // Restore ::

        formatted.push(formattedLine ? indent + formattedLine : '');
    }

    // Remove duplicate blank lines
    const result: string[] = [];
    let prevBlank = false;
    for (const line of formatted) {
        const isBlank = line.trim() === '';
        if (isBlank && prevBlank) continue;
        result.push(line);
        prevBlank = isBlank;
    }

    // Ensure trailing newline
    let output = result.join('\n');
    if (!output.endsWith('\n')) {
        output += '\n';
    }

    return output;
}

export function deactivate(): Thenable<void> | undefined {
    if (client) {
        return client.stop();
    }
    return undefined;
}
