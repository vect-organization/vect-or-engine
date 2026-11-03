/**
 * vect-or-engine MCP Server
 * Wraps the NAPI bindings as a local JSON-RPC HTTP server.
 *
 * Start:  node mcp/server.mjs
 * Port:   127.0.0.1:4000 (default, override with PORT env)
 *
 * Methods:
 *   ping                          → { status, engine }
 *   load  { path }                → { loaded, count }
 *   build                         → { indexed }
 *   search { vector[], top_k }    → { results[], mode }
 *   validate { text }             → { is_valid, markers[] }
 *   kb_info                       → { count, dim, hnsw_ready }
 *
 * All requests: POST /mcp  body: { id, method, params }
 * All responses: { jsonrpc:"2.0", id, result } | { jsonrpc:"2.0", id, error }
 */

import http from 'node:http';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const _require = createRequire(import.meta.url);

// Load NAPI engine binary
const engineCandidates = [
  path.join(__dirname, '..', 'vect-or-engine-v0.3.0.node'),
  path.join(__dirname, '..', 'vect-or-engine-napi.win32-x64-msvc.node'),
  path.join(__dirname, '..', 'index.js'),
];

let engine = null;
for (const c of engineCandidates) {
  try {
    engine = _require(c);
    if (engine) { console.log(`[MCP] Engine loaded: ${c}`); break; }
  } catch { /* try next */ }
}

if (!engine) {
  console.error('[MCP] FATAL: Could not load vect-or-engine NAPI binary.');
  process.exit(1);
}

// ─── JSON-RPC errors ──────────────────────────────────────────────────────────

// Errors caused by the request; their message is safe to return to the client
class RpcError extends Error {
  constructor(rpcCode, message) {
    super(message);
    this.rpcCode = rpcCode; // not `code`: engine errors use code = 'InvalidArg' (a string)
  }
}

// Engine errors marked InvalidArg carry messages written for the caller
// (invalid path, vector dimension mismatch); everything else stays internal
function toRpcError(err) {
  if (err instanceof RpcError) return { code: err.rpcCode, message: err.message };
  if (err?.code === 'InvalidArg') return { code: -32602, message: String(err.message) };
  console.error('[MCP] Request handling error:', err?.stack ?? err);
  return { code: -32603, message: 'Internal error' };
}

function sendRpc(res, payload) {
  res.writeHead(200); // JSON-RPC always 200
  res.end(JSON.stringify(payload));
}

// ─── JSON-RPC dispatcher ──────────────────────────────────────────────────────

async function dispatch(method, params) {
  switch (method) {

    case 'ping':
      return { status: 'ok', engine: 'vect-or-engine-mcp' };

    case 'load': {
      const kbPath = params?.path;
      if (!kbPath) throw new RpcError(-32602, 'Missing param: path');
      const fn = engine.loadKnowledgeBase ?? engine.load_knowledge_base;
      if (!fn) throw new Error('Engine method loadKnowledgeBase not found');
      const count = await fn.call(engine, kbPath);
      return { loaded: true, count };
    }

    case 'build': {
      const fn = engine.buildIndex ?? engine.build_index;
      if (!fn) throw new Error('Engine method buildIndex not found');
      const count = await fn.call(engine);
      return { indexed: count };
    }

    case 'search': {
      if (!params?.vector || !Array.isArray(params.vector)) {
        throw new RpcError(-32602, 'Missing or invalid param: vector (must be number[])');
      }
      const topK = typeof params.top_k === 'number' ? params.top_k : 5;
      const floatVec = new Float32Array(params.vector);
      const fn = engine.search;
      if (!fn) throw new Error('Engine method search not found');
      const raw = await fn.call(engine, floatVec, topK);
      return { results: raw, mode: 'hnsw' };
    }

    case 'validate': {
      const text = params?.text ?? '';
      const fn = engine.validate ?? engine.validateSync;
      if (!fn) throw new Error('Engine method validate not found');
      const result = await fn.call(engine, text);
      return result;
    }

    case 'kb_info': {
      const fn = engine.kbInfo ?? engine.kb_info;
      if (!fn) throw new Error('Engine method kbInfo not found');
      return await fn.call(engine);
    }

    default:
      throw new RpcError(-32601, `Unknown method: ${method}`);
  }
}

// ─── HTTP Server ──────────────────────────────────────────────────────────────

const PORT = Number(process.env['PORT'] ?? 4000);

const server = http.createServer(async (req, res) => {
  // CORS for local dev
  res.setHeader('Access-Control-Allow-Origin', '127.0.0.1');
  res.setHeader('Content-Type', 'application/json');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  if (req.method !== 'POST' || req.url !== '/mcp') {
    res.writeHead(404);
    res.end(JSON.stringify({ error: 'Not found. POST /mcp' }));
    return;
  }

  let body = '';
  for await (const chunk of req) body += chunk;

  let request;
  try {
    request = JSON.parse(body);
  } catch {
    sendRpc(res, { jsonrpc: '2.0', id: null, error: { code: -32700, message: 'Parse error' } });
    return;
  }
  if (!request || typeof request !== 'object' || Array.isArray(request)) {
    sendRpc(res, { jsonrpc: '2.0', id: null, error: { code: -32600, message: 'Invalid Request' } });
    return;
  }

  const id = request.id ?? null;
  try {
    const result = await dispatch(request.method, request.params ?? {});
    sendRpc(res, { jsonrpc: '2.0', id, result });
  } catch (err) {
    sendRpc(res, { jsonrpc: '2.0', id, error: toRpcError(err) });
  }
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`[MCP] vect-or-engine MCP server listening on http://127.0.0.1:${PORT}/mcp`);
  console.log('[MCP] Methods: ping | load | build | search | validate | kb_info');
});
