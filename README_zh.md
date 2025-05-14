# Layer7Benchmark

[English](https://github.com/shinnku-nikaidou/layer7benchmark/blob/main/README.md)

## 简介

Layer7Benchmark 是一款用于测试第 7 层（应用层）协议（如 HTTP/HTTPS）性能的基准测试工具。它能在不同负载条件下测量 Web 应用的响应时间和吞吐量。

该工具设计简洁易用，适合开发人员和系统管理员验证应用在预期流量下的表现。

注意：
请勿将此工具用于非法用途，如未经许可的攻击或破坏网站（例如 DDoS）。
本项目作者不对任何滥用行为负责。请务必合法、合规地使用。

## Features

- 完全使用 `Rust` 编写，高性能、低资源消耗

- 基于 `reqwest` 库进行 HTTP/HTTPS 请求

- 使用 `tokio` 实现异步并发

- 相比其他工具（如 `webBenchmark`），CPU 占用更低

## 使用方法

```bash
./layer7benchmark -c [并发数] -a [URL] -t [时长]

-c 或 --concurrent-count u16
    并发下载线程数（默认 2）
-u 或 --url string
    下载目标 URL（默认 https://www.google.com）
-t 或 --time u32
    下载持续时间（默认 60 秒）
--help
    显示帮助信息
-v 或 --version
    显示版本信息
-H 或 --header string
    要发送的 HTTP 头（-H 参数与 curl 命令兼容）
-i 或 --ip string
    要发送请求的目标 IP（默认根据 URL 自动解析）
    如果已知真实 IP，可通过此选项绕过 CDN 或某些 WAF，进而直接打击源站
--test
    测试模式，仅发送一次请求用于调试或测试响应，会打印请求结果和状态码
-X 或 --method string
    使用的 HTTP 方法（默认 GET，可选：GET、POST、PUT、DELETE、OPTIONS；-X 参数与 curl 命令兼容）
--timeout u64
    请求超时时间（默认 10 秒）
    若单次请求耗时超出此值则视为超时
    此超时不同于完整响应流超时，后者默认为 60 秒
--body string
    要随请求发送的请求体（默认空）
    此选项仅对 POST 和 PUT 请求有效
    可在 GET 请求中添加此选项，但会被忽略
--random
    ⚠️ 如果使用此选项，--url 选项的语法将会改变。
    总之，此程序将根据您的 --url 选项随机生成 URL。
    例如，如果您将 --url 设置为 https://www.example.com/[a-z0-9]{10}，
    程序将随机生成类似 https://www.example.com/abc123xyz0、https://www.example.com/xyz789abc1 等 URL，并向这些随机 URL 发送请求。
    如果您想使用此选项，请确保了解所设置 URL 的语法。
    此选项可以与 --test 选项结合使用。--test 选项将仅向随机生成的 URL 发送一次请求，并打印生成的 URL。
    完整语法详见下文。
```

### 示例

```bash
./layer7benchmark -u https://www.example.com --test

./layer7benchmark -u https://www.example.com -t 60 \
    --header "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3" \
    --header "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8" \
    --header "Accept-Language: en-US,en;q=0.5" \
    --header "Accept-Encoding: gzip, deflate, br" \
    --header "Connection: keep-alive" \
    --header "Cache-Control: max-age=0"

./layer7benchmark -u https://x.com/home -c 16 -t 360 --ip 172.66.0.227
```

## random URL grammar (待翻译)

In summary, this program will now randomly generate URLs based on your `--url` or `-u` option.
There are two types of grammar.

1. First is `[expr]`
2. Second is `[expr]{n}`

`expr` contains three types of characters: `a-z`, `A-Z`, `0-9`. You can combine them together.
such as `[a-zA-Z0-9]`, `[a-zA-Z]`, `[0-9]`, etc.

> note: `[a-zA-Z0-9]` will generate a random character from `a-z`, `A-Z`, `0-9`.

`n` is a number, which means how many characters you want to generate.
such as `[0-9]{3}` will generate 3 random characters from `0-9`. such as `111`, `456`, `364`, etc.

> note: `[a-zA-Z]` is exactly the same as `[a-zA-Z]{1}` or [A-Za-z].

### some examples

```bash
https://www.example.com/[0-9]{3}

# will generate a random URL like
https://www.example.com/111
https://www.example.com/456
https://www.example.com/364

https://www.example.com/random/path?foo=[a-z0-9]&bar=[a-zA-Z]{3}
# will generate a random URL like
https://www.example.com/random/path?foo=1&bar=MPL
https://www.example.com/random/path?foo=2&bar=deD
https://www.example.com/random/path?foo=a&bar=gHi

```

## License

本项目基于 MIT 协议授权，详情请参见 [LICENSE](https://github.com/shinnku-nikaidou/layer7benchmark/blob/main/License) 文件。
