//! WASM Runtime support.
//!
//! Генерация JavaScript glue code для WASM модулей.

/// Генератор JavaScript runtime.
pub struct RuntimeGenerator;

impl RuntimeGenerator {
    /// Генерирует JavaScript код для загрузки WASM модуля.
    pub fn generate_loader(wasm_filename: &str) -> String {
        format!(
            r#"// ASG WASM Runtime
// Auto-generated - do not edit

class ASGRuntime {{
    constructor() {{
        this.memory = null;
        this.instance = null;
        this.textDecoder = new TextDecoder('utf-8');
        this.textEncoder = new TextEncoder();
    }}

    async load(wasmPath) {{
        const importObject = {{
            env: {{
                print_int: (value) => {{
                    console.log(value);
                }},
                print_float: (value) => {{
                    console.log(value);
                }},
                print_string: (ptr) => {{
                    console.log(this.readString(ptr));
                }},
                // GC hooks
                gc_trigger: () => {{
                    // Called when GC needs to run
                    console.log('[GC] Collection triggered');
                }},
                // Debug
                debug_log: (value) => {{
                    console.log('[DEBUG]', value);
                }},
            }}
        }};

        const response = await fetch(wasmPath || '{wasm_filename}');
        const bytes = await response.arrayBuffer();
        const result = await WebAssembly.instantiate(bytes, importObject);

        this.instance = result.instance;
        this.memory = this.instance.exports.memory;

        return this;
    }}

    // Read a null-terminated string from memory
    readString(ptr) {{
        if (ptr === 0) return '';
        const memory = new Uint8Array(this.memory.buffer);
        let end = ptr;
        while (memory[end] !== 0) end++;
        return this.textDecoder.decode(memory.slice(ptr, end));
    }}

    // Write a string to memory, returns pointer
    writeString(str) {{
        const bytes = this.textEncoder.encode(str + '\0');
        const ptr = this.instance.exports.gc_alloc(bytes.length);
        const memory = new Uint8Array(this.memory.buffer);
        memory.set(bytes, ptr + 16); // Skip header
        return ptr;
    }}

    // Call the main function
    run() {{
        if (this.instance.exports.main) {{
            return this.instance.exports.main();
        }}
        throw new Error('No main function exported');
    }}

    // Get memory stats
    getMemoryStats() {{
        const view = new DataView(this.memory.buffer);
        return {{
            heapPtr: view.getUint32(0x404, true),
            allocatedBytes: view.getUint32(0x408, true),
            objectCount: view.getUint32(0x40C, true),
        }};
    }}

    // Force garbage collection
    gc() {{
        if (this.instance.exports.gc_collect) {{
            this.instance.exports.gc_collect();
        }}
    }}
}}

// Node.js support
if (typeof module !== 'undefined' && module.exports) {{
    module.exports = {{ ASGRuntime }};
}}

// Browser: auto-run if script tag has data-autorun
if (typeof document !== 'undefined') {{
    const script = document.currentScript;
    if (script && script.hasAttribute('data-autorun')) {{
        const runtime = new ASGRuntime();
        runtime.load().then(() => {{
            const result = runtime.run();
            console.log('Result:', result);
        }});
    }}
}}
"#,
            wasm_filename = wasm_filename
        )
    }

    /// Генерирует HTML страницу для запуска WASM.
    pub fn generate_html(wasm_filename: &str, title: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eee;
        }}
        h1 {{
            color: #6366f1;
        }}
        #output {{
            background: #16213e;
            padding: 20px;
            border-radius: 8px;
            font-family: 'Fira Code', 'Consolas', monospace;
            white-space: pre-wrap;
            min-height: 200px;
            border: 1px solid #0f3460;
        }}
        #stats {{
            margin-top: 20px;
            padding: 10px;
            background: #0f3460;
            border-radius: 4px;
            font-size: 14px;
        }}
        button {{
            background: #6366f1;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px;
        }}
        button:hover {{
            background: #4f46e5;
        }}
    </style>
</head>
<body>
    <h1>🧠 {title}</h1>

    <div>
        <button onclick="runProgram()">▶ Run</button>
        <button onclick="runGC()">🗑 Run GC</button>
        <button onclick="showStats()">📊 Memory Stats</button>
    </div>

    <h2>Output</h2>
    <div id="output"></div>

    <div id="stats"></div>

    <script>
        let runtime = null;
        const output = document.getElementById('output');
        const stats = document.getElementById('stats');

        // Capture console.log
        const originalLog = console.log;
        console.log = (...args) => {{
            originalLog.apply(console, args);
            output.textContent += args.join(' ') + '\\n';
        }};

        class ASGRuntime {{
            constructor() {{
                this.memory = null;
                this.instance = null;
                this.textDecoder = new TextDecoder('utf-8');
            }}

            async load(wasmPath) {{
                const importObject = {{
                    env: {{
                        print_int: (value) => console.log(value),
                        print_float: (value) => console.log(value),
                        print_string: (ptr) => console.log(this.readString(ptr)),
                        gc_trigger: () => console.log('[GC] Collection triggered'),
                        debug_log: (value) => console.log('[DEBUG]', value),
                    }}
                }};

                const response = await fetch(wasmPath || '{wasm_filename}');
                const bytes = await response.arrayBuffer();
                const result = await WebAssembly.instantiate(bytes, importObject);

                this.instance = result.instance;
                this.memory = this.instance.exports.memory;
                return this;
            }}

            readString(ptr) {{
                if (ptr === 0) return '';
                const memory = new Uint8Array(this.memory.buffer);
                let end = ptr;
                while (memory[end] !== 0) end++;
                return this.textDecoder.decode(memory.slice(ptr, end));
            }}

            run() {{
                if (this.instance.exports.main) {{
                    return this.instance.exports.main();
                }}
                throw new Error('No main function exported');
            }}

            getMemoryStats() {{
                const view = new DataView(this.memory.buffer);
                return {{
                    heapPtr: view.getUint32(0x404, true),
                    allocatedBytes: view.getUint32(0x408, true),
                    objectCount: view.getUint32(0x40C, true),
                }};
            }}

            gc() {{
                if (this.instance.exports.gc_collect) {{
                    this.instance.exports.gc_collect();
                }}
            }}
        }}

        async function init() {{
            try {{
                runtime = new ASGRuntime();
                await runtime.load();
                console.log('✓ WASM module loaded');
            }} catch (e) {{
                console.log('✗ Failed to load: ' + e.message);
            }}
        }}

        function runProgram() {{
            output.textContent = '';
            if (!runtime) {{
                console.log('Loading...');
                init().then(() => {{
                    const result = runtime.run();
                    console.log('Result: ' + result);
                }});
            }} else {{
                const result = runtime.run();
                console.log('Result: ' + result);
            }}
        }}

        function runGC() {{
            if (runtime) {{
                runtime.gc();
                console.log('GC completed');
                showStats();
            }}
        }}

        function showStats() {{
            if (runtime) {{
                const s = runtime.getMemoryStats();
                stats.innerHTML = `
                    <strong>Memory Stats:</strong><br>
                    Heap Pointer: 0x${{s.heapPtr.toString(16)}}<br>
                    Allocated: ${{s.allocatedBytes}} bytes<br>
                    Objects: ${{s.objectCount}}
                `;
            }}
        }}

        // Auto-init
        init();
    </script>
</body>
</html>
"#,
            title = title,
            wasm_filename = wasm_filename
        )
    }

    /// Генерирует Node.js скрипт для запуска WASM.
    pub fn generate_node_runner(wasm_filename: &str) -> String {
        format!(
            r#"#!/usr/bin/env node
// ASG WASM Runner for Node.js
// Auto-generated - do not edit

const fs = require('fs');
const path = require('path');

class ASGRuntime {{
    constructor() {{
        this.memory = null;
        this.instance = null;
    }}

    async load(wasmPath) {{
        const importObject = {{
            env: {{
                print_int: (value) => {{
                    console.log(value);
                }},
                print_float: (value) => {{
                    console.log(value);
                }},
                print_string: (ptr) => {{
                    console.log(this.readString(ptr));
                }},
                gc_trigger: () => {{
                    // GC triggered
                }},
                debug_log: (value) => {{
                    console.log('[DEBUG]', value);
                }},
            }}
        }};

        const wasmBuffer = fs.readFileSync(wasmPath || path.join(__dirname, '{wasm_filename}'));
        const result = await WebAssembly.instantiate(wasmBuffer, importObject);

        this.instance = result.instance;
        this.memory = this.instance.exports.memory;

        return this;
    }}

    readString(ptr) {{
        if (ptr === 0) return '';
        const memory = new Uint8Array(this.memory.buffer);
        let end = ptr;
        while (memory[end] !== 0) end++;
        const decoder = new TextDecoder('utf-8');
        return decoder.decode(memory.slice(ptr, end));
    }}

    run() {{
        if (this.instance.exports.main) {{
            return this.instance.exports.main();
        }}
        throw new Error('No main function exported');
    }}

    getMemoryStats() {{
        const view = new DataView(this.memory.buffer);
        return {{
            heapPtr: view.getUint32(0x404, true),
            allocatedBytes: view.getUint32(0x408, true),
            objectCount: view.getUint32(0x40C, true),
        }};
    }}
}}

async function main() {{
    const runtime = new ASGRuntime();
    const wasmPath = process.argv[2] || path.join(__dirname, '{wasm_filename}');

    try {{
        await runtime.load(wasmPath);
        const result = runtime.run();

        if (process.argv.includes('--stats')) {{
            const stats = runtime.getMemoryStats();
            console.log('\\nMemory Stats:');
            console.log('  Heap Pointer:', '0x' + stats.heapPtr.toString(16));
            console.log('  Allocated:', stats.allocatedBytes, 'bytes');
            console.log('  Objects:', stats.objectCount);
        }}

        process.exit(typeof result === 'number' ? (result > 255 ? 0 : result) : 0);
    }} catch (e) {{
        console.error('Error:', e.message);
        process.exit(1);
    }}
}}

main();
"#,
            wasm_filename = wasm_filename
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_loader() {
        let js = RuntimeGenerator::generate_loader("test.wasm");
        assert!(js.contains("ASGRuntime"));
        assert!(js.contains("test.wasm"));
    }

    #[test]
    fn test_generate_html() {
        let html = RuntimeGenerator::generate_html("demo.wasm", "Demo");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("demo.wasm"));
        assert!(html.contains("Demo"));
    }

    #[test]
    fn test_generate_node_runner() {
        let node = RuntimeGenerator::generate_node_runner("app.wasm");
        assert!(node.contains("#!/usr/bin/env node"));
        assert!(node.contains("app.wasm"));
    }
}
