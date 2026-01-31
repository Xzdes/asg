import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('ASG');
    outputChannel.appendLine('ASG extension activated');

    // Start LSP if enabled
    const config = vscode.workspace.getConfiguration('asg');
    if (config.get<boolean>('lsp.enable', true)) {
        startLspClient(context);
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('asg.runFile', runFile),
        vscode.commands.registerCommand('asg.runSelection', runSelection),
        vscode.commands.registerCommand('asg.startRepl', startRepl),
        vscode.commands.registerCommand('asg.restartLsp', () => restartLspClient(context))
    );

    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('asg.lsp')) {
                restartLspClient(context);
            }
        })
    );
}

function startLspClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('asg');
    const lspPath = config.get<string>('lsp.path', 'asg-lsp');
    const trace = config.get<string>('lsp.trace', 'off');

    const serverOptions: ServerOptions = {
        run: {
            command: lspPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: lspPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'asg' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.asg')
        },
        outputChannel: outputChannel,
        traceOutputChannel: outputChannel
    };

    client = new LanguageClient(
        'asg-lsp',
        'ASG Language Server',
        serverOptions,
        clientOptions
    );

    client.start().then(() => {
        outputChannel.appendLine('ASG LSP client started');
    }).catch(err => {
        outputChannel.appendLine(`Failed to start LSP: ${err}`);
        vscode.window.showWarningMessage(
            'ASG LSP not found. Install asg-lsp for full IDE support.'
        );
    });

    context.subscriptions.push(client);
}

async function restartLspClient(context: vscode.ExtensionContext) {
    if (client) {
        await client.stop();
        client = undefined;
    }

    const config = vscode.workspace.getConfiguration('asg');
    if (config.get<boolean>('lsp.enable', true)) {
        startLspClient(context);
    }
}

async function runFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active file');
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'asg') {
        vscode.window.showErrorMessage('Not an ASG file');
        return;
    }

    // Save the file first
    await document.save();

    // Run in terminal
    const terminal = getOrCreateTerminal();
    terminal.show();
    terminal.sendText(`asg "${document.fileName}"`);
}

async function runSelection() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const selection = editor.selection;
    const text = editor.document.getText(selection);

    if (!text.trim()) {
        vscode.window.showErrorMessage('No text selected');
        return;
    }

    // Run in REPL
    const terminal = getOrCreateTerminal();
    terminal.show();

    // Send each line to the REPL
    const lines = text.split('\n').filter(l => l.trim());
    for (const line of lines) {
        terminal.sendText(line);
    }
}

function startRepl() {
    const terminal = vscode.window.createTerminal({
        name: 'ASG REPL',
        shellPath: 'asg'
    });
    terminal.show();
}

let asgTerminal: vscode.Terminal | undefined;

function getOrCreateTerminal(): vscode.Terminal {
    if (asgTerminal && !asgTerminal.exitStatus) {
        return asgTerminal;
    }

    asgTerminal = vscode.window.createTerminal({
        name: 'ASG',
        shellPath: 'asg'
    });

    // Clean up when terminal is closed
    vscode.window.onDidCloseTerminal(t => {
        if (t === asgTerminal) {
            asgTerminal = undefined;
        }
    });

    return asgTerminal;
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
