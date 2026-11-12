# Lab 3：进程管理与 stride 调度

## 功能总结

spawn 直接加载 ELF 构造子进程，设置父子关系并入就绪队列，不复制父地址空间。增加优先级及 stride，初始分别为 16 和 0；每次选择 stride 最小的就绪任务并累加 BIG_STRIDE/priority。迁移跨页 get_time 和匿名 mmap/munmap，支持前章非 trace 测例。

## 实现说明

现有 VecDeque 保持稳定的相同 stride 次序，线性扫描选取最小值，不引入额外堆结构。BIG_STRIDE 为 2^63；合法优先级是正 isize 且至少为 2，因此步长至少为 1、至多为 2^62。使用 wrapping_add 更新，用差的有符号解释判断先后，避免无符号回绕直接比较错误。父子关系使用 Weak/Arc 避免所有权环，waitpid 回收仍沿用框架。exec 不改变进程调度权重，fork 和 spawn 的新进程按题意初始化。

## 问答：stride 溢出

### 8 位例子

p1=255，p2=250，pass=10。运行 p2 后回绕成 4。直接比较无符号整数会认为 4<255，再选 p2，违背预期；真实累计进度应视为 p2=260，下一次应选 p1。

### 为什么最大最小差不超过 BIG_STRIDE/2

在任务集合固定、初始 stride 一致且不考虑溢出的条件下，最小 stride 的任务每次增加最多 BIG_STRIDE/2。若更新后仍未超过旧最大值，差距不增加；若超过旧最大值，新最大值到其余任务最小值的距离也不超过本次增量。由归纳可得界限。动态加入初始 stride=0 的新任务并不自动满足这一固定集合证明的假设，必须单独考虑；本实验按要求新进程初始为 0。

### 溢出比较器

在任意两个待比较计数的真实距离严格小于计数空间一半时，可把模减结果看作有符号距离。BinaryHeap 是最大堆，需要反转自然的进度次序，让最小进度先弹出：

```rust
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let distance = self.0.wrapping_sub(other.0) as i64;
        Some(if distance < 0 {
            core::cmp::Ordering::Greater
        } else if distance > 0 {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Equal
        })
    }
}
```

若定义的是通常“小于”关系而非最大堆排序，则去掉反转：distance<0 时返回 Less。题目的 8 位提示中 125 到 255 的模差为 126（正），因此 125 不在 255 之前；129 到 255 的模差为 130，解释为 i8 是 -126，所以 129 在前。真实实现若要放入 BinaryHeap，还必须一致实现 Eq/Ord，而不能把相等性与比较器矛盾地定义；题目假定无相等值仅用于简化讨论。

## 验证

实际运行 `make -C ci-user test CHAPTER=5`：检查 15/15 通过，lab1 至 lab3 均被检出，命令退出码 0；`ch5 Usertests passed`，stride 子测例退出码 0。六组 runtime/priority 比值最小为 4,952,514，最大为 5,103,520，最大/最小约 1.031，小于标准 1.5。spawn、wait、时间和匿名映射的前向兼容测例也通过。自动报告检查只验证文件存在。

## 荣誉准则与来源

参考本仓库框架、rCore 官方第五章练习及第四章地址空间规范。个人交流记录、独立完成情况和诚信声明须由提交者按课程荣誉准则据实确认。不修改测试规避要求，不破坏系统；源码保管和公开遵循课程规则，自动报告存在检查不等于人工诚信审核。

- https://learningos.cn/rCore-Tutorial-Guide/chapter5/4exercise.html
- https://learningos.cn/rCore-Tutorial-Guide/honorcode.html
