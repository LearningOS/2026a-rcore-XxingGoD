# Lab 1：多道程序与分时多任务

## 功能总结

实现 sys_trace 的单字节读写和按任务统计系统调用次数。在 syscall 分发前更新计数，保证查询包含本次调用；使用当前任务编号隔离计数，查询超出计数表范围的编号返回 0。第三章遵循题意不检查用户地址权限，后续地址空间实验再完善。

## 实验环境与实际观察

工具链为 nightly-2024-05-02，目标 riscv64gc-unknown-none-elf，RustSBI 0.4.1，QEMU 11.1.3。补充报告后实际运行官方 `make -C ci-user test CHAPTER=3`：检查 7/7 通过，输出 `Report for lab1 found.`，命令退出码 0。此前缺少报告的失败已解决；报告检查只验证文件存在，不代表人工审核。

日志显示 bad_address 访问 0x0 导致 PageFault，bad_instructions 和 bad_register 分别触发 IllegalInstruction，内核终止对应任务，其余计算、yield、sleep 和 trace 程序继续执行。异常任务没有使其他任务一并终止，体现 U 态特权限制及任务级错误处理。

## Trap 汇编问答

### 1. __restore 的栈指针和两种使用情景

进入 __restore 时 sp 指向内核栈中的 TrapContext，其布局为 x[0..31]、sstatus、sepc。首次启动应用时，内核构造初始上下文，经任务切换进入 __restore；应用触发系统调用、异常或中断后，保存上下文，处理完成也经 __restore 返回用户态。两者共用恢复路径。

### 2. 特殊处理的寄存器

先从上下文取出 sstatus、sepc 和保存的 x2。sstatus 保存特权及中断状态，其中 SPP 决定 sret 返回的特权级；初始上下文明确把 SPP 设为 User。sepc 是返回执行的地址，系统调用处理会跳过 ecall 再恢复。保存的 x2 是用户栈指针，先写入 sscratch，稍后交换到 sp，避免提前失去访问内核栈上下文的能力。

### 3. 为什么跳过 x2 和 x4

x2 是 sp，需要最后通过交换指令恢复，不能按普通寄存器提前加载。x4 是 tp，本章框架明确假定应用不使用它，因此不保存恢复；这不是通用操作系统可以无条件省略 tp 的结论。

### 4. __restore 的 csrrw 之后

在交换前，sp 已释放 TrapContext，指向该任务内核栈顶，sscratch 为用户栈指针。`csrrw sp, sscratch, sp` 之后，sp 指向用户栈，sscratch 保存内核栈顶，供下一次 Trap 进入内核时交换使用。

### 5. 返回用户态的指令

`sret` 根据 sstatus.SPP 恢复特权级，并跳转到 sepc。这里 SPP 为 U，因此进入用户态；单独执行寄存器交换不会改变特权级。

### 6. __alltraps 开头交换之后

进入汇编时硬件已经进入 S 态，但 sp 仍是用户栈指针。交换后 sp 为内核栈顶，sscratch 为用户栈指针。随后在内核栈分配 34×8 字节保存 TrapContext，并把用户 sp 保存到 x[2]。

### 7. U 态何时进入 S 态

系统调用执行 ecall 触发异常，硬件依据异常委托配置进入 S 态并跳到 stvec；非法访问、非法指令及已委托中断也可以触发相同硬件过程。不是 __alltraps 的 csrrw 或 call 完成特权转换。

## 荣誉准则与参考资料

1. 交流记录：由提交者依据课程荣誉准则据实确认个人交流对象及内容。
2. 参考资料：本仓库 ch3 框架、trap.S、TrapContext、官方 ch3_trace 测例、rCore 实验指导第三章练习及荣誉准则。未通过修改官方测试规避评测。
3. 个人独立完成情况与诚信声明应由提交者本人据实确认，本报告不代签个人声明。
4. 实验实现仅用于教学内核，不以破坏评测或其他计算机系统为目的。源码的保管、公开及正式提交应遵守课程荣誉准则；自动报告存在检查不等于人工诚信审核通过。

资料：
- https://learningos.cn/rCore-Tutorial-Guide/chapter3/5exercise.html
- https://learningos.cn/rCore-Tutorial-Guide/honorcode.html
