"use client";

import { useState } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8081";

type Tab = "set" | "get" | "prefetch" | "stats";

export default function ConsolePage() {
  const [tab, setTab] = useState<Tab>("set");
  const [result, setResult] = useState<string>("");
  const [loading, setLoading] = useState(false);

  // set
  const [setNamespace, setSetNamespace] = useState("default");
  const [setKey, setSetKey] = useState("user:1");
  const [setValue, setSetValue] = useState('{"name":"Alice"}');
  const [setTtl, setSetTtl] = useState("3600");

  // get
  const [getNamespace, setGetNamespace] = useState("default");
  const [getKey, setGetKey] = useState("user:1");

  // prefetch
  const [prefetchNamespace, setPrefetchNamespace] = useState("default");
  const [prefetchKeys, setPrefetchKeys] = useState("user:1\nuser:2\nuser:3");

  const run = async () => {
    setLoading(true);
    setResult("");
    try {
      let res: Response;
      if (tab === "set") {
        res = await fetch(`${API_BASE}/api/v1/cache/set`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            namespace: setNamespace,
            key: setKey,
            value: setValue,
            ttl_secs: parseInt(setTtl, 10),
          }),
        });
      } else if (tab === "get") {
        res = await fetch(`${API_BASE}/api/v1/cache/get`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ namespace: getNamespace, key: getKey }),
        });
      } else if (tab === "prefetch") {
        res = await fetch(`${API_BASE}/api/v1/cache/prefetch`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            namespace: prefetchNamespace,
            keys: prefetchKeys.split("\n").map((k) => k.trim()).filter(Boolean),
          }),
        });
      } else {
        res = await fetch(`${API_BASE}/api/v1/cache/stats`);
      }
      const json = await res.json();
      setResult(JSON.stringify(json, null, 2));
    } catch (e) {
      setResult(`Error: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setLoading(false);
    }
  };

  const tabs: Tab[] = ["set", "get", "prefetch", "stats"];

  return (
    <div className="min-h-screen bg-gray-900 text-green-400 p-6 font-mono">
      <h1 className="text-2xl font-bold mb-6 text-green-300">
        ALICE-Cache-Cloud Console
      </h1>

      {/* Tab bar */}
      <div className="flex gap-2 mb-6">
        {tabs.map((t) => (
          <button
            key={t}
            onClick={() => { setTab(t); setResult(""); }}
            className={`px-4 py-2 rounded text-sm font-semibold transition-colors ${
              tab === t
                ? "bg-green-600 text-gray-900"
                : "bg-gray-800 text-green-400 hover:bg-gray-700"
            }`}
          >
            {t.toUpperCase()}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="bg-gray-800 rounded-lg p-6 mb-6 space-y-4">
        {tab === "set" && (
          <>
            <div>
              <label className="block text-xs text-green-500 mb-1">Namespace</label>
              <input
                value={setNamespace}
                onChange={(e) => setSetNamespace(e.target.value)}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs text-green-500 mb-1">Key</label>
              <input
                value={setKey}
                onChange={(e) => setSetKey(e.target.value)}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs text-green-500 mb-1">Value</label>
              <textarea
                value={setValue}
                onChange={(e) => setSetValue(e.target.value)}
                rows={3}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm resize-none"
              />
            </div>
            <div>
              <label className="block text-xs text-green-500 mb-1">TTL (seconds)</label>
              <input
                value={setTtl}
                onChange={(e) => setSetTtl(e.target.value)}
                type="number"
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
          </>
        )}

        {tab === "get" && (
          <>
            <div>
              <label className="block text-xs text-green-500 mb-1">Namespace</label>
              <input
                value={getNamespace}
                onChange={(e) => setGetNamespace(e.target.value)}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs text-green-500 mb-1">Key</label>
              <input
                value={getKey}
                onChange={(e) => setGetKey(e.target.value)}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
          </>
        )}

        {tab === "prefetch" && (
          <>
            <div>
              <label className="block text-xs text-green-500 mb-1">Namespace</label>
              <input
                value={prefetchNamespace}
                onChange={(e) => setPrefetchNamespace(e.target.value)}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs text-green-500 mb-1">
                Keys (one per line)
              </label>
              <textarea
                value={prefetchKeys}
                onChange={(e) => setPrefetchKeys(e.target.value)}
                rows={5}
                className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-green-400 text-sm resize-none"
              />
            </div>
          </>
        )}

        {tab === "stats" && (
          <p className="text-green-500 text-sm">
            Fetches GET /api/v1/cache/stats — click Run to retrieve hit/miss
            rates, memory usage, and namespace breakdown.
          </p>
        )}
      </div>

      <button
        onClick={run}
        disabled={loading}
        className="px-6 py-2 bg-green-600 hover:bg-green-500 disabled:bg-gray-700 text-gray-900 font-bold rounded transition-colors"
      >
        {loading ? "Running..." : "Run"}
      </button>

      {result && (
        <pre className="mt-6 bg-gray-800 rounded-lg p-4 text-green-300 text-sm overflow-x-auto whitespace-pre-wrap">
          {result}
        </pre>
      )}
    </div>
  );
}
