# x86_64虚拟化实现总结 (h_5_0)

## 概述

本项目实现了x86_64架构的虚拟化功能，参考了RISC-V h_1_0的实现思路，使用Intel VT-x (VMX) 技术创建和管理虚拟机。实现达到了与h_1_0相同的功能级别：能够托起客户机并正常退出。

## 实现的核心功能

### 1. VMX初始化
- 检查CPU是否支持VT-x
- 检查并启用VMX在IA32_FEATURE_CONTROL MSR中
- 设置CR4的VMXE位
- 分配VMXON区域并执行VMXON指令
- 执行VMXOFF指令进行清理

### 2. VMCS管理
- 分配和初始化VMCS (Virtual Machine Control Structure)
- 配置VMCS控制字段：
  - Pin-based VM-execution controls
  - Primary processor-based VM-execution controls
  - VM-exit controls
  - VM-entry controls
  - EPT pointer

### 3. 客户机状态设置
- 配置客户机的通用寄存器 (RAX, RCX, RDX, RBX, RSP, RBP, RSI, RDI, R8-R15)
- 配置控制寄存器 (CR0, CR2, CR3, CR4)
- 设置指令指针 (RIP) 和标志寄存器 (RFLAGS)
- 配置段寄存器 (CS, DS, ES, FS, GS, SS, LDTR, TR)
- 设置GDTR和IDTR
- 配置活动状态和可中断性状态

### 4. 宿主机状态设置
- 配置宿主机的CR0, CR3, CR4
- 设置宿主机的段选择器和基址
- 配置宿主机的RSP和RIP

### 5. 客户机镜像加载
- 从文件系统加载客户机二进制文件
- 将客户机镜像映射到客户机的虚拟地址空间
- 设置客户机的入口点

### 6. VM-Exit处理
实现了以下VM-Exit处理程序：
- **Triple Fault (退出原因2)**：客户机关闭，hypervisor正常退出
- **HLT (退出原因12)**：客户机暂停
- **CR Access (退出原因10)**：忽略并继续执行
- **I/O Instruction (退出原因28)**：忽略并继续执行
- **RDMSR (退出原因48)**：忽略并继续执行
- **WRMSR (退出原因49)**：忽略并继续执行

## 文件结构

```
tour/h_5_0/
├── Cargo.toml          # Rust项目配置
├── Makefile            # 构建脚本
├── README.md           # 项目说明
├── BUILD_GUIDE.md      # 详细构建指南
├── IMPLEMENTATION_SUMMARY.md  # 本文档
└── src/
    ├── main.rs         # 主程序入口，协调虚拟化初始化和执行流程
    ├── vcpu.rs         # VCPU状态结构定义
    ├── regs.rs         # 寄存器索引和访问
    ├── vmx.rs          # VMX操作实现（核心文件）
    ├── loader.rs       # 客户机镜像加载器
    └── task.rs         # 任务扩展数据结构
```

## 核心实现文件说明

### src/main.rs
- `main()`: 主函数，协调所有虚拟化步骤
- `prepare_guest_context()`: 准备客户机上下文
- `run_guest()`: 启动客户机执行

### src/vcpu.rs
- `GuestState`: 客户机CPU状态结构
- `HypervisorState`: 宿主机CPU状态结构
- `VmCpuRegisters`: 完整的VCPU寄存器状态

### src/regs.rs
- `GprIndex`: x86_64通用寄存器索引枚举
- 寄存器访问辅助函数

### src/vmx.rs
- `check_vmx_support()`: 检查VMX支持
- `vmx_init()`: 初始化VMX
- `setup_vmcs()`: 设置VMCS
- `vmx_launch()`: 启动虚拟机
- `vmexit_handler()`: 处理VM-Exit事件
- VMX相关的汇编指令封装 (VMXON, VMXOFF, VMLAUNCH, VMWRITE, VMREAD等)

### src/loader.rs
- `load_vm_image()`: 加载客户机镜像到客户机地址空间

### src/task.rs
- `TaskExt`: 任务扩展结构
- 定义hypervisor的任务扩展

## 与RISC-V h_1_0的对应关系

| RISC-V概念 | x86_64概念 | 实现位置 |
|-----------|-----------|---------|
| H扩展 | VT-x (VMX) | vmx.rs |
| sstatus/scause | VMCS控制字段 | vmx.rs |
| hgatp (G-stage) | EPT Pointer | vmx.rs |
| sret | VMLAUNCH/VMRESUME | vmx.rs |
| 异常处理 | VM-Exit处理 | vmx.rs/vmexit_handler() |
| 手动保存寄存器 | VMCS自动保存 | vcpu.rs/vmx.rs |
| CSR访问 | VMREAD/VMWRITE | vmx.rs |
| SBI调用 | (未实现) | - |

## 客户机实现 (skernel-x86)

客户机是一个极简的内核，包含：
- 基本的程序入口点 (`_start`)
- 故障处理（通过HLT或无效操作导致VM-exit）
- Panic处理器

### 客户机行为

客户机通过执行HLT指令导致VM-exit，hypervisor捕获这个退出并正确处理，然后正常退出。

## 技术亮点

1. **完整的VMX生命周期管理**：从VMXON到VMXOFF的完整流程
2. **灵活的VMCS配置**：支持多种控制字段的配置
3. **健壮的错误处理**：检查VMX支持，处理各种VM-exit情况
4. **清晰的代码结构**：模块化设计，易于理解和扩展
5. **详细的文档**：包含README、BUILD_GUIDE等文档

## 使用方法

### 1. 构建客户机
```bash
cd payload/skernel-x86
make
cd ../..
```

### 2. 准备磁盘镜像
```bash
make disk_img
make payload
./update_disk.sh payload/skernel-x86/skernel-x86
```

### 3. 运行虚拟机
```bash
make run A=tour/h_5_0 BLK=y LOG=info
```

## 预期输出

```
x86_64 Hypervisor ...
VMX is supported!
Initializing VMX...
VMX revision: 0x4002c1
VMXON region: 0x7f7c000
VMXON successful
Loading app: /sbin/skernel-x86
Guest image loaded at paddr: 0x8000
Guest context prepared
Setting up VMCS...
Allocated VMCS at: 0x7f80000
VMCS setup complete
Launching VM...
VM exit occurred
VM Exit - Reason: 0xc (12)
  Exit qualification: 0x0
  Guest RIP: 0x100000
HLT - guest is halting
VM exited successfully!
Cleaning up VMX...
VMX cleanup complete
```

## 限制和未来工作

### 当前限制
1. 仅支持基本的VM-exit处理
2. 没有实现设备虚拟化
3. 没有实现中断虚拟化
4. 仅支持单VCPU
5. EPT配置相对简单

### 未来改进方向
1. **增强VM-Exit处理**：支持更多退出类型
2. **设备虚拟化**：实现VirtIO设备模拟
3. **中断虚拟化**：实现APIC虚拟化
4. **多VCPU支持**：支持多核虚拟机
5. **动态内存管理**：实现更灵活的EPT管理
6. **嵌套虚拟化**：支持在虚拟机中运行虚拟机

## 参考资料

- Intel® 64 and IA-32 Architectures Software Developer's Manual, Volume 3C
- Intel VT-x技术规范
- h_1_0 (RISC-V虚拟化实现)
- ArceOS项目文档

## 总结

本项目成功实现了x86_64架构的基本虚拟化功能，达到了与RISC-V h_1_0相同的功能级别。通过使用Intel VT-x技术，我们能够：
- 初始化虚拟化环境
- 创建和配置虚拟机
- 加载并执行客户机代码
- 正确处理VM-exit事件
- 实现客户机的正常退出

这个实现为后续更复杂的虚拟化功能（如设备虚拟化、多VCPU支持等）奠定了坚实的基础。
