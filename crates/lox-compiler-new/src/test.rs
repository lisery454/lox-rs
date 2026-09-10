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

    // ---------- 其他 ----------

    #[test]
    fn test_line_comment() {
        assert_eq!(run_code("// comment\n print 1 + 1;").trim(), "2");
    }
}
