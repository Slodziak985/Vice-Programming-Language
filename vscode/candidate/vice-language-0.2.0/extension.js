const childProcess = require("child_process");
const path = require("path");
const vscode = require("vscode");

const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
const COMMAND_TIMEOUT_MS = 20_000;

let diagnosticCollection;
const checkGeneration = new Map();
const KEYWORD_COMPLETIONS = [
  "use", "@entry", "@test", "@read", "fn", "let", "mut", "return", "if", "else",
  "while", "for", "in", "match", "break", "continue", "emit", "fail",
];
const NATIVE_COMPLETIONS = [
  "array.len", "array.join", "array.first", "array.last",
  "cli.flag", "cli.option",
  "env.get", "env.has",
  "fs.read_text", "fs.write_text", "fs.try_read_text", "fs.try_write_text",
  "json.parse", "json.try_parse", "json.stringify",
  "option.some", "option.none",
  "path.join", "path.dirname", "path.basename", "path.extension", "path.exists", "path.is_file", "path.is_dir",
  "process.args", "process.cwd",
  "result.ok", "result.err",
  "string.len", "string.trim", "string.lower", "string.upper",
];

function executablePath() {
  return vscode.workspace
    .getConfiguration("vice")
    .get("executablePath", "vice");
}

function workingDirectory(document) {
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  return folder ? folder.uri.fsPath : path.dirname(document.uri.fsPath);
}

function runVice(args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = childProcess.spawn(executablePath(), args, {
      cwd: options.cwd,
      shell: false,
      windowsHide: true,
      stdio: "pipe",
    });
    let stdout = "";
    let stderr = "";
    let settled = false;

    const finish = (callback) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      callback();
    };
    const append = (current, chunk) => {
      const next = current + chunk.toString("utf8");
      if (Buffer.byteLength(next, "utf8") > MAX_OUTPUT_BYTES) {
        child.kill();
        finish(() => reject(new Error("Vice command produced too much output")));
      }
      return next;
    };

    child.stdout.on("data", (chunk) => {
      stdout = append(stdout, chunk);
    });
    child.stderr.on("data", (chunk) => {
      stderr = append(stderr, chunk);
    });
    child.on("error", (error) => finish(() => reject(error)));
    child.on("close", (code) =>
      finish(() => resolve({ code: code ?? 1, stdout, stderr })),
    );

    const timer = setTimeout(() => {
      child.kill();
      finish(() => reject(new Error("Vice command timed out")));
    }, COMMAND_TIMEOUT_MS);

    if (options.input !== undefined) {
      child.stdin.end(options.input, "utf8");
    } else {
      child.stdin.end();
    }
  });
}

function diagnosticFromReport(report) {
  const source = report && report.diagnostic;
  if (!source) {
    return undefined;
  }
  const line = Math.max(0, Number(source.line || 1) - 1);
  const column = Math.max(0, Number(source.column || 1) - 1);
  const length = Math.max(1, Number(source.length || 1));
  const diagnostic = new vscode.Diagnostic(
    new vscode.Range(line, column, line, column + length),
    String(source.message || "Vice check failed"),
    vscode.DiagnosticSeverity.Error,
  );
  diagnostic.source = "Vice";
  diagnostic.code = source.type || "error";
  return diagnostic;
}

function localSymbolCompletions(document) {
  const text = document.getText();
  const symbols = new Set();
  const patterns = [
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)/g,
    /\b(?:let|mut)\s+([A-Za-z_][A-Za-z0-9_]*)/g,
    /\b(?:data|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)/g,
  ];
  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(text)) !== null) {
      symbols.add(match[1]);
    }
  }
  return [...symbols].sort().map((symbol) => {
    const item = new vscode.CompletionItem(symbol, vscode.CompletionItemKind.Variable);
    item.detail = "Local Vice symbol";
    return item;
  });
}

function completionItems(document) {
  const keywords = KEYWORD_COMPLETIONS.map((word) => {
    const item = new vscode.CompletionItem(word, vscode.CompletionItemKind.Keyword);
    item.detail = "Vice v0.2 keyword";
    return item;
  });
  const natives = NATIVE_COMPLETIONS.map((name) => {
    const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
    item.detail = "Vice native API";
    return item;
  });
  return [...keywords, ...natives, ...localSymbolCompletions(document)];
}

async function checkDocument(document, announceSuccess = false) {
  if (!document || document.languageId !== "vice" || document.uri.scheme !== "file") {
    return;
  }
  if (document.isDirty) {
    return;
  }
  const key = document.uri.toString();
  const generation = (checkGeneration.get(key) || 0) + 1;
  checkGeneration.set(key, generation);

  try {
    const result = await runVice(["check", "--json", document.uri.fsPath], {
      cwd: workingDirectory(document),
    });
    let report;
    try {
      report = JSON.parse(result.stdout);
    } catch (_error) {
      throw new Error(result.stderr.trim() || "Vice returned invalid diagnostic JSON");
    }

    if (checkGeneration.get(key) !== generation) {
      return;
    }
    const diagnostic = diagnosticFromReport(report);
    diagnosticCollection.set(document.uri, diagnostic ? [diagnostic] : []);
    if (announceSuccess && !diagnostic && result.code === 0) {
      vscode.window.setStatusBarMessage("Vice: check passed", 2500);
    }
  } catch (error) {
    if (checkGeneration.get(key) !== generation) {
      return;
    }
    diagnosticCollection.delete(document.uri);
    vscode.window.showErrorMessage(`Vice check failed: ${error.message}`);
  }
}

async function runCurrentFile() {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "vice") {
    vscode.window.showInformationMessage("Open a Vice file first.");
    return;
  }
  if (!(await editor.document.save())) {
    return;
  }
  const execution = new vscode.ProcessExecution(executablePath(), [
    "run",
    editor.document.uri.fsPath,
  ], { cwd: workingDirectory(editor.document) });
  const task = new vscode.Task(
    { type: "vice", task: "run" },
    vscode.TaskScope.Workspace,
    `Run ${path.basename(editor.document.uri.fsPath)}`,
    "Vice",
    execution,
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    clear: true,
  };
  await vscode.tasks.executeTask(task);
}

function activate(context) {
  diagnosticCollection = vscode.languages.createDiagnosticCollection("vice");
  context.subscriptions.push(diagnosticCollection);

  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (vscode.workspace.getConfiguration("vice").get("checkOnSave", true)) {
        void checkDocument(document);
      }
    }),
    vscode.workspace.onDidChangeTextDocument(({ document }) => {
      if (document.languageId === "vice") {
        const key = document.uri.toString();
        checkGeneration.set(key, (checkGeneration.get(key) || 0) + 1);
        diagnosticCollection.delete(document.uri);
      }
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      checkGeneration.delete(document.uri.toString());
      diagnosticCollection.delete(document.uri);
    }),
    vscode.commands.registerCommand("vice.checkFile", () =>
      checkDocument(vscode.window.activeTextEditor?.document, true),
    ),
    vscode.commands.registerCommand("vice.runFile", runCurrentFile),
    vscode.languages.registerDocumentFormattingEditProvider("vice", {
      async provideDocumentFormattingEdits(document) {
        try {
          const result = await runVice(["fmt", "--stdin"], {
            cwd: workingDirectory(document),
            input: document.getText(),
          });
          if (result.code !== 0) {
            throw new Error(result.stderr.trim() || "Vice formatter failed");
          }
          const end = document.lineAt(document.lineCount - 1).rangeIncludingLineBreak.end;
          return [vscode.TextEdit.replace(new vscode.Range(0, 0, end.line, end.character), result.stdout)];
        } catch (error) {
          vscode.window.showErrorMessage(`Vice formatting failed: ${error.message}`);
          return [];
        }
      },
    }),
    vscode.languages.registerCompletionItemProvider("vice", {
      provideCompletionItems(document) {
        return completionItems(document);
      },
    }, ".", "@"),
  );

  for (const document of vscode.workspace.textDocuments) {
    if (!document.isDirty) {
      void checkDocument(document);
    }
  }
}

function deactivate() {
  diagnosticCollection?.dispose();
}

module.exports = { activate, deactivate };
