# The Army Language Specification

> 版本 0.1.0 | 2026-08-06

## 目录

1. [引言](#1-引言)
2. [记号](#2-记号)
3. [源文件组织](#3-源文件组织)
4. [词法元素](#4-词法元素)
5. [类型系统](#5-类型系统)
6. [表达式](#6-表达式)
7. [语句](#7-语句)
8. [声明与作用域](#8-声明与作用域)
9. [Struct](#9-struct)
10. [Interface](#10-interface)
11. [函数](#11-函数)
12. [泛型](#12-泛型)
13. [包系统](#13-包系统)

---

## 1. 引言

Army 是一门静态类型的编译型语言，融合了 C/Java 系的声明风格、Go 系的接口组合与鸭子类型、以及现代函数式特性（Lambda、匿名函数、方法引用）。

文件后缀：`.army`
基包定义文件：`base-info.army`

---

## 2. 记号

### 2.1 硬关键字（45 个）

#### 基本关键字（27 个）

```
break        default      func         interface    select
case         defer        go           map          struct
chan         else         goto         package      switch
static       final        if           range        type
continue     for          import       return       var
```

#### 类型关键字（18 个）

```
byte         int2         int4         int8
ubyte        uint2        uint4        uint8
int          uint
float4       float8
complex8     complex16
uintptr
rune
string       decimal
```

> **注意**：
> - Army 无 `const` 关键字，常量由 `final` 表达（`final` 的字段/变量不可重新赋值）。`static` 与 `final` 正交：
>   - `final` 实例字段 — 实例常量（每个实例可不同值，但构造后不可变）
>   - `static final` — 类级常量（所有实例共享）
>   - `final` 局部变量 — 局部不可变变量
> - Army 无 `fallthrough` 关键字，switch 各 case 默认穿透（C 风格），必须显式 `break`。
> - 类型名是硬关键字，不可被 shadow（与 Go 不同，Go 中类型名是预声明标识符）。
> - `bool`、`error`、`any` 是预声明标识符（非关键字），可被 shadow（不推荐）。

### 2.2 上下文关键字（1 个）

- `permits` — 仅出现在 `struct` 或 `interface` 名称之后，用于密封类型白名单。在其他位置可作为普通标识符。

### 2.3 运算符与分隔符

```
+    -    *    /    %
==   !=   <    >    <=   >=
=    :=   .    ,    ;
(    )    {    }    [    ]
::   #    &    |    ...
```

### 2.4 字面量

- **整数**: `0`, `42`, `-1`, `0xFF`, `0b1010`
- **浮点**: `3.14`, `1.0e10`, `0.5f4`, `0.5f8`
- **字符串**: `"hello"`, 支持 `\n`, `\t` 等转义
- **布尔**: `true`, `false`
- **空值**: `nil`

### 2.5 注释

```army
// 单行注释
/* 多行注释 */
```

### 2.6 文档注释

Army 使用 `///` 开头的**文档注释**，紧随被注释的声明之前。与 Java 25 一致，文档注释使用 **Markdown** 格式，**禁止 HTML 标签**：

```army
/// Returns the larger of two values.
///
/// # Parameters
/// - `a` — the first value
/// - `b` — the second value
///
/// # Returns
/// The greater of `a` and `b`.
///
/// # Examples
/// ```
/// int v := maxOf(3, 7)   // v == 7
/// ```
func maxOf(int a, int b) int {
    return a > b ? a : b
}

/// A thread-safe counter with an upper bound.
///
/// # Fields
/// - `count` — current value
/// - `limit` — maximum allowed value, defaults to **1024**
struct Counter {
    int count
    int limit = 1024
}

/// Represents a drawable shape.
///
/// Implementing types must provide
/// a `draw()` method.
interface Drawable {
    func draw()
}
```

规则：

- `///` 必须紧跟被注释的声明（`func`、`struct`、`interface` 等）
- 使用 Markdown 格式：标题 `#`、代码块 `` ``` ``、列表 `-`、加粗 `**`、内联代码 `` ` ``
- **禁止 HTML 标签**（`<b>`、`<code>` 等一律非法）
- 编译器可提取文档注释生成 API 文档

---

## 3. 源文件组织

### 3.1 分区思想

`.army` 文件按**区域（区）**组织，每个区内部必须连续，不能交错：

1. **struct 区** — 所有 struct 定义及紧跟的方法
2. **interface 区** — 所有 interface 定义及紧跟的静态方法
3. **全局 func 区** — 所有顶级函数
4. **全局变量区** — 所有顶级变量声明

各区必须按上述顺序出现，同一区内的定义不能散布到其他区。

### 3.2 基包定义

`base-info.army` 文件用于声明项目的基包：

```army
com.qinarmy.demo
```

---

## 4. 词法元素

### 4.1 大括号风格

Army 遵循 Go 的大括号规则：**左大括号 `{` 必须与声明/语句在同一行**，不允许单独换行（C 风格非法）。

```army
// ✅ Go 风格 — { 与声明同行
if x > 0 {
    return x
}

func maxOf(int a, int b) int {
    return a > b ? a : b
}

struct Point {
    int x
    int y
}

for int i := 0; i < n; i = i + 1 {
    print(i)
}

// ❌ C 风格 — { 另起一行，编译错误
if x > 0
{
    return x
}

func maxOf(int a, int b) int
{
    return a > b ? a : b
}
```

### 4.2 缩进

Army 使用 **4 个空格** 缩进，**禁止 Tab 字符**。编辑器应将 Tab 键映射为输入 4 个空格。

```army
// ✅ 正确——4 空格缩进
func maxOf(int a, int b) int {
    if a > b {
        return a
    }
    return b
}

// ❌ 错误——Tab 字符
func maxOf(int a, int b) int {
→   if a > b {          // Tab 字符非法
→   →   return a
→   }
    return b
}

// ❌ 错误——2 空格缩进
func maxOf(int a, int b) int {
  if a > b {            // 2 空格非法
    return a
  }
  return b
}
```

### 4.3 声明风格

类型在左，变量在右（C/Java 系）：

```army
int x := 10          // 声明 + 推断
int y = 20           // 显式类型声明
float f := 3.14
string s := "hello"
bool b := true
```

#### 4.3.1 类型与变量间的空格

**类型与变量名之间有且仅有一个空格**，多或少的空格均编译错误：

```army
// ✅ 正确——类型与变量间恰好 1 个空格
int x := 10
string name := "army"

// ❌ 错误——多余空格
int  x := 10        // 两个空格，编译错误
string   s := "hi"  // 三个空格，编译错误

// ❌ 错误——缺少空格
intx := 10          // 无空格，intx 被当作标识符而非类型
```

此规则同样适用于函数参数、struct 字段、返回值类型声明等所有"类型 + 名称"出现的位置。

#### 4.3.2 函数参数列表的格式

##### 括号

左括号 `(` 左右均**无空格**，右括号 `)` 左侧**无空格**。若参数换行，左括号后可直接换行，但参数列表内**不允许有空行**：

```army
// ✅ 正确——单行：括号紧贴名称
func add(int a, int b) int {
    return a + b
}

// ✅ 正确——换行：左括号后直接换行，无空行
func configure(
    string host,
    int port,
    bool tls,
    int timeout
) Config {
    // ...
}

// ❌ 错误——左括号左侧有空格
func add (int a, int b) int { ... }         // "add (" 非法

// ❌ 错误——左括号右侧有空格
func add( int a, int b) int { ... }         // "( " 非法

// ❌ 错误——右括号左侧有空格
func add(int a, int b ) int { ... }         // "b )" 非法

// ❌ 错误——参数列表内出现空行
func configure(
    string host,
    int port,

    bool tls,                               // 空行前移——编译错误
    int timeout
) Config { ... }

// ✅ 正确——调用时也同样规则
send(host, port, path, timeout)
send(
    host,
    port,
    path,
    timeout
)
)
```

##### 逗号

参数列表中，逗号**左右各有且仅有一个空格**。换行时**只能在逗号之后换行**，不可在逗号之前换行：

```army
// ✅ 正确——单行：逗号前后各 1 个空格
func add(int a, int b) int {
    return a + b
}

// ✅ 正确——换行：逗号在行尾（逗号后换行）
func send(string host, int port,
          string path, int timeout) {
    connect(host, port)
}

// ❌ 错误——逗号前换行（逗号在行首）
func send(string host , int port
    , string path , int timeout) { ... }

// ❌ 错误——逗号前缺少空格
func add(int a,int b) int { ... }

// ❌ 错误——逗号后缺少空格
func add(int a,int b) int { ... }

// ❌ 错误——逗号前多余空格
func add(int a  , int b) int { ... }

// ❌ 错误——逗号后多余空格
func add(int a,  int b) int { ... }
```

此规则同样适用于函数调用实参列表、多返回值声明（如 `func foo() (int, string)`）和多字段 struct 定义。

#### 4.3.3 设计动机

Army 将空格和逗号格式提升为**编译期强制规则**而非风格建议，目的与 Go 的 `gofmt` 一致：

- **消除风格不一致**：不同开发者不会写出不同的空格习惯
- **减少 git diff 噪音**：提交历史中不会出现无意义的格式改动，`git blame` 指向真正修改代码的人
- **Code Review 更高效**：reviewer 不必纠结于格式问题，编译器已经保证了统一
- **大型项目协作友好**：任何人不需记忆风格指南，编译器直接拒绝不符合规范的代码

> **原则：能在编译期杜绝的差异，就不要留给 lint 工具或 code review 去反复拉扯。**

### 4.4 可见性（6 种）

| 可见性     | 记号           | 说明                 |
|------------|----------------|----------------------|
| public     | 无后缀（默认） | 任何地方可访问       |
| 基包       | `#`            | 整个基包可见         |
| 本包及子包 | `$`            | 本包及其子包可见     |
| 本包       | `~`            | 仅本包可见           |
| private    | `-`            | 仅本文件可见         |
| 降级       | `_` 前缀       | 类型自身降一级可见性 |

用法示例：

```army
struct# FrameworkHelper { ... }     // 基包可见
struct$ PackageService { ... }      // 本包及子包
struct~ InternalUtil { ... }        // 本包可见
struct- Config { ... }              // 本文件可见

func# (Counter) audit(Order o) bool { ...  }
func~ (Counter c) resetLocal() { ... }
func- (Counter c) internalCheck() { ... }

// 降级可见性：_ 前缀将类型从 public 降为基包可见
struct _Person {
    string name                    // 字段仍是 public
}
// 外部可访问 p.name()，但不能声明 _Person 类型的变量
```

---

## 5. 类型系统

### 5.1 内置类型

| 类型      | 说明                   |
|-----------|------------------------|
| `byte`    | 1 字节有符号整数       |
| `int2`    | 2 字节有符号整数       |
| `int4`    | 4 字节有符号整数       |
| `int8`    | 8 字节有符号整数       |
| `int`     | 平台相关有符号整数     |
| `ubyte`   | 1 字节无符号整数       |
| `uint2`   | 2 字节无符号整数       |
| `uint4`   | 4 字节无符号整数       |
| `uint8`   | 8 字节无符号整数       |
| `uint`    | 平台相关无符号整数     |
| `uintptr` | 足够存指针的无符号整数 |
| `float4`   | 4 字节浮点             |
| `float8`   | 8 字节浮点             |
| `complex8` | 8 字节复数（real: float4, imag: float4） |
| `complex16` | 16 字节复数（real: float8, imag: float8） |
| `rune`    | Unicode 码点           |
| `string`  | 不可变字符串           |
| `decimal` | 精确小数               |
| `bool`    | 布尔值                 |
| `error`   | 错误类型               |
| `any`     | 空接口，可表示一切值     |

> 所有类型名是硬关键字，不能被 shadow。

### 5.2 数组

`[]` 必须紧跟类型名之后，不允许跟在变量名后面。数组长度可以是变量（运行时确定），不需要编译期常量：

```army
int[] numbers := [1, 2, 3]          // ✅
string[] names := ["foo", "bar"]    // ✅

int n := computeSize()
int[] values := [n]                 // ✅ 长度由变量 n 确定

string array[]                      // ❌ [] 不能跟在变量名后
```

### 5.3 Map

```army
map<string, int> scores := map<string, int>{"alice": 95, "bob": 87}
```

### 5.4 空值

`nil` 是所有引用类型和接口类型的零值。

---

## 6. 表达式

### 6.1 三元运算符

```
条件 ? 真值 : 假值
```

规则：

1. **不可换行** — 整个三元表达式必须在同一行
2. **不可函数调用** — 三个部分都不能包含函数调用
3. **至多一个运算符** — 条件、真值、假值三个部分各自至多允许一个运算符
4. **禁止嵌套** — 三元运算符不可嵌套另一个三元运算符

```army
// ✅ 合法
int result := a > b ? a : b
string label := ok ? "done" : "pending"
int v := x >= 0 ? x : -x
bool ready := notEmpty ? true : false

// ❌ 非法
int v := check() ? a : b                     // 函数调用
int v := a > b                               // 换行
    ? processA() : processB()
int v := a + b > c ? d * 2 : e               // 条件部分有两个运算符（+ 和 >）
int v := a > b ? b > c ? d : e : f           // 嵌套三元运算符
```

### 6.2 构造语法

```army
Point{int x: 10, int y: 20}
Price{decimal amount: 12.99, string currency: "CNY"}
Box<int>{int value: 42}

// 嵌套匿名组合
Dog{Pet{Animal{string species: "canine", int age: 3}, string name: "Buddy"}, string breed: "Golden"}
```

### 6.3 Lambda 表达式

```
(参数列表) -> 表达式              // 单行，隐式返回
(参数列表) -> { 语句块 }          // 多行，需显式 return
```

- 参数类型可推断时省略：`(a, b) -> a + b`
- 单参数且无歧义时可省略括号：`x -> x * 2`
- 无参数：`() -> compute()`
- Lambda 的目标类型为仅含一个抽象方法的 interface（SAM interface）。

```army
Calculator add := (int a, int b) -> a + b
Calculator mul := (a, b) -> a * b
StringFilter notEmpty := s -> s.length() > 0
Supplier<int> five := () -> 5

// 多行 Lambda
Calculator max := (a, b) -> {
    if a > b { return a }
    return b
}
```

### 6.4 匿名函数（Go 风格 func 字面量）

```
func(参数列表) [返回类型] { 语句块 }
```

匿名函数是一等公民，支持：

- 赋值给变量
- 作为参数传递
- 作为返回值
- 立即调用（IIFE）
- 闭包（捕获外部变量）

```army
add := func(int a, int b) int {
    return a + b
}

// 立即调用
int doubled := func(int x) int { return x * 2 }(21)   // 42

// 闭包
int base := 10
adder := func(int x) int { return base + x }
int v := adder(5)                                      // 15
```

### 6.5 方法引用

#### 语法

```army
Type::staticMethod              // 静态方法引用
instance::instanceMethod        // 实例方法引用
Type::instanceMethod            // 特定类型实例方法引用
this::method                    // 当前实例方法引用
```

#### 重载函数引用 — `#n` 消除歧义

当引用的函数被重载时，必须用 `#n` 指定参数个数：

```army
Math::increment#1     // increment(int a)
Math::increment#2     // increment(int a, int delta)
Math::increment       // ❌ 编译错误：有多个重载，须用 #n
```

#### 示例

```army
// 静态方法引用
Calculator mul := Math::multiply

// 类型实例方法引用（第一个参数成为 receiver）
StringFilter long := String::isEmpty

// 当前实例方法引用
Calculator self := this::compute

// 构造方法引用
Supplier<Box<int>> factory := Box<int>::new
```

### 6.6 函数作为 SAM Interface 实现

Army 中函数是一等公民，天生隐式实现仅含一个抽象方法且签名匹配的接口（SAM interface）：

```army
interface Calculator {
    func compute(int a, int b) int
}

Calculator add := func(int a, int b) int { return a + b }
Calculator mul := (a, b) -> a * b
Calculator ref := Math::multiply
```

无需手动包装或显式声明实现关系。编译器自动将函数值转换为匹配的 SAM interface 值。

函数类型**可以显式实现 interface**：

```army
interface Handler {
    func process(string data) bool
}

// 全局 func 显式实现 Handler
func Handler process(string data) bool {
    return data.length() > 0
}
```

语法：`func InterfaceName funcName(参数) [返回类型] { 块 }`，编译器检查函数是否真正实现了 interface 的所有方法。

规则：
- 普通（非密封）interface：函数类型可以隐式实现（默认），也可以显式实现
- **密封 interface：函数类型必须显式实现**，隐式实现不被接受

---

## 7. 语句

### 7.1 if / else

```army
if x > 0 {
    return "positive"
} else if x < 0 {
    return "negative"
} else {
    return "zero"
}

// if 支持 init 语句
if val := x; val < 0 {
    return -val
}
```

### 7.2 for 循环

Army 中 for 是唯一的循环语句：

```army
// 经典三段式
for int i := 0; i < n; i = i + 1 {
    // ...
}

// 条件循环（类似 while）
for n > 0 {
    n = n - 1
}

// 无限循环
for {
    handleRequest()
}

// range 遍历
for index, value := range items {
    print(index, value)
}

// range 只取值
for _, value := range items {
    print(value)
}
```

### 7.3 switch

Army 的 switch 与 C 一致：**case 默认穿透，必须显式 `break` 终止**。Army 无 `fallthrough` 关键字（穿透已是默认行为）。

```army
// 基本 switch
switch score {
    case 90, 100:
        grade := "A"
        break
    case 80, 89:
        grade := "B"
        break
    default:
        grade := "D"
        break
}

// 每个 case 最多 4 个选项
switch day {
    case 1, 2, 3, 4:
        type := "weekday"
        break
    case 5, 9, 13, 21:
        type := "special"
        break
}

// ❌ 超过 4 个选项——编译错误，必须拆分
// case 1, 2, 3, 4, 5:    // 错误：case 最多 4 个选项

// 有意穿透：省略 break 即可
switch char {
    case 'a':
    case 'e':
    case 'i':
    case 'o':
    case 'u':
        type := "vowel"
        break
    default:
        type := "consonant"
        break
}

// switch 支持 init 语句
switch v := x * 2; v {
    case 10:
        result := "ten"
        break
    default:
        result := "other"
        break
}

// 无表达式 switch（替代 if-else 链）
switch {
    case n < 0:
        result := "negative"
        break
    case n == 0:
        result := "zero"
        break
    default:
        result := "positive"
        break
}
```

规则：

- 无 `fallthrough` 关键字——**穿透是默认行为**（与 C 一致），靠 `break` 终止
- 每个需终止的 case 必须显式写 `break`；省略则穿透到下一个 case
- 每个 `case` 最多支持 **4 个选项**（逗号分隔），超过 4 个的匹配必须拆分到多个 case
- **`default` 必须出现在所有 `case` 之后**，不可插在中间
- 对枚举进行穷举检查（缺 case 产生编译警告）
- **严禁在 `switch` 内出现 `return` 语句**——编译错误。switch 只能通过 `break` 终止 case，通过 `goto` 跳出 switch 整体。返回值必须在 switch 之后处理

// ❌ default 插在 case 中间——编译错误
// switch v {
//     case 1:
//         break
//     default:
//         break
//     case 2:
//         break
// }

### 7.4 goto

Army 的 `goto` 规则与 Go 完全一致：

**允许：**
- 跳转到同一函数内的任意标签
- 向外跳出代码块（如跳出 `for`、`switch`）

**限制：**
- **禁止跳过变量声明**——`goto` 不能跳过带 `:=` 或 `=` 的变量声明语句，确保变量始终被正确初始化
- **禁止跳入新的代码块**——不能从外部跳入 `if`、`for`、`switch` 的大括号内部
- 标签与其跳转目标必须在同一函数内

```army
// ✅ 基本用法：跳出循环
func findFirst(int[] arr, int target) int {
    int index := -1
    for int i := 0; i < arr.length(); i = i + 1 {
        if arr[i] == target {
            index = i
            goto done
        }
    }
    done:
    return index
}

// ✅ 跳出 switch（结合禁止 return 的规则）
func classify(int n) string {
    string result
    switch n {
        case 1:
            result = "one"
            goto out
        case 2:
            result = "two"
            goto out
        default:
            result = "other"
            goto out
    }
    out:
    return result
}

// ❌ 跳过变量声明——编译错误
// goto skip
// int x := 10   // 被跳过
// skip:

// ❌ 跳入代码块——编译错误
// goto inside
// if condition {
//     inside:
// }
```

---

## 8. 声明与作用域

### 8.1 变量声明

```army
类型 名 := 表达式      // 推断
类型 名 = 表达式        // 显式

int x := 10
int y = 20
Point p := Point{int x: x, int y: y}
```

### 8.2 多变量接收

```army
int a, int b := swap(x, y)
string data, error err := r.read()
```

---

## 9. Struct

### 9.1 定义

```
struct 名称 [可见性][&e] [permits A, B] [: Interface1, Interface2] {
    [static] [final] 类型 字段名 [= 初值]
}
```

初值可以是字面量，也**可以调用静态函数**：

```army
struct Config {
    static final string DEFAULT_HOST = "localhost"            // 字面量
    static final int    TIMEOUT      = computeTimeout()       // 静态函数调用
    string              host         = getDefaultHost()       // 实例字段初值也可调用静态函数
    int                 port         = 8080
}
```

规则：

- struct 内 **只允许字段**，不允许方法声明。
- **字段必须手动排序**：静态字段区在前，实例字段区在后。各区内部按 public→private 顺序排列。否则语法错误。
- 支持匿名组合：直接写另一个 struct 类型名，无字段名。

### 9.2 选项 `&e`

`&e` 禁止本文件外为 struct 增加 func：

```army
struct&e ImmutableValue {
    static final int MAX = 100
    int              value
}
// 其他文件不可为 ImmutableValue 添加方法
```

可见性与选项可组合：

```army
struct~&e PackageSecret { string key }   // 本包可见 + 禁止外部扩展
struct-&e FilePrivate { int id }         // 本文件可见 + 禁止外部扩展
```

### 9.3 匿名组合

```army
struct Animal {
    string species
    int    age
}

struct Pet {
    Animal              // 匿名组合 — 继承 species、age
    string name
}

// Pet 可以直接访问 Animal 的字段
func (Pet p) introduce() string {
    return p.name + " is a " + p.species
}
```

### 9.4 构造语法

```army
// 嵌套组合构造
Dog {
    Pet {
        Animal { int species: "canine", int age: 3 },
        string name: "Buddy",
    },
    string breed: "Golden",
}
```

### 9.5 密封 Struct（permits）

```army
struct Shape permits Circle, Rectangle, Triangle {
    string color
}

struct Circle {                                       // 在 permits 列表中
    Shape
    float4 radius
}

// struct Square { Shape; float4 side }   ❌ 不在 permits 列表中
```

规则：
- **`permits` 列表只能出现基包内的 struct、interface、全局 func**，不允许引用外部包的类型或函数
- 基包外的类型即使同包也不得出现在 permits 中，编译器直接报错

### 9.6 Struct 方法（写在 struct 外部，必须紧跟）

方法按以下顺序编写，否则语法错误：

1. **构造 func 区** — 唯一。函数名 = struct 名，无返回值。
2. **静态 func 区** — 按可见性 public→private。
3. **实例 func 区** — 按可见性 public→private。

```army
struct Counter {
    static int total = 0
}

// 1. 构造 func
func (Counter) Counter() {
    Counter.total = 0
}

// 2. 静态 func 区（public → private）
func (Counter) factory() Counter { return Counter{} }
func- (Counter) privateFactory() Counter { return Counter{} }

// 3. 实例 func 区（public → private）
func (Counter c) increment() { c.total = c.total + 1 }
func (Counter c) value() int { return c.total }
func- (Counter c) internalCheck() { print(c.total) }
```

Receiver 语法：

- 无变量名 = 静态方法：`func (Counter) factory() Counter`
- 有变量名 = 实例方法：`func (Counter c) value() int`
- `*` 前缀 = 指针 receiver：`func (Counter* c) reset()`

### 9.7 枚举（@struct）

枚举是 struct 的特化，用 `@struct` 标注：

```army
// 简单枚举
@struct Day {
    MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY, SATURDAY, SUNDAY
}

// 带数据枚举
@struct Color {
    RED(255, 0, 0)
    GREEN(0, 255, 0)
    BLUE(0, 0, 255)

    int r, int g, int b
}

// 泛型枚举
@struct Option<T> {
    SOME(T)
    NONE

    T data
}
```

规则：

- 编译器自动生成构造；开发者不可手动编写构造。
- 编译器自动生成静态方法：`values()`, `valueOf(string)`, 实例方法：`name()`, `ordinal()`。
- 方法分区规则与普通 struct 完全一致（构造→静态→实例，public→private）。

### 9.8 包内扩展

同一包内，可在其他 `.army` 文件中为一个 struct（无 `&e` 选项、可见性允许）继续扩展。扩展按区域分两种：

#### 9.8.1 扩展静态字段

使用 `&StructName { ... }` 语法为 struct 增加静态字段。**只允许静态字段，实例字段不可扩展**：

```army
// counter_extra.army — 同包内扩展文件

&Counter {                              // 扩展 Counter 的静态字段
    static int maxValue = 1000
    static string description = "counter"
}

// ❌ 不可扩展实例字段
// &Counter { int extra }               // 编译错误：实例字段不可扩展
```

静态字段扩展遵循与 struct 定义内部相同的排序规则（public→private）。

#### 9.8.2 扩展方法

同一包内，可为 struct 继续添加方法（与 struct 本文件中的方法分区规则一致）：

```army
// point_extra.army — 同包内的扩展文件
func (Point) midpoint(Point other) Point {
    return Point{int x: (Point.x + other.x) / 2, int y: (Point.y + other.y) / 2}
}
```

`&e` 选项的 struct 不可被外部扩展（静态字段和方法均不可）。

---

## 10. Interface

### 10.1 定义

```
interface 名称 [可见性] [permits A, B] {
    InterfaceA                           // 嵌入组合区
    InterfaceB

    func 方法名(参数) [返回类型]                               // 抽象方法（非默认）：无 receiver、无函数体
    func (接口名 变量名) 方法名(参数) [返回类型] { 单行代码 }     // 默认方法：有函数体 + 带 receiver（类型 = 当前 interface）
}
```

规则：

- **嵌入组合区在前，func 声明区在后**。否则语法错误。
- 所有方法均为 public。
- 嵌入组合类似 Go 的 interface 组合：嵌入的 interface 的所有方法被合并到当前 interface。
- interface 无 `: InterfaceList` 语法，通过匿名嵌入实现组合。
- func 区中不使用 `default` 关键字。默认方法（有实现）与非默认方法（抽象） **靠两个特征同时区分**： **有函数体 且 有
  receiver（类型 = 当前 interface）**。receiver 与函数体必须成对出现，只满足其一即编译错误。
- **func 区内声明顺序强制**：先声明全部非默认方法（抽象），再声明默认方法。默认方法出现在任一非默认方法之前，即语法错误。

### 10.2 嵌入组合（Go 风格）

```army
interface Reader {
    func read() (string data, error err)
}

interface Writer {
    func write(string data) error
}

interface Closer {
    func close() error
}

// 组合：嵌入多个 interface
interface ReadWriter {
    Reader
    Writer
}

// 多组合 + 自有方法
interface ReadWriteCloser {
    Reader
    Writer
    Closer

    func flush() error
}
```

### 10.3 默认方法

interface 的默认方法 **不使用 `default` 关键字**。默认方法与非默认方法 **通过两点区分**，两点必须同时成立：

> **默认方法 ⇔ 有函数体（`{ ... }`）且 有 receiver（`(接口名 变量名)`）。**

| func 声明                                    | receiver             | 函数体 | 分类                                   |
|----------------------------------------------|----------------------|--------|----------------------------------------|
| `func toString() string`                     | 无                   | 无     | 非默认方法（抽象，由实现方提供）       |
| `func (Stringer s) toUpper() string { ... }` | 有（当前 interface） | 有     | 默认方法（interface 自带实现）         |
| `func (Stringer s) foo();`                   | 有                   | 无     | ❌ 编译错误：receiver 与函数体必须成对 |
| `func bar() { ... }`                         | 无                   | 有     | ❌ 编译错误：receiver 与函数体必须成对 |

```army
interface Stringer {
    func toString() string                    // 非默认方法（抽象）：无 receiver、无函数体

    func (Stringer s) toUpper() string {      // 默认方法：有函数体 + receiver（类型 = 当前 interface）
        return Stringer.toUpperImpl(s.toString())
    }
}

func (Stringer) toUpperImpl(string s) string {
    return s.toUpperCase()
}
```

**func 区顺序规则**：在一个 interface 内， **所有非默认方法必须声明在默认方法之前**
；默认方法出现在任一非默认方法之前即语法错误。非默认方法之间、默认方法之间的相对顺序不限（上面的 Stringer
即"先抽象后默认"的合法形态）：

```army
// ❌ 错误示范：默认方法出现在非默认方法之前
// interface Bad {
//     func (Bad b) process() string {      // 语法错误：默认方法须位于所有非默认方法之后
//         return Bad.check()
//     }
//
//     func toString() string               // 非默认方法不得晚于默认方法声明
// }
```

限制：

- **两点同时成立才是默认方法**：必须有 ① 函数体 ② receiver。receiver 与 struct 实例方法一样写在 `func` 之后、方法名之前，接口名必须等于当前
  interface 的类型名。 **只满足一点（有 receiver 无函数体、或无 receiver 有函数体）都是编译错误**。
- **receiver 不支持 `*` 前缀**：interface 本身即引用语义，无指针 receiver。
- 默认方法体内 **只能通过 receiver 变量调用本 interface 的抽象方法**（如 `s.toString()`），不得用 interface 类型名直接调用实例/抽象方法。
- 默认方法最多一行代码。
- 不允许链式调用。
- 必须委托给静态方法实现真正逻辑。
- **声明顺序**：默认方法必须位于本 interface 的所有非默认方法之后，违反即语法错误。

### 10.4 密封 Interface（permits）

```army
interface Drawable permits Circle, Rectangle, Triangle {
    func draw()
}
```

规则：
- 密封 interface 的实现必须显式声明 `: Drawable`，隐式实现不被接受
- **`permits` 列表只能出现基包内的 struct、interface、全局 func**，不允许引用外部包的类型或函数

### 10.5 Interface 静态方法（写在 interface 外部，必须紧跟）

按可见性 public→private 顺序编写：

```army
func (Validator) check(bool result) bool       // public
func$ (Validator) internalCheck(string key) bool // 本包及子包
func~ (Validator) packageCheck(string key) bool  // 本包
func- (Validator) fileCheck(string key) bool     // private
```

### 10.6 Struct 实现 Interface

#### 隐式实现（鸭子类型，默认）

```army
struct Circle {
    float4 radius
}

func (Circle c) area() float4 { return Math.PI * c.radius * c.radius }
func (Circle c) perimeter() float4 { return 2 * Math.PI * c.radius }

// Circle 自动满足 ShapeLike（无需显式声明）
ShapeLike s := Circle{float4 radius: 5.0}
```

#### 显式实现（: InterfaceName）

```army
struct Triangle : ShapeLike {
    float4 base
    float4 height
}
// 编译器检查是否真的实现了 ShapeLike 的所有方法
```

- 密封 interface 必须显式实现。
- 一个 struct 可以对部分 interface 显式，其余隐式。
- **函数类型也可以显式实现 interface**，语法：`func InterfaceName funcName(参数) [返回类型] { 块 }`。编译器检查是否真正实现了 interface 的所有方法。密封 interface 必须显式实现。

### 10.7 空接口与 Any

空接口即不含任何方法声明的 interface：

```army
interface Container {
    // 没有任何方法——空接口
}
```

Army 内置一个预声明的空接口 `any`（定义于 `army.lang` 包），它可以表示一切值：类型、struct、interface、func 均可赋值给 `any`。

```army
any x := 42                        // int
x = "hello"                        // string
x := Circle{float4 radius: 5.0}    // struct
x := func(int a) int { return a }  // func
```

规则：
- 允许声明自定义空接口
- **空接口和密封接口一样只能显式实现**，隐式实现不被接受
- `any` 是预声明标识符（非关键字），可被 shadow（不推荐）

---

## 11. 函数

### 11.1 函数定义

```
func 函数名(参数列表) [返回类型] { 语句块 }
```

```army
func maxOf(int a, int b) int {
    if a > b { return a }
    return b
}

// 多返回值
func divide(int a, int b) (int quotient, int remainder) {
    quotient  = a / b
    remainder = a % b
    return
}

// 无返回值
func main() {
    print("hello")
}
```

#### 11.1.1 `func` 关键字格式

全局函数左侧**不允许有空格**（顶格书写），`func` 与函数名之间**有且仅有一个空格**，且**不可换行**：

```army
// ✅ 正确——全局 func 顶格，func 与名称间恰好 1 个空格
func add(int a, int b) int {
    return a + b
}

// ❌ 错误——全局 func 左侧有空格
    func add(int a, int b) int { ... }      // 缩进非法

// ❌ 错误——func 与函数名之间多余空格
func  add(int a, int b) int { ... }         // 两个空格

// ❌ 错误——func 与函数名之间无空格
funcadd(int a, int b) int { ... }           // funcadd 视为标识符

// ❌ 错误——func 与函数名之间换行
func
add(int a, int b) int { ... }               // 换行非法
```

### 11.2 参数默认值

语法：`类型 名称 = 值`

```army
func greet(string name = "world") string {
    return "hello " + name
}

func configure(int port = 8080, string host = "localhost", bool tls = false) {
    // ...
}
```

规则：

- 默认值参数必须在参数列表末尾。
- 有默认值的函数在效果上已实现"重载"，同有效参数个数的重载版本不允许。

### 11.3 命名参数调用

语法：`名称 = 值`

```army
greet(name = "Alice")
configure(tls = true)                     // 配合默认值，只传最后一个参数
foo(a = 1, s = "hello")
```

声明和调用统一使用 `=` 号。

### 11.4 函数重载

仅支持参数个数不同的重载：

```army
func increment(int a) int { return a + 1 }                     // ✅
func increment(int a, int delta) int { return a + delta }      // ✅
func increment(int a, int delta, string log) (int, string) {   // ✅
    return a + delta, log
}

// ❌ 参数个数相同但类型不同 — 非法
// func increment(string a) string { ... }       // 编译错误
// func increment(int a, string b) int { ... }   // 编译错误
```

**与默认值的交互**：默认值让函数在调用时等价于多种参数个数版本，因此：

```army
func foo(int a = 0) { ... }
func foo() { ... }                         // ❌ 编译错误：有效参数个数都是 0

func bar(int a, string s = "hi") { ... }
func bar(int a) { ... }                    // ❌ 编译错误：有效参数个数都是 1
```

---

## 12. 泛型

### 12.1 类型参数

与 Java 一致，无 `extends`/`super` 关键字，用 `:` 表示约束：

```army
struct Box<T> { T value }

struct SortedItem<T : Comparable<T>> { T data }

struct NumberHolder<T : Comparable<T> & Stringer> { T data }
```

### 12.2 通配符

```army
func printBox(Box<?> box) { ... }                // 无界通配符
func processNumbers(Box<? : int> box) { ... }     // 上界通配符

// ❌ 不支持 ? super Foo
```

### 12.3 泛型函数

```army
func<T> swap(T a, T b) (T, T) {
    return b, a
}
```

### 12.4 泛型 Interface

```army
interface Repository<T> {
    func find(string id) T
    func save(T entity)
}
```

---

## 13. 包系统

### 13.1 基包

每个 Army 项目必须有一个 `base-info.army` 文件定义基包：

```army
com.qinarmy.demo
```

所有源码必须在基包及其子包内。

### 13.2 Import

import 路径**不使用引号**，只能包含**小写英文、下划线、数字**三种字符，`import` 关键字与路径之间**有且仅有一个空格**。

```army
import com.qinarmy.demo.util
import com.qinarmy.demo.models
import com.qinarmy.demo.sub_package1
import com.qinarmy.demo.v2
```

规则：
- 路径不允许 `""` 引号包裹
- 路径仅允许字符集：`[a-z0-9_]` 及 `.` 分隔符
- `import` 与路径之间有且仅有一个空格：`import com.xxx`（✅）、`import  com.xxx`（❌）、`import"com.xxx"`（❌）

---

> **参考实现**：`/src/lexer.rs` — 词法分析器 | `/src/token.rs` — Token 定义 | `/src/ast.rs` — AST 节点 | `/src/main.rs` —
> 入口
