import { invoke } from "@tauri-apps/api/core";

class Logger {
  // 分隔线
  private readonly LINE = "==================================================";
  private readonly SMALL_LINE =
    "--------------------------------------------------";

  // 发送给 Rust 打印
  private async print(level: string, title: string, content: string) {
    const log = `
${this.LINE}
[${level}] ${title}
${this.SMALL_LINE}
${content}
${this.LINE}
`;
    await invoke("js_log", { message: log.trim() });
  }

  // 信息日志
  async info(title: string, ...args: any[]) {
    const msg = this.formatArgs(args);
    await this.print("INFO", title, msg);
  }

  // 错误日志
  async error(title: string, ...args: any[]) {
    const msg = this.formatArgs(args);
    await this.print("ERROR", title, msg);
  }

  // 成功日志
  async success(title: string, ...args: any[]) {
    const msg = this.formatArgs(args);
    await this.print("SUCCESS", title, msg);
  }

  // 把所有参数转成可打印字符串
  private formatArgs(args: any[]): string {
    return args
      .map((item) => {
        if (typeof item === "object") {
          try {
            return JSON.stringify(item, null, 2);
          } catch {
            return String(item);
          }
        }
        return String(item);
      })
      .join("\n");
  }
}

export default new Logger();
