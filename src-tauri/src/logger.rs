/// Rust Logger 模块
/// 参考 src/common/Logger.ts 设计
pub struct Logger;

impl Logger {
    const LINE: &'static str = "==================================================";
    const SMALL_LINE: &'static str = "--------------------------------------------------";

    /// 格式化参数为字符串
    #[allow(dead_code)]
    fn format_args(args: &[impl std::fmt::Debug]) -> String {
        if args.is_empty() {
            return String::new();
        }
        args.iter()
            .map(|item| format!("{:?}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 打印格式化后的日志
    #[allow(dead_code)]
    fn print(level: &str, title: &str, content: &str) {
        let log = if content.is_empty() {
            format!("{}\n[{}] {}\n{}", Self::LINE, level, title, Self::LINE)
        } else {
            format!(
                "{}\n[{}] {}\n{}\n{}\n{}",
                Self::LINE,
                level,
                title,
                Self::SMALL_LINE,
                content,
                Self::LINE
            )
        };
        println!("{}", log);
    }

    /// 信息日志
    #[allow(dead_code)]
    pub fn info<T: std::fmt::Debug>(title: &str, args: &[T]) {
        let content = Self::format_args(args);
        Self::print("INFO", title, &content);
    }

    /// 错误日志
    #[allow(dead_code)]
    pub fn error<T: std::fmt::Debug>(title: &str, args: &[T]) {
        let content = Self::format_args(args);
        Self::print("ERROR", title, &content);
    }

    /// 成功日志
    #[allow(dead_code)]
    pub fn success<T: std::fmt::Debug>(title: &str, args: &[T]) {
        let content = Self::format_args(args);
        Self::print("SUCCESS", title, &content);
    }
}
