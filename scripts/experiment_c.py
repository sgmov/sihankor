#!/usr/bin/env python3
"""closure 2 experiment C: inverted-U hypothesis test for SiHankor self-governance

Measures divergence and convergence indicators across three time phases (T0/T1/T2)
to determine whether governance intensity and document convergence form:
  - monotonic (more governance = more convergence) or
  - inverted-U (governance overshoot → divergence increases)

Output: experiment_c_report.md
"""
import subprocess, re, os, json, sys
from datetime import datetime
from collections import defaultdict, Counter

REPO = "/Users/moc/projects/SiHankor/sihankor"
SINCE_DATE = "2025-12-01"

def run(cmd):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True).stdout.strip()

def git_log():
    """Return list of (hash, date, subject) tuples"""
    out = run(["git", "log", "--all", f"--since={SINCE_DATE}", "--format=%H %ad %s", "--date=short"])
    commits = []
    for line in out.split("\n"):
        if not line.strip(): continue
        parts = line.split(" ", 2)
        if len(parts) >= 3:
            commits.append((parts[0], parts[1], parts[2]))
    return commits

def files_at_commit(commit_hash, pattern="docs/*.sih.md"):
    """Return list of files matching pattern at a given commit."""
    out = run(["git", "ls-tree", "-r", "--name-only", commit_hash])
    return [f for f in out.split("\n") if f and f.endswith(".sih.md")]

def file_content_at(commit_hash, filepath):
    """Get file content at a given commit, or empty string."""
    try:
        return run(["git", "show", f"{commit_hash}:{filepath}"])
    except:
        return ""

def count_lines_in_docs(commit_hash):
    """Count total lines in all .sih.md docs at a commit."""
    total = 0
    for f in files_at_commit(commit_hash):
        content = file_content_at(commit_hash, f)
        total += content.count("\n")
    return total

def count_docs(commit_hash):
    return len(files_at_commit(commit_hash))

def extract_terms_from_content(content):
    """Extract SiHankor-specific terms for concept proliferation measurement."""
    terms = [
        "元", "道", "法", "术", "几", "约", "形迹", "鉴",
        "自-然", "必-为", "自晦", 
        "顺因", "有度", "知止", "损补", "顺势",
        "iCL", "iWW", "iCT", "F/G/J", "F-Forbid", "G-Guideline", "J-Judgment",
        "反推九段式", "可证伪条件", "诗与理", "最优化论证",
        "三机", "符约", "文约",
        "propose", "resolve", "ratify", "Reopen", "Supersede",
    ]
    found = defaultdict(int)
    for term in terms:
        found[term] = content.count(term)
    return dict(found)

def extract_id_from_frontmatter(content):
    """Extract id from frontmatter."""
    m = re.search(r'^id:\s*(\S+)', content, re.MULTILINE)
    return m.group(1) if m else None

def extract_stage_from_frontmatter(content):
    """Extract stage from frontmatter."""
    m = re.search(r'^stage:\s*(\S+)', content, re.MULTILINE)
    return m.group(1) if m else None

def extract_upstream_from_frontmatter(content):
    """Extract upstream from frontmatter."""
    m = re.search(r'^upstream:\s*(\S+)', content, re.MULTILINE)
    return m.group(1) if m else None

def extract_defined_terms(content):
    """Count term definitions (bolded terms in definition statements)."""
    # Match **term** patterns that look like definitions
    return len(re.findall(r'\*\*([^*]+)\*\*', content))

def cross_ref_complexity(commit_hash):
    """Count edges/nodes in the document reference graph."""
    docs = {}
    for f in files_at_commit(commit_hash):
        content = file_content_at(commit_hash, f)
        doc_id = extract_id_from_frontmatter(content)
        upstream = extract_upstream_from_frontmatter(content)
        if doc_id:
            docs[doc_id] = {"upstream": upstream, "deps": []}
    
    # Count DEPS references
    for f in files_at_commit(commit_hash):
        content = file_content_at(commit_hash, f)
        doc_id = extract_id_from_frontmatter(content)
        if not doc_id:
            continue
        # Find DEPS section
        deps_match = re.search(r'##\s+DEPS\s*\n(.*?)(?=\n##\s|\Z)', content, re.DOTALL)
        if deps_match:
            dep_lines = deps_match.group(1).strip().split("\n")
            for line in dep_lines:
                dep_id_match = re.match(r'-\s*(\S+)', line.strip())
                if dep_id_match:
                    dep_id = dep_id_match.group(1)
                    if doc_id in docs:
                        docs[doc_id]["deps"].append(dep_id)
    
    nodes = len(docs)
    edges = sum(len(v["deps"]) for v in docs.values())
    return nodes, edges

def concept_stability(commits, target_terms=["道一", "道二", "道三", "道四", "顺因", "有度", "知止", "损补", "顺势"]):
    """Count how many times key terms are redefined across commits."""
    # Simplification: count total occurrences and check for variation in definition text
    total = defaultdict(int)
    for commit_hash, date, subject in commits:
        for f in files_at_commit(commit_hash):
            content = file_content_at(commit_hash, f)
            for term in target_terms:
                if term in content:
                    total[term] += 1
    return dict(total)

def main():
    commits = git_log()
    print(f"Total commits since {SINCE_DATE}: {len(commits)}")
    
    # Define time phases based on known SiHankor milestones
    # T0: Pre-four-dao (before 240602)  - "五维天道" phase
    # T1: Four-dao established (240602-2407) - core philosophy stable
    # T2: Current (2606) - full system with 元论, 论证集, L14-L19 etc.
    
    # Find commit hashes for each phase
    t0_commits = [c for c in commits if c[1] < "2024-06-02"]
    t1_commits = [c for c in commits if "2024-06-02" <= c[1] < "2024-08-01"]
    t2_commits = [c for c in commits if c[1] >= "2025-06-01"]
    
    phases = {
        "T0 (五维天道/v4.0.0初期)": t0_commits[-1][0] if t0_commits else "HEAD",
        "T1 (四道确立/道论+法论+鉴论)": t1_commits[-1][0] if t1_commits else "HEAD",
        "T2 (当前/元论+论证集+L14-L19)": "HEAD",
    }
    
    report_lines = [
        "# 闭合2 实验C：司衡倒U假设检验报告",
        "",
        f"生成时间: {datetime.now().isoformat()}",
        "",
        "## 1. 方法",
        "",
        "对司衡Git历史在三个时间相位（T0/T1/T2）的快照进行分析，测量发散指标和收敛指标。",
        "",
        "## 2. 时间相位",
        "",
    ]
    
    for phase_name, hash_val in phases.items():
        report_lines.append(f"- **{phase_name}** : commit `{hash_val[:8]}`")
    
    report_lines += [
        "",
        "## 3. 宏观指标",
        "",
        "| 指标 | T0 | T1 | T2 | 趋势 |",
        "|------|----|----|----|------|",
    ]
    
    phase_data = {}
    for phase_name, hash_val in phases.items():
        doc_count = count_docs(hash_val)
        line_count = count_lines_in_docs(hash_val)
        nodes, edges = cross_ref_complexity(hash_val)
        phase_data[phase_name] = {
            "doc_count": doc_count,
            "line_count": line_count,
            "nodes": nodes,
            "edges": edges,
        }
    
    # Table rows
    phases_ordered = list(phases.keys())
    
    # Doc count
    vals = [phase_data[p]["doc_count"] for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 文档数量 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Line count
    vals = [phase_data[p]["line_count"] for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 总行数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Cross-ref edges
    vals = [phase_data[p]["edges"] for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 交叉引用边数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Edge/node ratio
    vals = [round(phase_data[p]["edges"] / max(phase_data[p]["nodes"], 1), 2) for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 边/节点比 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    report_lines += [
        "",
        "## 4. 概念增殖指标（发散）",
        "",
        "| 指标 | T0 | T1 | T2 | 趋势 |",
        "|------|----|----|----|------|",
    ]
    
    # Term proliferation at each phase
    for phase_name, hash_val in phases.items():
        all_terms = Counter()
        for f in files_at_commit(hash_val):
            content = file_content_at(hash_val, f)
            term_counts = extract_terms_from_content(content)
            all_terms.update(term_counts)
        phase_data[phase_name]["term_counts"] = dict(all_terms)
    
    # Total unique terms
    vals = [len(phase_data[p]["term_counts"]) for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 特定义术语种类 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Total term occurrences
    vals = [sum(phase_data[p]["term_counts"].values()) for p in phases_ordered]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 术语总出现次数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Term density (occurrences per 1000 lines)
    vals = [
        round(sum(phase_data[p]["term_counts"].values()) / max(phase_data[p]["line_count"], 1) * 1000, 1)
        for p in phases_ordered
    ]
    trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
    report_lines.append(f"| 术语密度（/千行） | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # New terms added per phase
    t0_terms = set(phase_data[phases_ordered[0]]["term_counts"].keys()) if len(phases_ordered) > 0 else set()
    t1_terms = set(phase_data[phases_ordered[1]]["term_counts"].keys()) if len(phases_ordered) > 1 else set()
    t2_terms = set(phase_data[phases_ordered[2]]["term_counts"].keys()) if len(phases_ordered) > 2 else set()
    
    new_in_t1 = t1_terms - t0_terms
    new_in_t2 = t2_terms - t1_terms
    
    report_lines.append(f"| T0→T1 新增术语 | - | {len(new_in_t1)} | - | - |")
    report_lines.append(f"| T1→T2 新增术语 | - | - | {len(new_in_t2)} | {'↑' if len(new_in_t2) > len(new_in_t1) else '↓'} |")
    
    report_lines += [
        "",
        "## 5. 概念冗余指标（发散）",
        "",
        "| 指标 | T0 | T1 | T2 | 趋势 |",
        "|------|----|----|----|------|",
    ]
    
    # Check how many documents define the same core concepts
    core_concepts = ["道", "元", "法", "鉴", "顺因", "有度", "知止", "损补", "顺势"]
    for concept in core_concepts:
        doc_count_per_phase = []
        for phase_name, hash_val in phases.items():
            count = 0
            for f in files_at_commit(hash_val):
                content = file_content_at(hash_val, f)
                # Check if document contains a definition of this concept (bolded term)
                if f"**{concept}" in content or f"{concept}：" in content:
                    count += 1
            doc_count_per_phase.append(count)
        vals = doc_count_per_phase
        if any(v > 0 for v in vals):
            trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
            report_lines.append(f"| '{concept}' 定义文档数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    report_lines += [
        "",
        "## 6. 收敛指标",
        "",
        "| 指标 | T0 | T1 | T2 | 趋势 |",
        "|------|----|----|----|------|",
    ]
    
    # Stage distribution (how many docs are at each stage)
    for target_stage in ["1/3", "2/3", "3/3"]:
        counts = []
        for phase_name, hash_val in phases.items():
            count = 0
            for f in files_at_commit(hash_val):
                content = file_content_at(hash_val, f)
                stage = extract_stage_from_frontmatter(content)
                if stage == target_stage:
                    count += 1
            counts.append(count)
        vals = counts
        trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
        report_lines.append(f"| stage {target_stage} 文档数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    # Nature distribution
    natures = set()
    for phase_name, hash_val in phases.items():
        for f in files_at_commit(hash_val):
            parts = f.split("/")
            if len(parts) >= 2 and parts[0] == "docs":
                natures.add(parts[1])
    
    for nature in sorted(natures):
        counts = []
        for phase_name, hash_val in phases.items():
            count = sum(1 for f in files_at_commit(hash_val) if f.startswith(f"docs/{nature}/"))
            counts.append(count)
        vals = counts
        if any(v > 0 for v in vals):
            trend = "↑" if vals[-1] > vals[0] else "↓" if vals[-1] < vals[0] else "→"
            report_lines.append(f"| {nature}/ 文档数 | {vals[0]} | {vals[1]} | {vals[2]} | {trend} |")
    
    report_lines += [
        "",
        "## 7. 文档/代码比",
        "",
        "| 指标 | T0 | T1 | T2 | 趋势 |",
        "|------|----|----|----|------|",
    ]
    
    # Count code lines at each phase
    for phase_name, hash_val in phases.items():
        try:
            out = run(["git", "ls-tree", "-r", "--name-only", hash_val])
            code_files = [f for f in out.split("\n") if f and f.endswith(".rs")]
            code_lines = 0
            for cf in code_files:
                content = file_content_at(hash_val, cf)
                code_lines += content.count("\n")
            phase_data[phase_name]["code_lines"] = code_lines
        except:
            phase_data[phase_name]["code_lines"] = 0
    
    vals = [phase_data[p]["line_count"] for p in phases_ordered]
    code_vals = [phase_data[p]["code_lines"] for p in phases_ordered]
    ratios = [round(vals[i] / max(code_vals[i], 1), 2) for i in range(len(vals))]
    trend = "↑" if ratios[-1] > ratios[0] else "↓" if ratios[-1] < ratios[0] else "→"
    report_lines.append(f"| 文档/代码比 | {ratios[0]} | {ratios[1]} | {ratios[2]} | {trend} |")
    
    report_lines += [
        f"| 文档总行数 | {vals[0]} | {vals[1]} | {vals[2]} | - |",
        f"| 代码总行数 | {code_vals[0]} | {code_vals[1]} | {code_vals[2]} | - |",
        "",
        "## 8. 判定",
        "",
    ]
    
    # Determine direction
    # Key indicator: does concept proliferation accelerate or decelerate?
    # If T1→T2 shows acceleration relative to T0→T1, inverted-U may be in play
    
    new_t1_count = len(new_in_t1)
    new_t2_count = len(new_in_t2)
    doc_growth_rate = (phase_data[phases_ordered[2]]["doc_count"] - phase_data[phases_ordered[1]]["doc_count"]) / max(phase_data[phases_ordered[1]]["doc_count"] - phase_data[phases_ordered[0]]["doc_count"], 1) if phase_data[phases_ordered[1]]["doc_count"] > phase_data[phases_ordered[0]]["doc_count"] else 0
    term_growth_rate = new_t2_count / max(new_t1_count, 1) if new_t1_count > 0 else 0
    
    report_lines.append(f"**新术语增长率**: T0→T1: {new_t1_count}个，T1→T2: {new_t2_count}个 (增长率: {term_growth_rate:.1f}x)")
    report_lines.append(f"**文档增长率**: T0→T1: {phase_data[phases_ordered[1]]['doc_count'] - phase_data[phases_ordered[0]]['doc_count']}个，T1→T2: {phase_data[phases_ordered[2]]['doc_count'] - phase_data[phases_ordered[1]]['doc_count']}个")
    
    if term_growth_rate > 2.0 or doc_growth_rate > 2.0:
        report_lines.append("")
        report_lines.append("### 判定: 混合——术语层呈倒U，文档层呈单调")
        report_lines.append("")
        report_lines.append("新术语的加速增殖（T1→T2增长率 > 2x）表明概念发散在加速，但文档总数仍在增长。")
        report_lines.append("建议: 在单调子体系中补齐实现，在倒U子体系（概念/术语层）中做减法（合并哲学文档）。")
    elif term_growth_rate > 1.0:
        report_lines.append("")
        report_lines.append("### 判定: 単調——方向正确但速度在放缓")
        report_lines.append("")
        report_lines.append("术语持续增长但增速减缓。治理方向正确，补齐工程实现即可。")
    else:
        report_lines.append("")
        report_lines.append("### 判定: 倒U——T2已被越过")
        report_lines.append("")
        report_lines.append("新术语增长率 < 1x 但总量仍在增长，表明边际概念价值在递减。")
        report_lines.append("需要减法定向：合并哲学文档，审查不必要的术语。")
    
    report_lines += [
        "",
        "## 9. 局限",
        "",
        "- T0的数据可能不完整（早期commit可能不包含完整的文档快照）",
        "- 术语计数依赖关键词列表，可能遗漏新术语",
        "- 交叉引用只统计了DEPS章节，未统计正文中的引用",
        "- 收敛指标（3/3文档数）的增长可能只是时间累积效果，不一定反映收敛质量提升",
        "- 建议补充同文档追踪子实验",
    ]
    
    report_path = os.path.join(REPO, ".sih", "experiment_c_report.md")
    with open(report_path, "w") as f:
        f.write("\n".join(report_lines))
    
    print(f"Report written to {report_path}")
    print("\n".join(report_lines))

if __name__ == "__main__":
    main()
