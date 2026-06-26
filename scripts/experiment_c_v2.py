#!/usr/bin/env python3
"""experiment C v2: use commit-density slices instead of calendar time
S0 (oldest) -> S6 (newest): 7 snapshots, ~17 commits each
"""
import subprocess, json, re, os
from collections import Counter

REPO = "/Users/moc/projects/SiHankor/sihankor"

def run(cmd):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True).stdout.strip()

def files_at(commit):
    out = run(["git", "ls-tree", "-r", "--name-only", commit])
    return [f for f in out.split("\n") if f and f.endswith(".sih.md")]

def content_at(commit, path):
    try:
        return run(["git", "show", f"{commit}:{path}"])
    except:
        return ""

def count_lines(commit):
    return sum(content_at(commit, f).count("\n") for f in files_at(commit))

def count_docs(commit):
    return len(files_at(commit))

TERMS = [
    "道", "元", "法", "术", "几", "约", "形迹", "鉴",
    "自-然", "必-为", "自晦",
    "顺因", "有度", "知止", "损补", "顺势",
    "iCL", "iWW", "iCT",
    "反推九段式", "可证伪条件", "诗与理",
    "F-Forbid", "G-Guideline", "J-Judgment",
    "propose", "resolve", "ratify", "Reopen", "Supersede",
]

def term_stats(commit):
    c = Counter()
    for f in files_at(commit):
        content = content_at(commit, f)
        for t in TERMS:
            c[t] += content.count(t)
    return dict(c)

def stage_stats(commit):
    stages = Counter()
    natures = Counter()
    for f in files_at(commit):
        content = content_at(commit, f)
        m = re.search(r'^stage:\s*(\S+)', content, re.MULTILINE)
        if m: stages[m.group(1)] += 1
        # nature from directory
        parts = f.split("/")
        if len(parts) >= 2 and parts[0] == "docs":
            natures[parts[1]] += 1
    return dict(stages), dict(natures)

def cross_ref_stats(commit):
    edges = 0
    for f in files_at(commit):
        content = content_at(commit, f)
        deps_match = re.search(r'##\s+DEPS\s*\n(.*?)(?=\n##\s|\Z)', content, re.DOTALL)
        if deps_match:
            edges += len([l for l in deps_match.group(1).strip().split("\n") if l.strip().startswith("- ")])
    return edges

def code_lines(commit):
    out = run(["git", "ls-tree", "-r", "--name-only", commit])
    code = [f for f in out.split("\n") if f and f.endswith(".rs")]
    total = 0
    for cf in code:
        total += content_at(commit, cf).count("\n")
    return total

# 7 snapshots from oldest to newest
snapshots = [
    ("S0-init",      "be604b52b2fd70a3c5dae851ca603fbce342926c", "2026-06-08"),
    ("S1-philosophy","6bc4ccc7a668634c0772fed634cc762757e2d63d", "2026-06-16"),
    ("S2-refactor",  "e9e93cbaa2a1c96434c0f7145b4d4f3361d3e214", "2026-06-16"),
    ("S3-engine",    "6ddc18a2fb0b48fbdff17ea0ac49dee7ca6ced71", "2026-06-16"),
    ("S4-dashboard", "ac4ee5ecac3948d234ff64450cb9b924ebe72fc0", "2026-06-25"),
    ("S5-fixes",     "39d1d61e55b7da59c008ca5dfe0b5c5ef9ec436b", "2026-06-25"),
    ("S6-closures",  "7d16129341e53707d4e790b47692a0971a964e3f", "2026-06-26"),
]

results = []
for name, h, date in snapshots:
    data = {
        "name": name, "date": date, "hash": h,
        "docs": count_docs(h),
        "lines": count_lines(h),
        "edges": cross_ref_stats(h),
        "code_lines": code_lines(h),
        "terms": term_stats(h),
    }
    data["stages"], data["natures"] = stage_stats(h)
    data["term_unique"] = len([k for k,v in data["terms"].items() if v > 0])
    data["term_total"] = sum(data["terms"].values())
    results.append(data)

print("| 指标 |", " | ".join(r["name"] for r in results), "| 趋势 |")
print("|------|", "|".join("---" for _ in results), "|------|")

# Doc count
vals = [r["docs"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 文档数 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Lines
vals = [r["lines"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 总行数 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Code lines
vals = [r["code_lines"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 代码行数 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Doc/code ratio
vals = [round(r["lines"]/max(r["code_lines"],1),2) for r in results]
trend = "↑" if vals[-1] > vals[0] else "↓"
print(f"| 文档/代码比 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Edges
vals = [r["edges"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 交叉引用边 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Edge/node ratio
vals = [round(r["edges"]/max(r["docs"],1),2) for r in results]
trend = "↑" if vals[-1] > vals[0] else "↓"
print(f"| 边/节点比 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Term unique
vals = [r["term_unique"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 独特术语数 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Term total
vals = [r["term_total"] for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| 术语总出现 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Term density
vals = [round(r["term_total"]/max(r["lines"],1)*1000,1) for r in results]
trend = "↑" if vals[-1] > vals[0] else "↓"
print(f"| 术语密度‰ | {' | '.join(str(v) for v in vals)} | {trend} |")

# Stage 3/3
vals = [r["stages"].get("3/3",0) for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| stage 3/3 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Stage 2/3
vals = [r["stages"].get("2/3",0) for r in results]
trend = "↑" if vals[-1] > vals[0] else "→"
print(f"| stage 2/3 | {' | '.join(str(v) for v in vals)} | {trend} |")

# Stage 1/3
vals = [r["stages"].get("1/3",0) for r in results]
print(f"| stage 1/3 | {' | '.join(str(v) for v in vals)} | - |")

print()
print("## 判定")

# Calculate growth rates between consecutive snapshots
doc_growths = [results[i+1]["docs"] - results[i]["docs"] for i in range(len(results)-1)]
term_growths = [results[i+1]["term_total"] - results[i]["term_total"] for i in range(len(results)-1)]
line_growths = [results[i+1]["lines"] - results[i]["lines"] for i in range(len(results)-1)]

print(f"文档增量: {doc_growths}")
print(f"术语增量: {term_growths}")
print(f"行数增量: {line_growths}")

# Check if growth rate accelerates or decelerates
if len(doc_growths) >= 3:
    early_doc = sum(doc_growths[:3])
    late_doc = sum(doc_growths[3:])
    early_term = sum(term_growths[:3])
    late_term = sum(term_growths[3:])
    early_line = sum(line_growths[:3])
    late_line = sum(line_growths[3:])
    
    print(f"\n前半段(S0-S3): 文档+{early_doc}  术语+{early_term}  行数+{early_line}")
    print(f"后半段(S3-S6): 文档+{late_doc}  术语+{late_term}  行数+{late_line}")
    
    if late_term > early_term:
        print("\n**术语增殖在加速** — 概念发散在增长（倒U上升段或顶点右移）")
    elif late_term < early_term:
        print("\n**术语增殖在减速** — 概念发散在放缓（可能已过倒U顶点）")
    else:
        print("\n**术语增殖速度恒定** — 单调增长")
    
    if late_doc < early_doc and late_term > early_term:
        print("**混合判定：文档增长在放缓但术语仍在加速增殖** — 概念层出现发散加速信号")
    elif late_doc > early_doc and late_term > early_term:
        print("**单调判定：文档和术语同步增长** — 方向正确但速度未放缓")
    elif late_doc < early_doc and late_term < early_term:
        print("**收敛判定：文档和术语增速都在放缓** — 体系正在收敛")
