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

### 2.1 硬关键字（43 个）

#### 基本关键字（27 个）

```
break        default      func         interface    select
case         defer        go           map          struct
chan         else         goto         package      switch
static       final        if           range        type
continue     for          import       return       var
```

#### 类型关键字（16 个）

```
byte         int2         int4         int8
ubyte        uint2        uint4        uint8
int          uint
float4       float8
uintptr
rune
string       decimal
```

> **注意**：
> - Army 无 `const` 关键字，常量由 `static final` 组合表达。
> - Army 无 `fallthrough` 关键字，switch 各 case 自动终止。
> - 类型名是硬关键字，不可被 shadow（与 Go 不同，Go 中类型名是预声明标识符）。

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

### 4.1 声明风格

类型在左，变量在右（C/Java 系）：

```army
int x := 10          // 声明 + 推断
int y = 20           // 显式类型声明
float f := 3.14
string s := "hello"
bool b := true
```

### 4.2 可见性（6 种）

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
| `float4`  | 4 字节浮点             |
| `float8`  | 8 字节浮点             |
| `rune`    | Unicode 码点           |
| `string`  | 不可变字符串           |
| `decimal` | 精确小数               |
| `bool`    | 布尔值                 |
| `error`   | 错误类型               |

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

```army
// 基本 switch
switch score {
    case 90, 100:
        return "A"
    case 80, 89:
        return "B"
    default:
        return "D"
}

// switch 支持 init 语句
switch v := x * 2; v {
    case 10:
        return "ten"
    default:
        return "other"
}

// 无表达式 switch（替代 if-else 链）
switch {
    case n < 0:
        return "negative"
    case n == 0:
        return "zero"
    default:
        return "positive"
}
```

规则：

- 无 `fallthrough` 关键字，每个 case 自动终止（无需 `break`）
- 显式 `break` 可用于提前跳出 switch
- 对枚举进行穷举检查（缺 case 产生编译警告）

### 7.4 goto

```army
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
struct 名称 [可见性][&选项] [permits A, B] [: Interface1, Interface2] {
    [static] [final] 类型 字段名 [= 初值]
}
```

规则：

- struct 内 **只允许字段**，不允许方法声明。
- **字段必须手动排序**：静态字段区在前，实例字段区在后。各区内部按 public→private 顺序排列。否则语法错误。
- 支持匿名组合：直接写另一个 struct 类型名，无字段名。

### 9.2 选项 `&ne`

`&ne`（no-extension）禁止本文件外为 struct 增加 func：

```army
struct&ne ImmutableValue {
    static final int MAX = 100
    int              value
}
// 其他文件不可为 ImmutableValue 添加方法
```

可见性与选项可组合：

```army
struct~&ne PackageSecret { string key }   // 本包可见 + 禁止外部扩展
struct-&ne FilePrivate { int id }         // 本文件可见 + 禁止外部扩展
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

### 9.8 包内扩展方法

同一包内，可在其他 `.army` 文件中为一个 struct（无 `&ne` 选项、可见性允许）继续添加方法：

```army
// point_extra.army — 同包内的扩展文件
func (Point) midpoint(Point other) Point {
    return Point{int x: (Point.x + other.x) / 2, int y: (Point.y + other.y) / 2}
}
```

`&ne` 选项的 struct 不可被外部扩展。

---

## 10. Interface

### 10.1 定义

```
interface 名称 [可见性] [permits A, B] {
    InterfaceA                           // 嵌入组合区
    InterfaceB

    func 方法名(参数) [返回类型]          // 抽象方法（func 区）
    default func 方法名(参数) [返回类型] { 单行代码 }
}
```

规则：

- **嵌入组合区在前，func 声明区在后**。否则语法错误。
- 所有方法均为 public。
- 嵌入组合类似 Go 的 interface 组合：嵌入的 interface 的所有方法被合并到当前 interface。
- interface 无 `: InterfaceList` 语法，通过匿名嵌入实现组合。

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

### 10.3 Default 方法

```army
interface Stringer {
    func toString() string

    default func toUpper() string {
        return Stringer.toUpperImpl(Stringer.toString())
    }
}

func (Stringer) toUpperImpl(string s) string {
    return s.toUpperCase()
}
```

限制：

- `default func` 最多一行代码。
- 不允许链式调用。
- 必须委托给静态方法实现真正逻辑。

### 10.4 密封 Interface（permits）

```army
interface Drawable permits Circle, Rectangle, Triangle {
    func draw()
}
```

密封 interface 的实现必须显式声明 `: Drawable`，隐式实现不被接受。

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

```army
import "com.qinarmy.demo.util"
import "com.qinarmy.demo.models"
```

---

> **参考实现**：`/src/lexer.rs` — 词法分析器 | `/src/token.rs` — Token 定义 | `/src/ast.rs` — AST 节点 | `/src/main.rs` —
> 入口
