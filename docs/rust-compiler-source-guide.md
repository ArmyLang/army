# Rust 编译器源码阅读指南

## 入口

入口函数在 `compiler/rustc_driver_impl/src/lib.rs` 中的 `main()`，调用链：

```
compiler/rustc/main.rs (二进制入口)
  → rustc_driver_impl::main()          // CLI 参数解析、ICE 处理、信号设置
    → rustc_interface::run_compiler()  // 创建 Compiler{Session, CodegenBackend}
      → 编译管线开始
```

## 编译管线总览

```
1. parse        → AST
2. expand       → 宏展开后的 AST
3. HIR lowering → AST → HIR (高层中间表示)
4. typeck       → 类型检查 + 类型推断
5. MIR build    → HIR → MIR (中层中间表示)
6. borrowck     → 借用检查
7. MIR optimize → MIR 优化 pass
8. monomorphize → 单态化泛型
9. codegen      → MIR → LLVM IR → 目标代码
10. link         → 链接可执行文件
```

## 分阶段阅读路线

### 第一阶段：先掌握基础设施

| Crate | 负责 | 核心类型 |
|---|---|---|
| `rustc_span` | 源码位置（文件/行列） | `Span`, `SourceMap`, `Symbol` |
| `rustc_errors` | 诊断/错误/警告 | `DiagCtxt`, `EmissionGuarantee` |
| `rustc_session` | 编译配置/选项 | `Session`, `Options` |
| `rustc_middle` | **编译器中核** | `Ty`, `TyCtxt<'tcx>`, `Body<'tcx>`, `FnSig` |
| `rustc_query_impl` | 增量编译查询引擎 | query 定义、缓存、依赖追踪 |

先读 `Ty` 和 `TyCtxt`——整个编译器几乎每个函数都通过 `TyCtxt` 访问类型和 MIR，不理解它就寸步难行。

### 第二阶段：前端（按编译顺序）

```
rustc_lexer       → 词法分析，把源码切成 token 流
rustc_parse       → 语法分析，token → AST
rustc_expand      → 宏展开（macro_rules!, derive, proc_macro）
rustc_resolve     → 名称解析（name → DefId）
rustc_ast_lowering → AST → HIR（降级）
```

重点读 `rustc_parse`——理解 AST 结构就能看懂后续所有处理。

### 第三阶段：中端（类型系统 + 借用检查）

```
rustc_hir            → HIR 数据结构定义
rustc_infer          → 类型推断（类型变量、合一、subtyping）
rustc_hir_typeck     → HIR 上的类型检查（FnCtxt 是核心）
rustc_trait_selection → Trait 求解（impl 匹配、chalk 集成）
rustc_mir_build      → HIR → MIR 的构造
rustc_borrowck       → 借用检查器（核心算法：NLL/Polonius）
rustc_mir_transform  → MIR 优化 pass（内联、常量折叠等）
rustc_const_eval     → 常量求值（MIR 解释器）
rustc_monomorphize   → 泛型单态化
```

重点读 `rustc_borrowck`——Rust 的灵魂在此，理解 NLL borrow checker 就真正懂了 Rust。

### 第四阶段：后端

```
rustc_codegen_ssa    → CodegenBackend trait（解耦后端的抽象层）
rustc_codegen_llvm   → LLVM 后端实现（Builder 模式生成 LLVM IR）
```

重点读 `CodegenBackend` trait 的方法签名，然后看 `rustc_codegen_llvm` 怎么把 MIR 翻译成 LLVM IR。

### 第五阶段：查询系统（横向贯穿）

`rustc_query_impl` 不是单独一步，而是**贯穿整个编译管线**的机制。每个编译步骤（typeck、borrowck、mir_build 等）都是一个 query，通过 `TyCtxt` 惰性求值：

```rust
tcx.type_of(def_id)      // 触发 type_of query
tcx.optimized_mir(def_id) // 触发 optimized_mir query
```

这套系统支撑了增量编译——query 结果被缓存，输入不变就不会重新计算。

## 建议的阅读顺序

```
1. compiler/rustc/main.rs                    ← 入口，5 分钟
2. compiler/rustc_driver_impl/src/lib.rs      ← main() 函数，10 分钟
3. rustc_interface                             ← run_compiler()，看 Session 创建
4. rustc_middle/src/ty/mod.rs                  ← Ty/TyKind/TyCtxt 核心类型
5. rustc_middle/src/mir/mod.rs                 ← MIR Body 定义
6. rustc_parse/src/parser/mod.rs               ← 解析器主体
7. rustc_hir_typeck/src/fn_ctxt/mod.rs         ← FnCtxt，类型检查核心
8. rustc_borrowck/src/lib.rs                   ← 借用检查入口
9. rustc_codegen_llvm                            ← LLVM 后端
```

第一遍不求深，跑通链路就行。第二遍再细读每个阶段的算法细节。
