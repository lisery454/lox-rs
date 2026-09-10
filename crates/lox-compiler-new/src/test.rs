#[cfg(test)]
mod tests {

    use crate::{Compiler, VM};

    /// 编译并运行 lox 代码，返回所有打印输出的拼接结果
    fn run_code(code: &str) -> String {
        let chunk = Compiler::new(code).compile().unwrap();

        let mut buffer = Vec::new();
        VM::with_writer(&mut buffer).interpret(chunk).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    // ---------- 字面量打印 ----------
    #[test]
    fn test_print_number() {
        assert_eq!(run_code("print 123;").trim(), "123");
    }

    #[test]
    fn test_print_string() {
        assert_eq!(run_code("print \"hello\";").trim(), "hello");
    }

    #[test]
    fn test_print_boolean() {
        assert_eq!(run_code("print true;").trim(), "true");
        assert_eq!(run_code("print false;").trim(), "false");
    }

    #[test]
    fn test_print_nil() {
        assert_eq!(run_code("print nil;").trim(), "<nil>");
    }

    #[test]
    fn test_print_multiple_lines() {
        let output = run_code(concat!("print \"first\";\nprint \"second\";\nprint 3;"));
        assert_eq!(output.trim(), "first\nsecond\n3");
    }

    // ---------- 算术运算 ----------

    #[test]
    fn test_arithmetic() {
        assert_eq!(run_code("print 1 + 2;").trim(), "3");
        assert_eq!(run_code("print 5 - 3;").trim(), "2");
        assert_eq!(run_code("print 4 * 6;").trim(), "24");
        assert_eq!(run_code("print 6 / 2;").trim(), "3");
    }

    #[test]
    fn test_div_result_is_float() {
        assert_eq!(run_code("print 5 / 2;").trim(), "2.5");
    }

    #[test]
    fn test_operator_precedence() {
        // * 优先于 +
        assert_eq!(run_code("print 1 + 2 * 3;").trim(), "7");
        // 左结合
        assert_eq!(run_code("print 10 - 4 - 3;").trim(), "3");
        assert_eq!(run_code("print 12 / 3 / 2;").trim(), "2");
    }

    #[test]
    fn test_grouping() {
        assert_eq!(run_code("print (1 + 2) * 3;").trim(), "9");
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(run_code("print -3;").trim(), "-3");
        assert_eq!(run_code("print -(1 + 2);").trim(), "-3");
    }

    #[test]
    fn test_string_concatenation() {
        assert_eq!(run_code("print \"con\" + \"cat\";").trim(), "concat");
    }

    // ---------- 比较与逻辑 ----------

    #[test]
    fn test_comparison() {
        assert_eq!(run_code("print 5 > 3;").trim(), "true");
        assert_eq!(run_code("print 5 < 3;").trim(), "false");
        assert_eq!(run_code("print 5 >= 5;").trim(), "true");
        assert_eq!(run_code("print 5 <= 4;").trim(), "false");
    }

    #[test]
    fn test_equality() {
        assert_eq!(run_code("print 1 == 1;").trim(), "true");
        assert_eq!(run_code("print 1 == 2;").trim(), "false");
        assert_eq!(run_code("print 1 != 2;").trim(), "true");
        // 字符串相等按值比较
        assert_eq!(run_code("print \"a\" == \"a\";").trim(), "true");
        assert_eq!(run_code("print \"a\" == \"b\";").trim(), "false");
        // nil 等于 nil
        assert_eq!(run_code("print nil == nil;").trim(), "true");
        // 不同类型不相等
        assert_eq!(run_code("print 1 == \"1\";").trim(), "false");
    }

    #[test]
    fn test_not() {
        assert_eq!(run_code("print !true;").trim(), "false");
        assert_eq!(run_code("print !false;").trim(), "true");
        // 只有 nil 和 0 是 falsy
        assert_eq!(run_code("print !nil;").trim(), "true");
        assert_eq!(run_code("print !0;").trim(), "true");
        assert_eq!(run_code("print !123;").trim(), "false");
    }

    // ---------- 变量 ----------

    #[test]
    fn test_global_var_decl() {
        assert_eq!(run_code("var a = 5; print a;").trim(), "5");
    }

    #[test]
    fn test_global_var_uninitialized_is_nil() {
        assert_eq!(run_code("var a; print a;").trim(), "<nil>");
    }

    #[test]
    fn test_global_var_assign() {
        assert_eq!(run_code("var a = 1; a = 2; print a;").trim(), "2");
    }

    #[test]
    fn test_local_var_scope() {
        assert_eq!(run_code("{ var a = 1; print a; }").trim(), "1");
    }

    #[test]
    fn test_nested_scope_shadowing() {
        let code = concat!("var a = 1;\n", "{ var a = 2; print a; }\n", "print a;");
        assert_eq!(run_code(code).trim(), "2\n1");
    }

    #[test]
    fn test_var_expression_scope() {
        // 表达式语句会被 Pop 弹出，不会污染堆栈
        assert_eq!(run_code("1 + 2; print \"done\";").trim(), "done");
    }

    // ---------- if-else 控制流 ----------

    #[test]
    fn test_if_true_executes_then_branch() {
        assert_eq!(run_code("if (true) { print \"then\"; }").trim(), "then");
        assert_eq!(
            run_code("if (2 > 1) { print \"bigger\"; }").trim(),
            "bigger"
        );
    }

    #[test]
    fn test_if_false_skips_then_branch() {
        // 条件为假，then 分支不执行，无任何输出
        assert_eq!(run_code("if (false) { print \"no\"; }").trim(), "");
        assert_eq!(run_code("if (1 > 2) { print \"no\"; }").trim(), "");
    }

    #[test]
    fn test_if_else_selects_branch() {
        // 与 test.lox 相同：条件为真走 then，为假走 else
        let code = concat!(
            "if (2 > 1) {\n",
            "    print \"then\";\n",
            "} else {\n",
            "    print \"else\";\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "then");

        let code = concat!(
            "if (1 > 2) {\n",
            "    print \"then\";\n",
            "} else {\n",
            "    print \"else\";\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "else");
    }

    #[test]
    fn test_if_else_condition_falsey() {
        // nil、0、false 都是 falsy，走 else；非零数字是 truthy，走 then
        assert_eq!(run_code("if (nil) { print \"then\"; }").trim(), "");
        assert_eq!(
            run_code("if (0) { print \"then\"; } else { print \"else\"; }").trim(),
            "else"
        );
        assert_eq!(
            run_code("if (false) { print \"then\"; } else { print \"else\"; }").trim(),
            "else"
        );
        assert_eq!(
            run_code("if (123) { print \"then\"; } else { print \"else\"; }").trim(),
            "then"
        );
    }

    #[test]
    fn test_if_without_braces() {
        // 单条语句可以省略大括号
        assert_eq!(run_code("if (true) print \"yes\";").trim(), "yes");
        assert_eq!(
            run_code("if (false) print \"yes\"; else print \"no\";").trim(),
            "no"
        );
    }

    #[test]
    fn test_nested_if_else() {
        let code = concat!(
            "if (1 < 2) {\n",
            "    if (2 < 3) {\n",
            "        print \"inner\";\n",
            "    } else {\n",
            "        print \"inner else\";\n",
            "    }\n",
            "} else {\n",
            "    print \"outer else\";\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "inner");
    }

    #[test]
    fn test_if_else_with_local_var() {
        // if 分支中定义局部变量，离开作用域后自动弹出
        let code = concat!(
            "var x = 10;\n",
            "if (x > 5) {\n",
            "    var y = x * 10;\n",
            "    print y;\n",
            "} else {\n",
            "    print 0;\n",
            "}\n",
            "print x;"
        );
        assert_eq!(run_code(code).trim(), "100\n10");
    }

    #[test]
    fn test_if_condition_does_not_pollute_stack() {
        // if 的条件值会被 Pop 清除，不会残留在栈上
        let code = concat!("if (true) print \"a\";\nprint \"b\";");
        assert_eq!(run_code(code).trim(), "a\nb");
    }

    // ---------- while 循环 ----------

    #[test]
    fn test_while_basic_count() {
        let code = concat!(
            "var i = 0;\n",
            "while (i < 5) {\n",
            "    print i;\n",
            "    i = i + 1;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "0\n1\n2\n3\n4");
    }

    #[test]
    fn test_while_falsey_condition_skips_body() {
        // false、0、nil 均为 falsy，循环体一次也不执行
        assert_eq!(run_code("while (false) { print \"no\"; }").trim(), "");
        assert_eq!(run_code("while (0) { print \"no\"; }").trim(), "");
        assert_eq!(run_code("while (nil) { print \"no\"; }").trim(), "");
    }

    #[test]
    fn test_while_single_statement_body() {
        // 循环体是单条语句，可省略大括号
        let code = concat!("var i = 0;\nwhile (i < 3) i = i + 1;\nprint i;");
        assert_eq!(run_code(code).trim(), "3");
    }

    #[test]
    fn test_while_with_local_var_in_body() {
        // 每次迭代进入新的块作用域，局部变量 x 不残留
        let code = concat!(
            "var i = 0;\n",
            "while (i < 3) {\n",
            "    var x = i * 2;\n",
            "    print x;\n",
            "    i = i + 1;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "0\n2\n4");
    }

    #[test]
    fn test_while_decrement_counter() {
        // 递减循环，验证 > 比较与减法
        let code = concat!(
            "var i = 3;\n",
            "while (i > 0) {\n",
            "    print i;\n",
            "    i = i - 1;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "3\n2\n1");
    }

    #[test]
    fn test_nested_while() {
        // 双层 while，内层用局部变量 j
        let code = concat!(
            "var i = 0;\n",
            "while (i < 3) {\n",
            "    var j = 0;\n",
            "    while (j < 2) {\n",
            "        print i * 10 + j;\n",
            "        j = j + 1;\n",
            "    }\n",
            "    i = i + 1;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "0\n1\n10\n11\n20\n21");
    }

    // ---------- for 循环 ----------

    #[test]
    fn test_for_basic() {
        // 标准三子句 for：初始化、条件、增量
        let code = concat!(
            "for (var i = 0; i < 3; i = i + 1) {\n",
            "    print i;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "0\n1\n2");
    }

    #[test]
    fn test_for_single_statement_body() {
        // 循环体单条语句省略大括号
        assert_eq!(
            run_code("for (var i = 0; i < 3; i = i + 1) print i;").trim(),
            "0\n1\n2"
        );
    }

    #[test]
    fn test_for_without_initializer() {
        // 省略初始化子句，由外部变量驱动
        let code = concat!(
            "var i = 1;\n",
            "for (; i < 4; i = i + 1) {\n",
            "    print i;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "1\n2\n3");
    }

    #[test]
    fn test_for_declared_var_does_not_leak_outside() {
        // for 中用 var 声明的变量是局部变量，限循环内，并遮蔽外层同名全局变量
        let code = concat!(
            "var i = 99;\n",
            "for (var i = 0; i < 3; i = i + 1) {\n",
            "    print i;\n",
            "}\n",
            "print i;"
        );
        assert_eq!(run_code(code).trim(), "0\n1\n2\n99");
    }

    #[test]
    fn test_for_decrement() {
        let code = concat!(
            "for (var i = 5; i > 0; i = i - 1) {\n",
            "    print i;\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "5\n4\n3\n2\n1");
    }

    #[test]
    fn test_nested_for() {
        // 双层 for，内层变量用完即弹栈，互不干扰
        let code = concat!(
            "for (var i = 1; i <= 3; i = i + 1) {\n",
            "    for (var j = 1; j <= 2; j = j + 1) {\n",
            "        print i * 10 + j;\n",
            "    }\n",
            "}"
        );
        assert_eq!(run_code(code).trim(), "11\n12\n21\n22\n31\n32");
    }

    #[test]
    fn test_for_updates_outer_variable() {
        // 循环体内访问外层全局变量
        let code = concat!(
            "var n = 0;\n",
            "for (var i = 0; i < 3; i = i + 1) {\n",
            "    n = n + 100;\n",
            "}\n",
            "print n;"
        );
        assert_eq!(run_code(code).trim(), "300");
    }

    // ---------- and 运算符 ----------

    #[test]
    fn test_and_truth_table() {
        assert_eq!(run_code("print true and true;").trim(), "true");
        assert_eq!(run_code("print true and false;").trim(), "false");
        assert_eq!(run_code("print false and true;").trim(), "false");
        assert_eq!(run_code("print false and false;").trim(), "false");
    }

    #[test]
    fn test_and_returns_operand_not_boolean() {
        // lox 中 and 返回操作数值本身，而非布尔
        assert_eq!(run_code("print 1 and 2;").trim(), "2");
        assert_eq!(run_code("print 1 and nil;").trim(), "<nil>");
        assert_eq!(run_code("print \"a\" and \"b\";").trim(), "b");
    }

    #[test]
    fn test_and_short_circuit() {
        // 左侧 falsy 时不求值右侧：若右侧执行会得到 3，短路后直接返回左侧值
        assert_eq!(run_code("print false and (1 + 2);").trim(), "false");
        assert_eq!(run_code("print 0 and (1 + 2);").trim(), "0");
        assert_eq!(run_code("print nil and (1 + 2);").trim(), "<nil>");
    }

    #[test]
    fn test_and_left_associative() {
        assert_eq!(run_code("print 1 and 2 and 3;").trim(), "3");
        assert_eq!(run_code("print true and nil and 3;").trim(), "<nil>");
    }

    // ---------- or 运算符 ----------

    #[test]
    fn test_or_truth_table() {
        assert_eq!(run_code("print false or false;").trim(), "false");
        assert_eq!(run_code("print false or true;").trim(), "true");
        assert_eq!(run_code("print true or false;").trim(), "true");
        assert_eq!(run_code("print true or true;").trim(), "true");
    }

    #[test]
    fn test_or_returns_operand_not_boolean() {
        assert_eq!(run_code("print false or 42;").trim(), "42");
        assert_eq!(run_code("print false or \"default\";").trim(), "default");
        assert_eq!(run_code("print 7 or 9;").trim(), "7");
    }

    #[test]
    fn test_or_short_circuit() {
        // 左侧 truthy 时不求值右侧：若右侧执行会得到 3，短路后直接返回左侧值
        assert_eq!(run_code("print true or (1 + 2);").trim(), "true");
        assert_eq!(run_code("print 1 or (1 + 2);").trim(), "1");
        assert_eq!(run_code("print \"x\" or (1 + 2);").trim(), "x");
    }

    #[test]
    fn test_or_falls_through_on_falsy() {
        // nil 和 0 是 falsy，继续求值右侧
        assert_eq!(run_code("print nil or 5;").trim(), "5");
        assert_eq!(run_code("print 0 or 5;").trim(), "5");
        assert_eq!(run_code("print false or 5;").trim(), "5");
    }

    #[test]
    fn test_or_left_associative() {
        assert_eq!(run_code("print 0 or 0 or 5;").trim(), "5");
        assert_eq!(run_code("print 0 or nil or 5;").trim(), "5");
        assert_eq!(run_code("print 3 or 0 or 5;").trim(), "3");
    }

    // ---------- and / or 优先级与组合 ----------

    #[test]
    fn test_and_binds_tighter_than_or() {
        // and 优先级(3) 高于 or(2)：解析为 true or (false and false) => true
        assert_eq!(run_code("print true or false and false;").trim(), "true");
        // 若按 (true or false) and false 解析，结果应为 false
        assert_eq!(run_code("print (true or false) and false;").trim(), "false");
    }

    #[test]
    fn test_logic_with_comparison() {
        // 比较优先级高于 and/or
        assert_eq!(run_code("print 1 < 2 and 2 < 3;").trim(), "true");
        assert_eq!(run_code("print 1 == 2 or 2 == 2;").trim(), "true");
        assert_eq!(run_code("print 1 > 2 or 3 > 4;").trim(), "false");
    }

    #[test]
    fn test_logic_with_unary_not() {
        // ! 优先级高于 and/or，先对操作数取反
        assert_eq!(run_code("print !false and !false;").trim(), "true");
        assert_eq!(run_code("print !true or !true;").trim(), "false");
    }

    #[test]
    fn test_logic_in_if_condition() {
        assert_eq!(
            run_code("if (true and false) { print \"y\"; } else { print \"n\"; }").trim(),
            "n"
        );
        assert_eq!(
            run_code("if (false or true) { print \"y\"; } else { print \"n\"; }").trim(),
            "y"
        );
    }

    // ---------- 其他 ----------

    #[test]
    fn test_line_comment() {
        assert_eq!(run_code("// comment\n print 1 + 1;").trim(), "2");
    }
}
