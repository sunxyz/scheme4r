# scheme4r R7RS 完整重构方案

## 1. 目标和结论

这次重构按“全量替换”处理，不在当前实现上继续修补。现有 `parser`、`interpreter`、`procedure/*`、`types/*` 的设计都更接近一个实验性原型，不适合继续演进到完整的 R7RS 实现。

最终目标是把 `scheme4r` 重建成一个面向嵌入式使用的 R7RS-small 解释器，核心只保留两个一等入口：

```rust
pub fn eval(source: &str) -> Result<Value, SchemeError>;

pub fn interpreter(source: &str, env: EnvRef) -> Result<Value, SchemeError>;
```

约束如下：

- `eval` 用全新的标准环境执行输入字符串，返回最后一个表达式的值。
- `interpreter` 接受字符串和外部注入环境，执行后保留环境副作用，适合宿主程序集成、REPL、脚本加载和测试。
- 输入必须支持多表达式，不再要求当前这种把多个 form 包进一层大列表。
- 输出语义以 R7RS 为准，不再兼容当前实现里的 `nil`、全角引号 `’`、`float -> isize` 这类非标准行为。

## 2. 为什么必须重写

当前代码的问题不是“缺一些功能”，而是基础抽象已经和 R7RS 目标冲突：

- `parser.rs` 基于字符串切片和括号位置猜测，没有稳定 token 流、没有 source span、对 quote/string/vector 的处理不可靠。
- `Type::float_of` 直接把 `f64` 转成 `isize`，数值系统从根上不符合标准。
- 当前把 `nil` 当作特殊值使用，但 R7RS 需要区分空表 `()`、布尔值、EOF、unspecified 等运行时对象。
- `interpreter.rs` 把“符号查找 / 过程应用 / 特殊形式”揉在一起，没有独立的扩展阶段，无法正确支撑 `syntax-rules`、`define-syntax`、`let-syntax`、`quasiquote`。
- `procedure/*` 的组织方式是按功能散落的内建函数集合，不适合表达“primitive procedure / special form / macro transformer / derived form bootstrap”这几种不同层级。
- `env.rs` 只支持简单层级查找，缺少绑定单元、递归绑定初始化、错误类型和库级命名空间。
- 现有测试数量有限，而且大多绑定于当前实现细节，不是面向 R7RS 语义的验收测试。

换句话说，这不是“在旧房子上加一层楼”，而是“保留 crate 名字，重建承重结构”。

## 3. 可借鉴 `scheme4j` 的地方

`/home/bloom/data/i-work/github.com/weekend-project-space/scheme4j` 可以借鉴的是工程分层思路，不是直接照搬其值模型：

- 可以借鉴：
  - `Scheme.eval(String)` 这种清晰的对外入口。
  - `Parser / Evaluator / Environment / stdlib` 的模块边界。
  - 标准库按主题注册 primitive 的方式。
  - `eval(String)` 支持一次执行多个 top-level form。

- 不应照搬：
  - `scheme4j` 里“列表是独立 ArrayList 类型，不以 pair 为基础”的设计不符合 R7RS。
  - 它没有单独的宏展开阶段，R7RS 完整实现必须加入 expander。
  - 它当前更像“可运行的最小核”，而不是严格的 R7RS-small 完整实现。

因此我们参考它的分层方式，但运行时模型仍然按 R7RS 重新设计。

## 4. 目标范围

最终验收目标定为 **R7RS-small 完整实现**，包含：

- 标准 reader 语法。
- 核心表达式求值。
- proper tail recursion。
- `syntax-rules` 卫生宏。
- 标准数据类型：number、boolean、pair/list、symbol、char、string、vector、bytevector、procedure、port、EOF object、unspecified。
- `(scheme base)` 为核心的标准过程和派生表达式。
- `read` / `write` / `display` / `load` / `eval` / 文件与端口相关能力。
- 库系统和 `import`/`define-library`。

实际实施上会分阶段交付，但“完成”只在以上范围达标后宣称。

## 5. 新架构

整体流程改为：

```text
source
  -> lexer
  -> tokens
  -> reader/parser
  -> datum / syntax object
  -> expander
  -> core expr
  -> evaluator
  -> value
```

这里最重要的变化有两个：

- `reader` 和 `evaluator` 之间加入独立的 `expander`，专门处理 `syntax-rules`、关键字绑定、派生表达式和 hygienic macro。
- 解析阶段先得到 `datum`，再把可求值表达式编译成 `Expr`，而不是直接把读入结构当运行时值硬求值。

## 6. 建议目录结构

建议把 `src/` 重新组织成下面这套骨架：

```text
src/
  lib.rs
  api.rs
  error.rs
  reader/
    mod.rs
    lexer.rs
    token.rs
    span.rs
    datum.rs
    parser.rs
  syntax/
    mod.rs
    object.rs
    expand.rs
    pattern.rs
    template.rs
    keyword.rs
  core/
    mod.rs
    expr.rs
  runtime/
    mod.rs
    value.rs
    number.rs
    pair.rs
    procedure.rs
    environment.rs
    port.rs
    symbol.rs
  eval/
    mod.rs
    engine.rs
    special_form.rs
    apply.rs
  stdlib/
    mod.rs
    base.rs
    numbers.rs
    list.rs
    vector.rs
    string.rs
    char.rs
    bytevector.rs
    port.rs
    file.rs
    eval.rs
    process_context.rs
    time.rs
  bootstrap/
    base.scm
    derived.scm
    libraries.scm
  tests/
    reader.rs
    eval_core.rs
    macros.rs
    libraries.rs
    conformance.rs
```

说明：

- `api.rs` 只暴露 `eval` 和 `interpreter`。
- `reader` 负责把字符串读成 datum。
- `syntax` 负责 hygienic macro 和关键字环境。
- `core::expr` 是扩展后的核心表达式 IR。
- `runtime` 放运行时值、环境和端口。
- `eval` 只做求值，不承担词法解析和宏展开职责。
- `bootstrap/*.scm` 用于实现适合用 Scheme 自举的派生表达式和部分库函数，尽量减少 Rust 侧硬编码。

## 7. 两个核心 API 的设计

建议保持用户要求的两个入口，同时把内部状态隐藏起来：

```rust
pub type EnvRef = Rc<RefCell<Environment>>;

pub fn eval(source: &str) -> Result<Value, SchemeError> {
    let env = Environment::standard();
    interpreter(source, env)
}

pub fn interpreter(source: &str, env: EnvRef) -> Result<Value, SchemeError>;
```

设计约束：

- `interpreter` 一次读取并执行 source 中的全部 form，返回最后一个值。
- `define`、`set!`、`define-syntax`、`import` 等副作用直接写回注入环境。
- `eval` 不复用全局静态可变环境，避免跨调用污染。
- 如果后面需要更高性能，可以额外提供内部 `Engine` 结构缓存标准库和 symbol interner，但对外 API 仍保持这两个入口。

## 8. 核心实现细节

### 8.1 Reader / Lexer

第一层必须先把“读”这件事做标准，而不是继续用字符串拼接。

需要支持：

- `(` `)` `.` `'` `` ` `` `,` `,@`
- `#t` `#f`
- `#\a`、`#\space`、`#\newline`
- 字符串与转义序列
- 符号
- 向量 `#(...)`
- 字节向量 `#u8(...)`
- 注释 `;`
- datum comment `#;`
- 多个 top-level form 连续出现
- source span，至少保留 line/column

reader 输出应为 `Datum`，而不是当前的运行时 `Type`。这是后续 quote、macro、read/write 正确性的前提。

### 8.2 宏展开层

这是这次重写里最关键、也是当前实现完全缺失的一层。

需要引入：

- `SyntaxObject { datum, scopes, span }`
- 关键字环境：区分变量绑定和语法绑定
- `syntax-rules` pattern matcher
- ellipsis `...` 展开
- literal identifier 匹配
- hygienic rename / scope-set 机制

实现建议：

- 不走“把宏当普通过程”这条路。
- reader 产出 datum 后，先包装为 syntax object。
- expander 负责把表层 Scheme 语法展开成较小的一组 `Expr` 核心节点。
- `let`、`let*`、`letrec`、`cond`、`case`、`and`、`or`、`when`、`unless`、`do` 等能用宏表达的，优先通过 bootstrap 宏实现，而不是全部写死在 Rust evaluator 里。

这一步做对了，后面 `define-syntax`、库系统和 derived forms 才不会越写越乱。

### 8.3 运行时值模型

当前 `Type` 需要完全替换。新的 `Value` 至少应包含：

```rust
enum Value {
    Boolean(bool),
    Number(Number),
    Symbol(SymbolId),
    Character(char),
    String(StringRef),
    Pair(PairRef),
    EmptyList,
    Vector(VectorRef),
    ByteVector(ByteVectorRef),
    Procedure(ProcedureRef),
    Port(PortRef),
    EofObject,
    Unspecified,
}
```

关键点：

- `EmptyList` 必须和 `Unspecified` 分开，不能再用 `Nil` 一把梭。
- pair、string、vector、bytevector 要支持可变语义。
- list 不是单独的基本存储结构，而是 pair 链。
- `eq?`、`eqv?`、`equal?` 要基于这个值模型分别实现。

### 8.4 数值系统

如果目标真的是完整 R7RS-small，数值系统不能继续简化成 `isize`。

建议：

- 使用 `num-bigint`、`num-rational`、`num-complex` 搭建数值塔。
- 区分 exact / inexact。
- 内部表示可以是：

```rust
enum Number {
    ExactInteger(BigInt),
    ExactRational(BigRational),
    InexactReal(f64),
    Complex(ComplexValue),
}
```

- reader 解析数字时就保留 exactness 和 radix 信息。
- 算术 primitive 统一走数值提升规则，不在具体过程里各自写一套转换逻辑。

如果没有这一步，`number?`、`exact?`、`inexact?`、`= / < / >`、`quotient`、`remainder`、`modulo`、`sqrt`、复数相关过程都很难收敛。

### 8.5 环境模型

环境建议改成“frame + binding cell”：

```rust
struct Environment {
    parent: Option<EnvRef>,
    variables: HashMap<SymbolId, CellRef>,
    syntactic_keywords: HashMap<SymbolId, KeywordBinding>,
}
```

这样做的原因：

- `set!` 修改的是绑定单元，不是简单覆盖 clone。
- `letrec` / 内部定义需要先占位再回填。
- 变量和语法关键字需要分开管理。
- 库导入时可以做符号层级映射，而不是全靠字符串拷贝。

### 8.6 求值器

求值器建议使用显式循环或 trampoline，保证 proper tail recursion。

核心能力：

- 自求值对象
- 变量查找
- 特殊形式求值
- 过程应用
- 闭包创建
- primitive 调用
- tail position 复用当前控制流

真正需要保留在 evaluator 内部的特殊形式应尽量少，只留下必须的内核：

- `quote`
- `lambda`
- `if`
- `set!`
- `define`
- `define-syntax`
- `begin`
- `let-syntax`
- `letrec-syntax`
- `import`

其他派生形式尽量在扩展阶段解决。

### 8.7 过程模型

过程至少分三类：

- PrimitiveProcedure
- ClosureProcedure
- MacroTransformer

其中：

- primitive 接收已求值参数还是原始表达式，必须显式声明，不能像当前 `ApplyArgs` 一样混在一起。
- closure 捕获定义时环境。
- macro transformer 只存在于 expand 阶段，不进入普通运行时求值分派。

### 8.8 端口和 IO

为了支持 `read`、`write`、`load`、文件 API，需要统一端口抽象：

- 文本输入端口
- 文本输出端口
- 二进制输入端口
- 二进制输出端口
- current-input-port / current-output-port 的动态绑定

默认实现可以先围绕字符串端口和文件端口展开，后续再补更丰富的宿主集成。

### 8.9 错误系统

新的错误类型必须替换现有 `panic!`/`expect()`。

建议统一成：

```rust
struct SchemeError {
    kind: ErrorKind,
    message: String,
    span: Option<Span>,
    irritants: Vec<Value>,
}
```

至少区分：

- ReadError
- SyntaxError
- ExpandError
- RuntimeError
- TypeError
- ArityError
- IoError
- LibraryError

这样 `eval` 和 `interpreter` 才能稳定作为嵌入式 API 对外暴露。

## 9. 分阶段实施计划

### 阶段 0：冻结旧实现，搭新骨架

目标：

- 新建 `reader / syntax / core / runtime / eval / stdlib / bootstrap` 目录。
- `lib.rs` 改成以新模块为中心。
- 保留旧代码仅用于过渡编译，不再继续往旧模块加功能。

交付物：

- 新目录骨架。
- 新 `SchemeError`、`Value`、`Environment` 空实现。
- 新 API 占位。

### 阶段 1：完成 reader

目标：

- 实现 lexer、token、parser、datum。
- 支持多 form 输入、标准 quote 语法、字符串、字符、vector、bytevector、comment。

验收：

- reader 测试全部通过。
- 不再依赖当前 `parser.rs`。

### 阶段 2：完成运行时基础

目标：

- 实现 `Value`、pair、symbol、environment、procedure、number 基础结构。
- 搭好标准环境注册入口。

验收：

- 可以创建环境、定义变量、查找变量、构造基本值。

### 阶段 3：完成最小求值内核

目标：

- 支持 `quote`、`if`、`define`、`lambda`、`set!`、`begin`、应用。
- `eval` / `interpreter` 能正确执行多表达式 source。

验收：

- 算术、闭包、递归、变量更新测试通过。

### 阶段 4：加入 hygienic macro 和派生形式

目标：

- 实现 `syntax-rules`、`define-syntax`、`let-syntax`、`letrec-syntax`。
- 用 bootstrap 宏实现 `and/or/cond/case/let/let*/letrec/when/unless` 等派生形式。
- 补齐 `quasiquote` / `unquote` / `unquote-splicing`。

验收：

- 宏展开测试和 quote/quasiquote 测试通过。
- 当前 `syntax.rs` 里的样例迁移成标准化测试。

### 阶段 5：补齐标准值和标准过程

目标：

- 数值塔、字符串、字符、列表、向量、字节向量、等价性谓词、类型谓词。
- map、for-each、apply、call/cc 相关能力按优先级引入。

验收：

- `(scheme base)` 主体通过。

### 阶段 6：端口、文件、eval、load、read/write

目标：

- 文件端口、字符串端口、`read`、`write`、`display`、`newline`、`load`、`eval`。
- current input/output port 行为稳定。

验收：

- 脚本加载和读写回环测试通过。

### 阶段 7：库系统与 import

目标：

- `define-library`
- `import`
- `export`
- 库查找和注册
- 内建 `(scheme ...)` 库装配

验收：

- 标准库可通过导入方式访问，不再只是“全部灌进根环境”。

### 阶段 8：一致性清理和删除旧代码

目标：

- 切换默认导出到新实现。
- 删除旧的 `src/parser.rs`、`src/interpreter.rs`、`src/env.rs`、`src/procedure/*`、`src/types/*` 和旧测试。
- 更新 README 和示例。

验收：

- 仓库中不再保留旧实现并行路径。

## 10. 删除和迁移策略

虽然目标是“旧代码全部不要”，但实际施工时不建议第一步就直接删光，否则没有任何可对照的回归点。

建议策略：

- 第一步先新增新实现并让新模块独立编译。
- 第二步用新 API 接管 crate 导出。
- 第三步在新测试通过后整体删除旧目录。

也就是说，工程过程上是“先并行、后切换、再删除”，但架构上是“全量替换，不复用旧实现”。

## 11. 测试与验收标准

新的测试不再围绕旧实现细节写，而围绕 R7RS 语义写。

测试层次建议如下：

- reader 单测：token、datum、quote、vector、bytevector、comment、span。
- expand 单测：`syntax-rules`、ellipsis、literal identifier、hygiene。
- evaluator 单测：特殊形式、闭包、递归、tail call。
- stdlib 单测：numbers、pair/list、string、char、vector、bytevector、predicate。
- integration 测试：从字符串执行完整脚本。
- conformance fixture：把 R7RS 示例和自定义 `.scm` 脚本作为 golden case。

完成判定建议至少满足：

- 所有公开 API 返回 `Result`，不再依赖 `panic!` 作为常规错误路径。
- `eval` 和 `interpreter` 支持多 form 输入。
- 标准 quote 使用 ASCII `'`，不再接受全角 `’` 作为主路径。
- proper tail recursion 有专项回归测试。
- `syntax-rules` 具备卫生性，而不是简单字符串替换。

## 12. 推荐依赖

为了把实现做稳，建议接受少量基础依赖，而不是所有东西手搓：

- `thiserror`：错误定义。
- `num-bigint`：大整数。
- `num-rational`：有理数。
- `num-complex`：复数。
- `once_cell` 或 `lazy_static`：全局只读表。
- `lasso` 可选：symbol interning。

如果后续希望保持依赖极简，可以把 interner 放到第二阶段再决定，但数值塔相关依赖基本值得一开始就引入。

## 13. 我建议的落地顺序

按工程风险排序，最稳的顺序是：

1. 先把 reader 做对。
2. 再把运行时值和环境做稳。
3. 然后做最小 evaluator。
4. 接着补 expander 和 `syntax-rules`。
5. 最后再往上加标准库、端口、库系统。

原因很简单：reader、value model、environment 是地基；宏系统是承重墙；标准库只是往上装修。当前仓库最不应该做的事，就是继续在旧 `parser + procedure + interpreter` 路线上叠功能。

## 14. 下一步施工建议

如果按这份方案继续推进，我建议下一步直接开始做下面三件事：

1. 重写 `src/lib.rs` 和模块骨架，建立新目录结构。
2. 先落 `error.rs`、`reader/*`、`runtime/value.rs`、`runtime/environment.rs` 的第一版。
3. 用最小可运行路径打通 `eval("(+ 1 2)")` 和 `interpreter("(define x 1)", env)`。

做到这一步后，后续补宏系统和标准库就会顺很多。
