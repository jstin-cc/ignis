#!/usr/bin/env node
// Copies the ignis-api binary into src-tauri/binaries/ with the Tauri-expected
// target-triple suffix. Works on Windows, Linux, and macOS.
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const rustcOutput = execSync("rustc -Vv").toString();
const triple = rustcOutput.match(/^host: (.+)$/m)?.[1]?.trim();
if (!triple) throw new Error("Could not determine rustc host triple");

const isWindows = process.platform === "win32";
const srcName = isWindows ? "ignis-api.exe" : "ignis-api";
const dstName = isWindows ? `ignis-api-${triple}.exe` : `ignis-api-${triple}`;

const src = path.join(__dirname, "..", "..", "target", "release", srcName);
const dst = path.join(__dirname, "..", "src-tauri", "binaries", dstName);

fs.mkdirSync(path.dirname(dst), { recursive: true });
fs.copyFileSync(src, dst);
console.log(`sidecar: ${src} → ${dst}`);
