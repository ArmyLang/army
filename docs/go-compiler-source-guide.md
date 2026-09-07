# Go 编译器源码阅读指南

## 入口

```
cmd/compile/main.go
  → gc.Main(archInit)
    → cmd/compile/internal/gc/main.go  (编译器总控中心)
```

`main.go` 只做三件事：校验构建配置 → 根据 GOARCH 选架构 → 调用 `gc.Main()`。真正的逻辑全在 `gc.Main()` 里。

## 编译管线总览

```
1. syntax Parse  →  语法树 (syntax AST)
2. types2 Check  →  类型标注
3. noder         →  编译器 IR (ir.Node 树)     ← "Unified IR"
4. 中端优化       →  内联、逃逸分析、去虚拟化
5. walk          →  语法糖降级 (desugaring)
6. ssagen        →  IR → SSA
7. ssa           →  SSA 优化 (~50 个 pass)
8. ssagen+obj    →  SSA → 机器码 → .o 文件
```

---

## 核心思想：传统流水线，但自研全部

Go 编译器是**经典的流水线架构**，和 Rust 的查询驱动完全不同：

| | Go 编译器 | Rust 编译器 |
|---|---|---|
| 架构 | 传统流水线，顺序执行 | 查询驱动，按需/惰性 |
| 入口 | `gc.Main()` 一次性跑完所有阶段 | `TyCtxt` 触发 query，谁需要谁算 |
| 缓存 | 无内置缓存（每次全量重编） | Query 系统天然缓存 + 增量 |
| 后端 | **完全自研**（SSA → obj） | 委托给 LLVM |

Go 的哲学很简单：**放弃增量编译的复杂度，换取极致的编译速度和全链路自主可控。**

---

## 分阶段阅读路线

### 第一阶段：基础设施（先读这 3 个包）

| 包 | 路径 | 核心文件 | 是什么 |
|---|---|---|---|
| **syntax** | `internal/syntax/` | `scanner.go`, `parser.go`(64KB), `nodes.go` | 编译器自己的 AST 定义 |
| **ir** | `internal/ir/` | `node.go`, `expr.go`, `func.go`, `stmt.go` | 编译器 IR 节点定义 |
| **base** | `internal/base/` | `flag.go`, `debug.go`, `print.go` | 编译基础设施（标志、调试、计时） |

Go 编译器**不使用标准库的 `go/ast`**，而是自己定义了一套 `syntax` AST + `ir` IR。先读 `nodes.go` 和 `node.go`，理解核心节点类型（`FuncDecl`, `CallExpr`, `AssignStmt` 等）。

### 第二阶段：前端（解析 + 类型检查）

```
syntax.Parse(file)          → token 流 → syntax AST
  |
  ▼
types2 (check.go)           → 类型检查，标注的 syntax AST
  |
  ▼
noder (reader.go 118KB)     → Unified IR 反序列化 → ir.Node 树
```

**关键文件**：

| 文件 | 看什么 |
|---|---|
| `internal/syntax/scanner.go` | 词法分析——理解 Go 的 token 定义 |
| `internal/syntax/parser.go` | 语法分析——递归下降解析器主体，函数 `parseFile()` 是入口 |
| `internal/types2/check.go` | 类型检查驱动——`Checker` 结构体和 `checkFiles()` 方法 |
| `internal/types2/expr.go` | 表达式类型检查——操作数类型推导 |
| `internal/noder/reader.go` | **Unified IR 的核心**——118KB，从序列化数据构建编译器 IR |

**重点理解 Unified IR**：类型检查完成后，不直接构建 IR，而是先序列化成 `pkgbits` 格式，再从序列化数据反序列化构建 IR。这样做是为了统一本地编译和导入外部包的代码路径——泛型支持使得这个统一变得必要。

### 第三阶段：中端优化

```
gc.Main() 中的调用顺序：
  inline + devirtualize     ← 内联 + 接口去虚拟化（交织进行）
  → escape                  ← 逃逸分析
  → deadlocals              ← 死局部变量消除
```

| 包 | 核心文件 | 做什么 |
|---|---|---|
| `internal/inline/` | `inl.go`(43KB) | 函数内联——决定哪些调用可以展开，并执行展开 |
| `internal/escape/` | `escape.go` | 逃逸分析——变量分配在栈还是堆？Go 编译器的灵魂 |
| `internal/devirtualize/` | — | 接口方法去虚拟化——把 `接口.方法()` 变成直接调用 |

**重点读 `escape.go`**——Go 最出名的编译器优化就在此。理解了逃逸分析的图构建和求解过程，就理解了 Go 内存管理的核心。

### 第四阶段：Walk（语法糖降级）

```
walk.Walk(funcIR)
  → order.go   (求值顺序规范化)
  → switch.go  (switch → 二分搜索/跳表)
  → range.go   (range → 初等循环)
  → builtin.go (map/channel 操作 → 运行时调用)
  → assign.go  (多赋值展开)
  → compare.go (比较操作展开)
```

| 文件 | 做什么 |
|---|---|
| `internal/walk/walk.go` | 入口 `Walk()` |
| `internal/walk/order.go` | **求值顺序规范化**——Go 规范要求求值从左到右，但 IR 可能不保证，这里强制排序 |
| `internal/walk/builtin.go` | 把 `make`, `append`, `delete`, `len` 等展开为运行时调用 |
| `internal/walk/range.go` | 把 `for range` 语法糖展开成普通 for 循环 |

Walk 的目的：**把高级语法全部转换成 SSA 能理解的简单形式。** Go 在 IR 层面保留高级结构，在 walk 阶段一步到位全降级。

### 第五阶段：SSA 后端（Go 的自研核心）

这是 Go 自研后端最精彩的部分。分两步：

#### 步骤 1：IR → SSA（ssagen）

| 文件 | 做什么 |
|---|---|
| `internal/ssagen/ssa.go`(265KB) | **IR → SSA 转换主入口**，`buildssa()` 函数遍历 IR 节点，逐一转换为 SSA 值和块 |
| `internal/ssagen/intrinsics.go`(98KB) | 内建函数 intrinsic 替换——像 `math.Sqrt` 直接映射为 `SQRT` 指令 |

#### 步骤 2：SSA 优化管道（ssa）

**入口**：`internal/ssa/compile.go` 中的 `passes` 数组——定义了约 50 个 pass 的执行顺序。

SSA pass 分 7 组，按顺序执行：

```
第 1 组：前期清理
  early phielim, early copyelim, early deadcode,
  short circuit, decompose user

第 2 组：通用优化
  opt, zero arg cse, generic cse, phiopt,
  nilcheckelim, prove, divisible, fuse

第 3 组：调用展开
  expand calls, decompose builtin, softfloat, branchelim

第 4 组：后期优化
  late opt, dead auto elim, sccp, generic deadcode,
  late fuse, dse, memcombine

第 5 组：架构 lowering ← 关键！
  writebarrier, lower, addressing modes, late lower, pair

第 6 组：lowered 后优化
  lowered cse, elim unread autos, lowered deadcode,
  checkLower, licm, tighten

第 7 组：代码生成准备
  late deadcode, critical, layout, schedule, regalloc
```

**重点 pass 说明**：

| Pass | 文件 | 做什么 |
|---|---|---|
| **opt** | `_opt.go` (自动生成) | 通用代数简化——`x+0→x`，`x*0→0` |
| **cse** | `cse.go` | 公共子表达式消除——`a+b` 算多次只算一次 |
| **prove** | `prove.go`(87KB) | 值域证明——推导变量的取值范围，消除边界检查 |
| **nilcheckelim** | `nilcheck.go` | 消除多余的 nil 检查——已经被证明非 nil 的指针不需要再检查 |
| **lower** | 架构特定的 rewrite 规则 | **架构 lowering**——将通用 SSA 操作匹配到具体 CPU 指令 |
| **regalloc** | `regalloc.go`(110KB) | 寄存器分配——将虚拟寄存器映射到物理寄存器 |
| **schedule** | `schedule.go` | 指令调度——乱序排列利用 CPU 流水线 |

### 第六阶段：输出

```
ssagen/pgen.go
  → 将 SSA 值转换为 obj.Prog 指令
  → cmd/internal/obj/  (汇编器框架)
  → 生成机器码 → 写入 .o 文件

gc/dumpobj()
  → 写入目标文件 + 导出数据 + DWARF + 反射元数据
```

---

## 建议的阅读顺序

```
1. cmd/compile/main.go                          ← 入口，2 分钟
2. internal/gc/main.go                          ← gc.Main() 总控，30 分钟
3. internal/syntax/nodes.go                     ← AST 节点定义
4. internal/syntax/parser.go                    ← 递归下降解析器，重点 func parseFile()
5. internal/ir/node.go                          ← IR 节点接口
6. internal/types2/check.go                     ← 类型检查驱动
7. internal/noder/reader.go                     ← Unified IR 反序列化（长但关键）
8. internal/escape/escape.go                    ← 逃逸分析（Go 编译器的灵魂）
9. internal/inline/inl.go                       ← 内联
10. internal/walk/walk.go + order.go            ← Desugaring 入口
11. internal/ssa/compile.go                     ← SSA pass 总调度
12. internal/ssagen/ssa.go                      ← IR → SSA 转换
13. internal/ssa/regalloc.go                    ← 寄存器分配
```

---

## Go 编译器独特的设计哲学

### 1. "快" 是一切的前提

Go 编译器放弃增量编译、放弃高级全局优化，只为两个字：**速度**。全量重编一个中型项目通常只需要几秒。

### 2. 全链路自研

不像 Rust 把后端外包给 LLVM，Go 从词法分析到机器码生成**全是自己写的**。代价是不能享受 LLVM 的激进优化，但换来：
- 编译速度极快
- 二进制文件完全自描述（不依赖外部链接器）
- 和运行时（GC、调度器）深度绑定

### 3. 双/三 AST 设计

```
syntax AST  ← 贴近源码，给 types2 用
    ↓
types2 类型标注
    ↓
ir.Node     ← 编译器 IR，给优化/代码生成用
```

三套表示看似多余，实际上各司其职：syntax 纯语法、types2 标注类型、ir 面向优化。互不干扰。

### 4. SSA 就是中间语言

Go 没有复杂的中间表示层次（不像 Rust 有 HIR、MIR、THIR）。IR 直接一步降到 SSA，然后在 SSA 上做所有后端优化。简单粗暴。

### 5. Unified IR 解决泛型

Go 1.18 引入泛型后，编译器的 `noder` 被重写为 Unified IR——类型检查后先序列化再反序列化。这样同一个泛型函数的不同实例化可以共享序列化数据，再由 reader 分别展开。

---

## 和 Rust 编译器的思想对比

| | Go 编译器 | Rust 编译器 |
|---|---|---|
| 架构 | 传统流水线 | 查询驱动 |
| 入口 | 一次性 `gc.Main()` 跑完 | `TyCtxt` 惰性求值 |
| 增量 | 不支持（每次全量重编） | 支持（query 缓存） |
| IR 层次 | syntax → ir → SSA（三层） | AST → HIR → MIR → LLVM IR（四层） |
| 后端 | 自研 SSA → obj | LLVM（可通过 trait 替换） |
| 优势 | 编译极快、全链路可控 | 运行效率极高、增量编译友好 |
| 代价 | 优化不够激进 | 编译速度慢 |

两个编译器代表了两种极端的设计哲学：Go 为编译速度牺牲一切，Rust 为执行性能和正确性牺牲编译速度。理解了这个，就理解了两种语言的设计理念。
