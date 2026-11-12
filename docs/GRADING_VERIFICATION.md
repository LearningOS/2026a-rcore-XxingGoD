# 远端评测与成绩同步核验

## 核验范围

仓库：`LearningOS/2026a-rcore-XxingGoD`。课程 ID：`2073`。以下结果由 GitHub Actions 作业状态、`gh-pages` 分支的实际成绩记录及最终上传作业日志交叉核对，不以本地测例通过替代远端成绩同步。

各次运行的 `Test chapter and reports` 和 `Save progress and upload score` 均为 `success`：

| 分支 | 已通过的提交 | 检查器结果 | 章节成绩 | Actions 运行 |
| --- | --- | --- | ---: | --- |
| ch3 | `5e353ac144b9a6426ac117683d707015c9bf84f6` | 7/7 | 100 | [43160834334](/LearningOS/2026a-rcore-XxingGoD/actions/runs/43160834334) |
| ch4 | `af6ebfd71f628583c64995f55c641678ee2e1c48` | 16/16 | 100 | [43160834211](/LearningOS/2026a-rcore-XxingGoD/actions/runs/43160834211) |
| ch5 | `c1004b19cadb0fd40ea90bb017dd872b26a312df` | 15/15 | 100 | [43160833964](/LearningOS/2026a-rcore-XxingGoD/actions/runs/43160833964) |
| ch6 | `0326751ddc1ff78440eb058a6a73e9f9aec02779` | 31/31 | 100 | [43160834019](/LearningOS/2026a-rcore-XxingGoD/actions/runs/43160834019) |
| ch8 | `9ec0251528a4408cff4cf2e6ea7fa592292b9850` | 25/25 | 100 | [43160833970](/LearningOS/2026a-rcore-XxingGoD/actions/runs/43160833970) |

## 累计成绩与上传回执

实际读取的 [gh-pages/course-2073.json](/LearningOS/2026a-rcore-XxingGoD/blob/gh-pages/course-2073.json) 记录五章的完整通过结果和以上提交，关键字段为：

```json
{
  "courseId": 2073,
  "repository": "LearningOS/2026a-rcore-XxingGoD",
  "user": "XxingGoD",
  "score": 500,
  "totalScore": 500,
  "upload": {
    "status": "accepted",
    "runId": "43160833970",
    "runAttempt": "1"
  }
}
```

最终上传作业 `130992041939` 的日志记录：

```text
Submitting measured score: course=2073, user=XxingGoD, score=500/500
OpenCamp accepted the score (result=1).
```

该成功回执位于 `2026-11-12T05:23:30.9650272Z` 的日志行。上传后，成绩状态已写回 `gh-pages`，确认提交为 `11d519a`。因此可以确认：五章远端评测通过，累计 500/500 已提交并被 OpenCamp 接口接收，而不只是工作流被触发或本地测试通过。

核验未登录学员个人成绩页面，未独立验证页面缓存或展示状态。若页面显示延迟，应将上述运行与回执交给课程方核查，不应把页面未核实表述为接口未接收。自动报告检查只验证文件存在，不代替课程对报告内容和诚信声明的人工审核。

## 汇总方式

本次只将 `reports/lab1.md` 至 `reports/lab5.md`、分支导航和本核验记录汇集到 `main`。五章源码和已通过评测的提交仍分别保留在 `ch3`、`ch4`、`ch5`、`ch6`、`ch8` 分支；未合并不同章节的内核代码，也未更改成绩发布脚本或成绩文件。

## 复核命令

在已授权该仓库的 GitHub CLI 环境中运行：

```sh
gh run list --repo LearningOS/2026a-rcore-XxingGoD --workflow build.yml --limit 10
gh run view 43160833970 --repo LearningOS/2026a-rcore-XxingGoD --json jobs
gh run view 43160833970 --repo LearningOS/2026a-rcore-XxingGoD --job 130992041939 --log
gh api 'repos/LearningOS/2026a-rcore-XxingGoD/contents/course-2073.json?ref=gh-pages' -H 'Accept: application/vnd.github.raw+json'
```
