#!/usr/bin/env python3
"""MCP stdio 客户端 — 司衡治理运行时的无头客户端。

MCP 协议流程：
  1. initialize 握手 → 获取 server capabilities
  2. notifications/initialized → 通知就绪
  3. tools/list → 获取工具列表（可选验证）
  4. tools/call → 调用具体工具

使用方式：
  python mcp_client.py --help
  python mcp_client.py index-rebuild --docs-dir /path/to/docs
  python mcp_client.py project-status --docs-dir /path/to/docs
  python mcp_client.py record-trail --docs-dir /path --anchor-doc X --type discovery ...
  python mcp_client.py snapshot-diff --docs-dir /path
  python mcp_client.py full-cycle --docs-dir /path  # 完整 DSR 周期
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


class MCPClient:
    """MCP stdio 客户端，通过 subprocess 与 sihankor binary 通信。"""

    def __init__(self, binary_path: str | Path, docs_dir: str | Path, debug: bool = False):
        self.binary = Path(binary_path).resolve()
        self.docs_dir = str(Path(docs_dir).resolve())
        self.debug = debug
        self._proc: subprocess.Popen | None = None
        self._request_id = 0

    # ── 协议层 ──

    def _next_id(self) -> int:
        self._request_id += 1
        return self._request_id

    def _send(self, method: str, params: dict | None = None) -> dict:
        """发送 JSON-RPC 请求并读取响应。"""
        req = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
        }
        if params is not None:
            req["params"] = params

        payload = json.dumps(req)
        if self.debug:
            print(f"[→] {payload[:300]}", file=sys.stderr)

        assert self._proc is not None and self._proc.stdin is not None and self._proc.stdout is not None
        self._proc.stdin.write(payload + "\n")
        self._proc.stdin.flush()

        response_line = self._proc.stdout.readline()
        if not response_line:
            raise ConnectionError("MCP server closed connection (stdout closed)")

        resp = json.loads(response_line.strip())
        if self.debug:
            print(f"[←] {json.dumps(resp, ensure_ascii=False)[:300]}", file=sys.stderr)

        if "error" in resp and resp["error"] is not None:
            error_detail = resp["error"]
            raise RuntimeError(f"MCP error ({error_detail.get('code', '?')}): {error_detail.get('message', 'unknown')}")

        return resp

    def _read_notification(self, timeout: float = 1.0) -> dict | None:
        """尝试读取一个通知（非阻塞 + timeout）。用于处理 initialized 通知。"""
        import select
        assert self._proc is not None and self._proc.stdout is not None
        if select.select([self._proc.stdout], [], [], timeout)[0]:
            line = self._proc.stdout.readline()
            if line:
                return json.loads(line.strip())
        return None

    # ── 生命周期 ──

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, *args):
        self.stop()

    def start(self):
        """启动 sihankor binary 进程，执行 MCP initialize 握手。"""
        self._proc = subprocess.Popen(
            [str(self.binary)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE if not self.debug else None,
            env={**os.environ, "SIHANKOR_DOCS_DIR": self.docs_dir},
            text=True,
        )

        # 1. initialize 请求
        init_resp = self._send("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "dsr2-mcp-client", "version": "0.1.0"},
        })
        server_caps = init_resp.get("capabilities", {})
        if self.debug:
            print(f"[i] Server capabilities: {json.dumps(server_caps, ensure_ascii=False)[:200]}", file=sys.stderr)

        # 2. 发送 initialized 通知（无 id，不需要响应）
        assert self._proc.stdin is not None
        initialized = json.dumps({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        })
        self._proc.stdin.write(initialized + "\n")
        self._proc.stdin.flush()

        # 3. 消费可能的通知响应（超时 500ms，不阻塞）
        self._read_notification(timeout=0.5)

    def stop(self):
        """终止 MCP server 进程。"""
        if self._proc:
            # 发送 shutdown
            try:
                self._send("shutdown")
            except Exception:
                pass
            self._proc.terminate()
            self._proc.wait(timeout=5)
            self._proc = None

    # ── 工具调用 ──

    def call_tool(self, name: str, arguments: dict | None = None) -> Any:
        """调用 MCP 工具并返回 parsed content。"""
        params = {"name": name}
        if arguments:
            params["arguments"] = arguments
        resp = self._send("tools/call", params)
        result = resp.get("result", {})
        # MCP 工具结果通常在 content[0].text 中
        content = result.get("content", [])
        if content and isinstance(content, list) and len(content) > 0:
            # 合并多个 text content item（MCP 工具可能分段返回）
            texts = []
            for item in content:
                if isinstance(item, dict) and item.get("type") == "text":
                    item_text = item.get("text", "")
                    # 跳过纯标记内容（如 "[SiHankor]"）
                    if item_text and item_text not in ("[SiHankor]",):
                        texts.append(item_text)
            combined = "\n".join(texts).strip()
            if combined:
                # 尝试解析为 JSON
                try:
                    return json.loads(combined)
                except (json.JSONDecodeError, TypeError):
                    pass
            return combined if combined else str(content)
        return result

    # ── 高层便利方法 ──

    def index_rebuild(self) -> str:
        """全量索引重建。"""
        result = self.call_tool("index_rebuild")
        return str(result)

    def project_status(self) -> str:
        """项目治理概览。"""
        result = self.call_tool("project_status")
        return str(result)

    def snapshot_diff(self) -> str:
        """最近两次快照差异。"""
        result = self.call_tool("snapshot_diff")
        return str(result)

    def variance_metric(self) -> str:
        """产出方差度量。"""
        result = self.call_tool("variance_metric")
        return str(result)

    def rule_audit(self) -> str:
        """规则审计。"""
        result = self.call_tool("rule_audit")
        return str(result)

    def rule_density(self) -> str:
        """规则密度。"""
        result = self.call_tool("rule_density")
        return str(result)

    def tradeoff_coverage(self) -> str:
        """ADR 三段式覆盖率。"""
        result = self.call_tool("tradeoff_coverage")
        return str(result)

    def trend_alignment(self) -> str:
        """趋势对齐。"""
        result = self.call_tool("trend_alignment")
        return str(result)

    def get_document(self, doc_id: str) -> str:
        """按 ID 获取文档。"""
        result = self.call_tool("get_document", {"id": doc_id})
        return str(result)

    def search_docs(self, query: str) -> str:
        """搜索文档。"""
        result = self.call_tool("search_docs", {"query": query})
        return str(result)

    def record_trail(
        self,
        anchor_doc: str,
        trail_type: str,
        turning_point: str,
        rationale: str,
        consequences: str,
        agents_involved: str | None = None,
    ) -> str:
        """记录行迹。"""
        args = {
            "anchor_doc": anchor_doc,
            "type": trail_type,
            "turning_point": turning_point,
            "rationale": rationale,
            "consequences": consequences,
        }
        if agents_involved:
            args["agents_involved"] = agents_involved
        result = self.call_tool("record_trail", args)
        return str(result)

    def analyze_document(self, target: str) -> str:
        """文档认知分析（iCL 仅 cognition）。"""
        result = self.call_tool("analyze_document", {"target": target})
        return str(result)

    def propose_decision(self, target: str) -> str:
        """生成决策建议（iCL + iWW）。"""
        result = self.call_tool("propose_decision", {"target": target})
        return str(result)

    def full_analysis(self, target: str) -> str:
        """全链分析（iCL + iWW + iCT）。"""
        result = self.call_tool("full_analysis", {"target": target})
        return str(result)

    def suggest_next_action(self) -> str:
        """流程推进建议。"""
        result = self.call_tool("suggest_next_action")
        return str(result)


# ── CLI ──

def cmd_index_rebuild(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.index_rebuild())


def cmd_project_status(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.project_status())


def cmd_snapshot_diff(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.snapshot_diff())


def cmd_metrics(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        if args.all or args.variance:
            print("=== Variance Metric ===\n" + client.variance_metric() + "\n")
        if args.all or args.snapshot:
            print("=== Snapshot Diff ===\n" + client.snapshot_diff() + "\n")
        if args.all or args.audit:
            print("=== Rule Audit ===\n" + client.rule_audit() + "\n")
        if args.all or args.density:
            print("=== Rule Density ===\n" + client.rule_density() + "\n")
        if args.all or args.tradeoff:
            print("=== Tradeoff Coverage ===\n" + client.tradeoff_coverage() + "\n")
        if args.all or args.trend:
            print("=== Trend Alignment ===\n" + client.trend_alignment() + "\n")


def cmd_record_trail(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.record_trail(args.anchor_doc, args.type, args.turning_point, args.rationale, args.consequences, args.agents))


def cmd_analyze(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.analyze_document(args.target))


def cmd_search(args):
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        print(client.search_docs(args.query))


def cmd_dsr2(args):
    """完整 DSR-2 治理迭代。"""
    with MCPClient(args.binary, args.docs_dir, debug=args.debug) as client:
        # 步骤 2: 索引 + 基线快照
        print("=" * 60)
        print("DSR-2: 步骤 2 — 索引重建")
        print("=" * 60)
        idx_result = client.index_rebuild()
        print(idx_result)
        print()

        print("=" * 60)
        print("DSR-2: 基线快照 (ProjectSnapshot #1)")
        print("=" * 60)
        baseline = client.project_status()
        print(baseline)
        print()

        # 步骤 4: 行迹记录（先记录 DSR-2 启动行迹）
        print("=" * 60)
        print("DSR-2: 步骤 4 — 行迹记录")
        print("=" * 60)
        trail1 = client.record_trail(
            anchor_doc="260628-1700-dsr-cycle",
            trail_type="method_selection",
            turning_point="DSR-2 改用 MCP 全工具集替代 DSR-1 的 rebuild_index 二进制",
            rationale="DSR-1 仅验证了索引管道；DSR-2 需验证治理运行时完整链路（MCP 工具 + 行迹 + 度量）",
            consequences="首次通过 MCP stdio 客户端驱动全工具集，MCP 工具的 IDE 外部调用能力得到验证",
        )
        print(trail1)
        print()

        # 度量检查
        print("=" * 60)
        print("DSR-2: 度量检查")
        print("=" * 60)
        print("--- Variance Metric ---")
        print(client.variance_metric())
        print()
        print("--- Rule Audit ---")
        print(client.rule_audit())
        print()
        print("--- Trend Alignment ---")
        print(client.trend_alignment())
        print()

        # 下一步建议
        print("=" * 60)
        print("DSR-2: 流程推进建议")
        print("=" * 60)
        print(client.suggest_next_action())

        print()
        print("=" * 60)
        print("DSR-2: MCP 全工具集验证完成。")
        print("snapshot_diff 需至少两次快照才能计算差异，")
        print("请在 governance 迭代完成后再次调用 dsr2 或 snapshot-diff。")
        print("=" * 60)


def build_parser():
    p = argparse.ArgumentParser(description="司衡 MCP 无头客户端")
    p.add_argument("--binary", default=None, help="sihankor binary 路径（默认自动查找）")
    p.add_argument("--docs-dir", default=None, help="docs/ 目录路径（默认 SIHANKOR_DOCS_DIR 环境变量）")
    p.add_argument("--debug", action="store_true", help="打印 MCP 协议调试信息")

    sub = p.add_subparsers(dest="command", required=True)

    p_index = sub.add_parser("index-rebuild", help="全量索引重建")
    p_index.set_defaults(func=cmd_index_rebuild)

    p_ps = sub.add_parser("project-status", help="项目治理概览")
    p_ps.set_defaults(func=cmd_project_status)

    p_diff = sub.add_parser("snapshot-diff", help="最近两次快照差异")
    p_diff.set_defaults(func=cmd_snapshot_diff)

    p_metrics = sub.add_parser("metrics", help="度量管道（支持 -all 或单项）")
    p_metrics.add_argument("--all", action="store_true")
    p_metrics.add_argument("--variance", action="store_true")
    p_metrics.add_argument("--snapshot", action="store_true")
    p_metrics.add_argument("--audit", action="store_true")
    p_metrics.add_argument("--density", action="store_true")
    p_metrics.add_argument("--tradeoff", action="store_true")
    p_metrics.add_argument("--trend", action="store_true")
    p_metrics.set_defaults(func=cmd_metrics)

    p_trail = sub.add_parser("record-trail", help="记录行迹")
    p_trail.add_argument("--anchor-doc", required=True)
    p_trail.add_argument("--type", required=True, choices=["direction_shift", "method_selection", "discovery"])
    p_trail.add_argument("--turning-point", required=True)
    p_trail.add_argument("--rationale", required=True)
    p_trail.add_argument("--consequences", required=True)
    p_trail.add_argument("--agents", default=None, help="参与 agent（可选）")
    p_trail.set_defaults(func=cmd_record_trail)

    p_analyze = sub.add_parser("analyze", help="文档认知分析（iCL）")
    p_analyze.add_argument("target", help="文档 ID 或路径")
    p_analyze.set_defaults(func=cmd_analyze)

    p_search = sub.add_parser("search", help="搜索已索引文档")
    p_search.add_argument("query", help="搜索关键词")
    p_search.set_defaults(func=cmd_search)

    p_dsr2 = sub.add_parser("dsr2", help="运行完整 DSR-2 治理迭代")
    p_dsr2.set_defaults(func=cmd_dsr2)

    return p


def find_binary() -> str:
    """自动查找 sihankor binary。"""
    candidates = [
        "target/release/sihankor",
        "target/debug/sihankor",
    ]
    # 从脚本路径或 CWD 查找
    for base in [Path.cwd(), Path(__file__).resolve().parent]:
        for rel in candidates:
            p = base / rel
            if p.exists():
                return str(p)
    # 从环境变量
    env_bin = os.environ.get("SIHANKOR_BINARY")
    if env_bin and Path(env_bin).exists():
        return env_bin
    return ""


def main():
    parser = build_parser()
    args = parser.parse_args()

    # 自动查找 binary
    if getattr(args, "binary", None) is None:
        found = find_binary()
        if found:
            args.binary = found
        else:
            parser.error("无法找到 sihankor binary。请通过 --binary 指定或设置 SIHANKOR_BINARY 环境变量。")

    bin_path = Path(args.binary)

    # 确定 docs_dir（优先级：CLI 参数 > 环境变量 > binary 项目推断）
    docs_dir = args.docs_dir or os.environ.get("SIHANKOR_DOCS_DIR") or ""
    if not docs_dir:
        for parent in [bin_path.parent, bin_path.parent.parent]:
            candidate = parent / "docs"
            if candidate.exists():
                docs_dir = str(candidate)
                break

    if not docs_dir:
        # 最后一次尝试：从 CWD 推断
        for parent in [Path.cwd(), Path.cwd().parent]:
            candidate = parent / "docs"
            if candidate.exists():
                docs_dir = str(candidate)
                break

    if not docs_dir:
        parser.error(
            "无法确定 docs_dir。请通过 --docs-dir 指定或设置 SIHANKOR_DOCS_DIR 环境变量。"
        )

    args.docs_dir = docs_dir
    args.func(args)


if __name__ == "__main__":
    main()
