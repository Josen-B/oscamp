# x86_64 Hypervisor 构建和运行指南

本文档介绍如何构建和运行tour/h_5_0 - x86_64架构的虚拟化实现。

## 前置条件

1. 支持VT-x的x86_64系统
2. Rust工具链
3. QEMU (用于x86_64架构)

## 构建客户机镜像 (skernel-x86)

客户机镜像是运行在虚拟机中的简单内核。

### 方法1: 使用Makefile

```bash
cd payload/skernel-x86
make
```

### 方法2: 手动构建

```bash
cd payload/skernel-x86
cargo build --release --target x86_64-unknown-none
rust-objcopy --binary-architecture=x86_64 --strip-all -O binary target/x86_64-unknown-none/release/skernel-x86 skernel-x86
```

## 准备磁盘镜像

### 1. 创建disk.img (如果不存在)

```bash
# 在arceos根目录
make disk_img
```

### 2. 将客户机镜像写入disk.img

```bash
make payload
./update_disk.sh payload/skernel-x86/skernel-x86
```

注意：需要root权限来挂载disk.img。

## 运行虚拟机

### 使用QEMU运行

```bash
# 在arceos根目录
make run A=x86/h1 BLK=y
```

### 预期输出

你应该看到类似以下的输出：

```
x86_64 Hypervisor ...
VMX is supported!
Initializing VMX...
VMX revision: 0x...
VMXON region: 0x...
VMXON successful
Loading app: /sbin/skernel-x86
Guest image loaded at paddr: 0x...
Guest context prepared
Setting up VMCS...
Allocated VMCS at: 0x...
VMCS setup complete
Launching VM...
VM exit occurred
VM Exit - Reason: 0x30 (48)
  Exit qualification: 0x0
  Guest RIP: 0x100000
HLT - guest is halting
VM exited successfully!
Cleaning up VMX...
VMX cleanup complete
```

## 故障排除

### VMX not supported!

如果看到此错误，说明：
1. CPU不支持VT-x
2. VT-x在BIOS/UEFI中未启用
3. 运行环境不是支持虚拟化的x86_64系统

### Segfault或Crash

如果程序崩溃，请检查：
1. 是否使用正确的QEMU目标 (x86_64)
2. 是否启用了CPU虚拟化支持 (在QEMU中添加 `-enable-kvm` 或类似选项)

## 实现说明

### 当前功能

- ✅ VMX初始化 (VMXON/VMXOFF)
- ✅ VMCS配置
- ✅ 客户机镜像加载
- ✅ 基本的VM-exit处理
- ✅ 客户机正常退出

### 已知的VM-Exit处理

| 退出原因 | 处理方式 |
|---------|---------|
| Triple Fault (2) | 客户机关闭 |
| HLT (12) | 客户机暂停 |
| CR Access (10) | 忽略并继续 |
| I/O Instruction (28) | 忽略并继续 |
| RDMSR (48) | 忽略并继续 |
| WRMSR (49) | 忽略并继续 |

### 与RISC-V h_1_0的对应关系

| RISC-V概念 | x86_64概念 |
|-----------|-----------|
| H扩展 | VT-x (VMX) |
| sstatus/scause | VMCS中的控制字段 |
| hgatp (G-stage) | EPT Pointer |
| sret | VMLAUNCH/VMRESUME |
| 异常处理 | VM-exit处理 |

## 代码结构

```
x86/h1/
├── Cargo.toml          # Rust项目配置
├── Makefile            # 构建脚本
├── README.md           # 项目说明
├── BUILD_GUIDE.md      # 本文档
└── src/
    ├── main.rs         # 主程序入口
    ├── vcpu.rs         # VCPU状态定义
    ├── regs.rs         # 寄存器定义
    ├── vmx.rs          # VMX操作实现
    ├── loader.rs       # 客户机镜像加载器
    └── task.rs         # 任务扩展
```

## 进一步开发

如需扩展功能，可以考虑：

1. **更多的VM-exit处理**：处理更多类型的退出，如页错误、中断等
2. **设备虚拟化**：实现VirtIO设备模拟
3. **多VCPU支持**：支持多核虚拟机
4. **动态内存分配**：实现更灵活的EPT管理
5. **嵌套虚拟化**：支持在虚拟机中运行另一个虚拟机

## 参考

- Intel® 64 and IA-32 Architectures Software Developer's Manual, Volume 3C
- h_1_0 (RISC-V虚拟化实现)
